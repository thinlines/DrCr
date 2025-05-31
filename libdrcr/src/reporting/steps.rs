/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022-2025  Lee Yingtong Li (RunasSudo)

	This program is free software: you can redistribute it and/or modify
	it under the terms of the GNU Affero General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU Affero General Public License for more details.

	You should have received a copy of the GNU Affero General Public License
	along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! This module contains concrete [ReportingStep] implementations

use std::collections::HashMap;
use std::fmt::Display;

use async_trait::async_trait;
use chrono::Datelike;
use tokio::sync::RwLock;

use crate::account_config::kinds_for_account;
use crate::model::transaction::{
	update_balances_from_transactions, Posting, Transaction, TransactionWithPostings,
};
use crate::reporting::types::{BalancesAt, DateStartDateEndArgs, ReportingProductId, Transactions};
use crate::util::{get_eofy, sofy_from_eofy};
use crate::QuantityInt;

use super::calculator::ReportingGraphDependencies;
use super::dynamic_report::{
	entries_for_kind, DynamicReport, DynamicReportEntry, Row, Section,
};
use super::executor::ReportingExecutionError;
use super::types::{
	BalancesBetween, DateArgs, MultipleDateArgs, MultipleDateStartDateEndArgs, ReportingContext,
	ReportingProductKind, ReportingProducts, ReportingStep, ReportingStepArgs, ReportingStepId,
	VoidArgs,
};

/// Call [ReportingContext::register_lookup_fn] for all steps provided by this module
pub fn register_lookup_fns(context: &mut ReportingContext) {
	AllTransactionsExceptEarningsToEquity::register_lookup_fn(context);
	AllTransactionsExceptEarningsToEquityBalances::register_lookup_fn(context);
	AllTransactionsIncludingEarningsToEquity::register_lookup_fn(context);
	BalanceSheet::register_lookup_fn(context);
	CombineOrdinaryTransactions::register_lookup_fn(context);
	CombineOrdinaryTransactionsBalances::register_lookup_fn(context);
	CurrentYearEarningsToEquity::register_lookup_fn(context);
	DBBalances::register_lookup_fn(context);
	DBTransactions::register_lookup_fn(context);
	IncomeStatement::register_lookup_fn(context);
	PostUnreconciledStatementLines::register_lookup_fn(context);
	RetainedEarningsToEquity::register_lookup_fn(context);
	TrialBalance::register_lookup_fn(context);
}

/// Target representing all transactions except charging current year and retained earnings to equity (returns transaction list)
///
/// By default, this is [CombineOrdinaryTransactions] and, if requested, [CalculateIncomeTax].
///
/// Used as the basis for the income statement.
#[derive(Debug)]
pub struct AllTransactionsExceptEarningsToEquity {
	pub args: DateArgs,
}

impl AllTransactionsExceptEarningsToEquity {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"AllTransactionsExceptEarningsToEquity",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsExceptEarningsToEquity {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for AllTransactionsExceptEarningsToEquity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for AllTransactionsExceptEarningsToEquity {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptEarningsToEquity",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		// AllTransactionsExceptEarningsToEquity always depends on CombineOrdinaryTransactions at least
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::Transactions,
			args: Box::new(self.args.clone()),
		}]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		combine_transactions_of_all_dependencies(self.id(), dependencies, products).await
	}
}

/// Target representing all transactions except charging current year and retained earnings to equity (returns balances)
///
/// By default, this is [CombineOrdinaryTransactions] and, if requested, [CalculateIncomeTax].
///
/// Used as the basis for the income statement.
#[derive(Debug)]
pub struct AllTransactionsExceptEarningsToEquityBalances {
	pub product_kinds: &'static [ReportingProductKind; 1], // Must have single member - represented as static array for compatibility with ReportingStepId
	pub args: Box<dyn ReportingStepArgs>,
}

impl AllTransactionsExceptEarningsToEquityBalances {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"AllTransactionsExceptEarningsToEquity",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			|a| Self::from_args(&[ReportingProductKind::BalancesAt], a),
		);

		context.register_lookup_fn(
			"AllTransactionsExceptEarningsToEquity",
			&[ReportingProductKind::BalancesBetween],
			Self::takes_args,
			|a| Self::from_args(&[ReportingProductKind::BalancesBetween], a),
		);
	}

	fn takes_args(_args: &Box<dyn ReportingStepArgs>) -> bool {
		true
	}

	fn from_args(
		product_kinds: &'static [ReportingProductKind; 1],
		args: Box<dyn ReportingStepArgs>,
	) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsExceptEarningsToEquityBalances {
			product_kinds,
			args,
		})
	}
}

impl Display for AllTransactionsExceptEarningsToEquityBalances {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for AllTransactionsExceptEarningsToEquityBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptEarningsToEquity",
			product_kinds: self.product_kinds,
			args: self.args.clone(),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		// AllTransactionsExceptEarningsToEquity always depends on CombineOrdinaryTransactions at least
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: self.product_kinds[0],
			args: self.args.clone(),
		}]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get all dependencies
		let step_dependencies = dependencies.dependencies_for_step(&self.id());

		// Identify the product_kind dependency most recently generated
		// TODO: Make this deterministic - parallel execution may cause the order to vary
		let product_kind = self.product_kinds[0];

		for (product_id, product) in products.map().iter().rev() {
			if step_dependencies.iter().any(|d| d.product == *product_id) {
				// Store the result
				let mut result = ReportingProducts::new();
				result.insert(
					ReportingProductId {
						name: self.id().name,
						kind: product_kind,
						args: self.args.clone(),
					},
					product.clone(),
				);
				return Ok(result);
			}
		}

		// No dependencies?! - this is likely a mistake
		panic!(
			"Requested {:?} but no available dependencies to provide it",
			self.product_kinds[0]
		);
	}
}

/// Target representing all transactions including charging current year and retained earnings to equity
///
/// In other words, this is [AllTransactionsExceptEarningsToEquity], [CurrentYearEarningsToEquity] and [RetainedEarningsToEquity].
///
/// Used as the basis for the balance sheet.
#[derive(Debug)]
pub struct AllTransactionsIncludingEarningsToEquity {
	pub args: DateArgs,
}

impl AllTransactionsIncludingEarningsToEquity {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"AllTransactionsIncludingEarningsToEquity",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsIncludingEarningsToEquity {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for AllTransactionsIncludingEarningsToEquity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for AllTransactionsIncludingEarningsToEquity {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsIncludingEarningsToEquity",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![
			// AllTransactionsIncludingEarningsToEquity requires AllTransactionsExceptEarningsToEquity
			ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			// AllTransactionsIncludingEarningsToEquity requires CurrentYearEarningsToEquity
			ReportingProductId {
				name: "CurrentYearEarningsToEquity",
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			// AllTransactionsIncludingEarningsToEquity requires RetainedEarningsToEquity
			ReportingProductId {
				name: "RetainedEarningsToEquity",
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
		]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get opening balances from AllTransactionsExceptEarningsToEquity
		let opening_balances = products
			.get_or_err(&ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap();

		// Get CurrentYearEarningsToEquity transactions
		let transactions_current = products
			.get_or_err(&ReportingProductId {
				name: "CurrentYearEarningsToEquity",
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<Transactions>()
			.unwrap();

		// Get RetainedEarningsToEquity transactions
		let transactions_retained = products
			.get_or_err(&ReportingProductId {
				name: "RetainedEarningsToEquity",
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<Transactions>()
			.unwrap();

		// Update balances
		let mut balances = BalancesAt {
			balances: opening_balances.balances.clone(),
		};
		update_balances_from_transactions(
			&mut balances.balances,
			transactions_current.transactions.iter(),
		);
		update_balances_from_transactions(
			&mut balances.balances,
			transactions_retained.transactions.iter(),
		);

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// Generates a balance sheet [DynamicReport]
#[derive(Debug)]
pub struct BalanceSheet {
	pub args: MultipleDateArgs,
}

impl BalanceSheet {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"BalanceSheet",
			&[ReportingProductKind::DynamicReport],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<MultipleDateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(BalanceSheet {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for BalanceSheet {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for BalanceSheet {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "BalanceSheet",
			product_kinds: &[ReportingProductKind::DynamicReport],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		let mut result = Vec::new();

		// BalanceSheet depends on AllTransactionsIncludingEarningsToEquity in each requested period
		for date_args in self.args.dates.iter() {
			result.push(ReportingProductId {
				name: "AllTransactionsIncludingEarningsToEquity",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(date_args.clone()),
			});
		}

		result
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances for each period
		let mut balances: Vec<&HashMap<String, QuantityInt>> = Vec::new();
		for date_args in self.args.dates.iter() {
			let product = products.get_or_err(&ReportingProductId {
				name: "AllTransactionsIncludingEarningsToEquity",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(date_args.clone()),
			})?;

			balances.push(&product.downcast_ref::<BalancesAt>().unwrap().balances);
		}

		// Get names of all balance sheet accounts
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		// Init report
		let mut report = DynamicReport::new(
			"Balance sheet".to_string(),
			self.args.dates.iter().map(|d| d.date.to_string()).collect(),
			Vec::new(),
		);

		// Add assets section
		let mut assets = Section {
			text: Some("Assets".to_string()),
			id: None,
			visible: true,
			entries: entries_for_kind("drcr.asset", false, &balances, &kinds_for_account),
		};
		let total_assets = assets.subtotal(&report);
		assets.entries.push(
			Row {
				text: "Total assets".to_string(),
				quantity: total_assets,
				id: Some("total_assets".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(assets.into());
		report.entries.push(DynamicReportEntry::Spacer);

		// Add liabilities section
		let mut liabilities = Section {
			text: Some("Liabilities".to_string()),
			id: None,
			visible: true,
			entries: entries_for_kind("drcr.liability", true, &balances, &kinds_for_account),
		};
		let total_liabilities = liabilities.subtotal(&report);
		liabilities.entries.push(
			Row {
				text: "Total liabilities".to_string(),
				quantity: total_liabilities,
				id: Some("total_liabilities".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(liabilities.into());
		report.entries.push(DynamicReportEntry::Spacer);

		// Add equity section
		let mut equity = Section {
			text: Some("Equity".to_string()),
			id: None,
			visible: true,
			entries: entries_for_kind("drcr.equity", true, &balances, &kinds_for_account),
		};
		let total_equity = equity.subtotal(&report);
		equity.entries.push(
			Row {
				text: "Total equity".to_string(),
				quantity: total_equity,
				id: Some("total_equity".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(equity.into());

		// Store the result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: "BalanceSheet",
				kind: ReportingProductKind::DynamicReport,
				args: Box::new(self.args.clone()),
			},
			Box::new(report),
		);
		Ok(result)
	}
}

/// Combines all steps producing ordinary transactions (returns transaction list)
///
/// By default, these are [DBTransactions] and [PostUnreconciledStatementLines].
#[derive(Debug)]
pub struct CombineOrdinaryTransactions {
	pub args: DateArgs,
}

impl CombineOrdinaryTransactions {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"CombineOrdinaryTransactions",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CombineOrdinaryTransactions {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for CombineOrdinaryTransactions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for CombineOrdinaryTransactions {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CombineOrdinaryTransactions",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![
			// CombineOrdinaryTransactions depends on DBTransactions
			ReportingProductId {
				name: "DBTransactions",
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			// CombineOrdinaryTransactions depends on PostUnreconciledStatementLines
			ReportingProductId {
				name: "PostUnreconciledStatementLines",
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
		]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		combine_transactions_of_all_dependencies(self.id(), dependencies, products).await
	}
}

/// Combines all steps producing ordinary transactions (returns balances)
///
/// By default, these are [DBBalances] and [PostUnreconciledStatementLines].
#[derive(Debug)]
pub struct CombineOrdinaryTransactionsBalances {
	pub args: DateArgs,
}

impl CombineOrdinaryTransactionsBalances {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"CombineOrdinaryTransactions",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CombineOrdinaryTransactionsBalances {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for CombineOrdinaryTransactionsBalances {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for CombineOrdinaryTransactionsBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CombineOrdinaryTransactions",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![
			// CombineOrdinaryTransactions depends on DBBalances
			ReportingProductId {
				name: "DBBalances",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			// CombineOrdinaryTransactions depends on PostUnreconciledStatementLines
			ReportingProductId {
				name: "PostUnreconciledStatementLines",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
		]
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Sum balances of all dependencies

		let mut balances = BalancesAt {
			balances: HashMap::new(),
		};

		for dependency in dependencies.dependencies_for_step(&self.id()) {
			let dependency_balances = &products
				.get_or_err(&dependency.product)?
				.downcast_ref::<BalancesAt>()
				.unwrap()
				.balances;
			for (account, balance) in dependency_balances.iter() {
				let running_balance = balances.balances.get(account).unwrap_or(&0) + balance;
				balances.balances.insert(account.clone(), running_balance);
			}
		}

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// Transfer year-to-date balances in income and expense accounts (as at the requested date) to the current year earnings equity account
#[derive(Debug)]
pub struct CurrentYearEarningsToEquity {
	pub args: DateArgs,
}

impl CurrentYearEarningsToEquity {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"CurrentYearEarningsToEquity",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CurrentYearEarningsToEquity {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for CurrentYearEarningsToEquity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for CurrentYearEarningsToEquity {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CurrentYearEarningsToEquity",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// CurrentYearEarningsToEquity depends on AllTransactionsExceptEarningsToEquity
		vec![ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::BalancesBetween,
			args: Box::new(DateStartDateEndArgs {
				date_start: sofy_from_eofy(get_eofy(&self.args.date, &context.eofy_date)),
				date_end: self.args.date,
			}),
		}]
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances for this financial year
		let balances = products
			.get_or_err(&ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(DateStartDateEndArgs {
					date_start: sofy_from_eofy(get_eofy(&self.args.date, &context.eofy_date)),
					date_end: self.args.date,
				}),
			})?
			.downcast_ref::<BalancesBetween>()
			.unwrap();

		// Get income and expense accounts
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		// Transfer income and expense balances to current year earnings
		let mut transactions = Transactions {
			transactions: Vec::new(),
		};

		for (account, balance) in balances.balances.iter() {
			if let Some(kinds) = kinds_for_account.get(account) {
				if kinds
					.iter()
					.any(|k| k == "drcr.income" || k == "drcr.expense")
				{
					transactions.transactions.push(TransactionWithPostings {
						transaction: Transaction {
							id: None,
							dt: self.args.date.and_hms_opt(0, 0, 0).unwrap(),
							description: "Current year earnings".to_string(),
						},
						postings: vec![
							Posting {
								id: None,
								transaction_id: None,
								description: None,
								account: account.clone(),
								quantity: -balance,
								commodity: context.reporting_commodity.clone(),
								quantity_ascost: None,
							},
							Posting {
								id: None,
								transaction_id: None,
								description: None,
								account: crate::CURRENT_YEAR_EARNINGS.to_string(),
								quantity: *balance,
								commodity: context.reporting_commodity.clone(),
								quantity_ascost: None,
							},
						],
					})
				}
			}
		}

		// Store product
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			Box::new(transactions),
		);
		Ok(result)
	}
}

/// Look up account balances from the database
#[derive(Debug)]
pub struct DBBalances {
	pub args: DateArgs,
}

impl DBBalances {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"DBBalances",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(DBBalances {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for DBBalances {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for DBBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "DBBalances",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		// Get balances from DB
		let balances = BalancesAt {
			balances: context.db_connection.get_balances(self.args.date).await,
		};

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);
		Ok(result)
	}
}

/// Look up transactions from the database
#[derive(Debug)]
pub struct DBTransactions {}

impl DBTransactions {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"DBTransactions",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<VoidArgs>()
	}

	fn from_args(_args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(DBTransactions {})
	}
}

impl Display for DBTransactions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for DBTransactions {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "DBTransactions",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(VoidArgs {}),
		}
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		// Get transactions from DB
		let transactions = Transactions {
			transactions: context.db_connection.get_transactions().await,
		};

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			Box::new(transactions),
		);
		Ok(result)
	}
}

/// Generates an income statement [DynamicReport]
#[derive(Debug)]
pub struct IncomeStatement {
	pub args: MultipleDateStartDateEndArgs,
}

impl IncomeStatement {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"IncomeStatement",
			&[ReportingProductKind::DynamicReport],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<MultipleDateStartDateEndArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(IncomeStatement {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for IncomeStatement {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for IncomeStatement {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "IncomeStatement",
			product_kinds: &[ReportingProductKind::DynamicReport],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		let mut result = Vec::new();

		// IncomeStatement depends on AllTransactionsExceptEarningsToEquity in each requested period
		for date_args in self.args.dates.iter() {
			result.push(ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(date_args.clone()),
			});
		}

		result
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances for each period
		let mut balances: Vec<&HashMap<String, QuantityInt>> = Vec::new();
		for date_args in self.args.dates.iter() {
			let product = products.get_or_err(&ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(date_args.clone()),
			})?;

			balances.push(&product.downcast_ref::<BalancesBetween>().unwrap().balances);
		}

		// Get names of all income statement accounts
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		// Init report
		let mut report = DynamicReport::new(
			"Income statement".to_string(),
			self.args
				.dates
				.iter()
				.map(|d| d.date_end.to_string())
				.collect(),
			Vec::new(),
		);

		// Add income section
		let mut income = Section {
			text: Some("Income".to_string()),
			id: None,
			visible: true,
			entries: entries_for_kind("drcr.income", true, &balances, &kinds_for_account),
		};
		let total_income = income.subtotal(&report);
		income.entries.push(
			Row {
				text: "Total income".to_string(),
				quantity: total_income.clone(),
				id: Some("total_income".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(income.into());
		report.entries.push(DynamicReportEntry::Spacer);

		// Add expenses section
		let mut expenses = Section {
			text: Some("Expenses".to_string()),
			id: None,
			visible: true,
			entries: entries_for_kind("drcr.expense", false, &balances, &kinds_for_account),
		};
		let total_expenses = expenses.subtotal(&report);
		expenses.entries.push(
			Row {
				text: "Total expenses".to_string(),
				quantity: total_expenses.clone(),
				id: Some("total_expenses".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(expenses.into());
		report.entries.push(DynamicReportEntry::Spacer);

		// Add net surplus (deficit) row
		let net_surplus = total_income
			.into_iter()
			.zip(total_expenses.into_iter())
			.map(|(i, e)| i - e)
			.collect();
		report.entries.push(
			Row {
				text: "Net surplus (deficit)".to_string(),
				quantity: net_surplus,
				id: Some("net_surplus".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);

		// Store the result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: "IncomeStatement",
				kind: ReportingProductKind::DynamicReport,
				args: Box::new(self.args.clone()),
			},
			Box::new(report),
		);
		Ok(result)
	}
}

/// Generate transactions for unreconciled statement lines
#[derive(Debug)]
pub struct PostUnreconciledStatementLines {}

impl PostUnreconciledStatementLines {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"PostUnreconciledStatementLines",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<VoidArgs>()
	}

	fn from_args(_args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(PostUnreconciledStatementLines {})
	}
}

impl Display for PostUnreconciledStatementLines {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for PostUnreconciledStatementLines {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "PostUnreconciledStatementLines",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(VoidArgs {}),
		}
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		_products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let unreconciled_statement_lines = context
			.db_connection
			.get_unreconciled_statement_lines()
			.await;

		// Post unreconciled statement lines
		let mut transactions = Transactions {
			transactions: Vec::new(),
		};

		for line in unreconciled_statement_lines {
			let unclassified_account = if line.quantity >= 0 {
				"Unclassified Statement Line Debits"
			} else {
				"Unclassified Statement Line Credits"
			};
			transactions.transactions.push(TransactionWithPostings {
				transaction: Transaction {
					id: None,
					dt: line.dt,
					description: line.description.clone(),
				},
				postings: vec![
					Posting {
						id: None,
						transaction_id: None,
						description: None,
						account: line.source_account.clone(),
						quantity: line.quantity,
						commodity: line.commodity.clone(),
						quantity_ascost: None,
					},
					Posting {
						id: None,
						transaction_id: None,
						description: None,
						account: unclassified_account.to_string(),
						quantity: -line.quantity,
						commodity: line.commodity.clone(),
						quantity_ascost: None,
					},
				],
			});
		}

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			Box::new(transactions),
		);
		Ok(result)
	}
}

/// Transfer historical balances in income and expense accounts to the retained earnings equity account
#[derive(Debug)]
pub struct RetainedEarningsToEquity {
	pub args: DateArgs,
}

impl RetainedEarningsToEquity {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"RetainedEarningsToEquity",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(RetainedEarningsToEquity {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for RetainedEarningsToEquity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for RetainedEarningsToEquity {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "RetainedEarningsToEquity",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		let eofy_date = get_eofy(&self.args.date, &context.eofy_date);
		let last_eofy_date = eofy_date.with_year(eofy_date.year() - 1).unwrap();

		// RetainedEarningsToEquity depends on CombineOrdinaryTransactions for last financial year
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::BalancesAt,
			args: Box::new(DateArgs {
				date: last_eofy_date,
			}),
		}]
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;
		let eofy_date = get_eofy(&self.args.date, &context.eofy_date);
		let last_eofy_date = eofy_date.with_year(eofy_date.year() - 1).unwrap();

		// Get balances at end of last financial year
		let balances_last_eofy = products
			.get_or_err(&ReportingProductId {
				name: "CombineOrdinaryTransactions",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: last_eofy_date.clone(),
				}),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap();

		// Get income and expense accounts
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		// Transfer income and expense balances to retained earnings
		let mut transactions = Transactions {
			transactions: Vec::new(),
		};

		for (account, balance) in balances_last_eofy.balances.iter() {
			if let Some(kinds) = kinds_for_account.get(account) {
				if kinds
					.iter()
					.any(|k| k == "drcr.income" || k == "drcr.expense")
				{
					transactions.transactions.push(TransactionWithPostings {
						transaction: Transaction {
							id: None,
							dt: last_eofy_date.and_hms_opt(0, 0, 0).unwrap(),
							description: "Retained earnings".to_string(),
						},
						postings: vec![
							Posting {
								id: None,
								transaction_id: None,
								description: None,
								account: account.clone(),
								quantity: -balance,
								commodity: context.reporting_commodity.clone(),
								quantity_ascost: None,
							},
							Posting {
								id: None,
								transaction_id: None,
								description: None,
								account: crate::RETAINED_EARNINGS.to_string(),
								quantity: *balance,
								commodity: context.reporting_commodity.clone(),
								quantity_ascost: None,
							},
						],
					})
				}
			}
		}

		// Store product
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			Box::new(transactions),
		);
		Ok(result)
	}
}

/// Generates a trial balance [DynamicReport]
#[derive(Debug)]
pub struct TrialBalance {
	pub args: DateArgs,
}

impl TrialBalance {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"TrialBalance",
			&[ReportingProductKind::DynamicReport],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(TrialBalance {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for TrialBalance {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

#[async_trait]
impl ReportingStep for TrialBalance {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "TrialBalance",
			product_kinds: &[ReportingProductKind::DynamicReport],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		let mut result = Vec::new();

		// TrialBalance depends on AllTransactionsExceptEarningsToEquity at the requested date
		result.push(ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::BalancesAt,
			args: Box::new(self.args.clone()),
		});

		result
	}

	async fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances for each period
		let balances = &products
			.get_or_err(&ReportingProductId {
				name: "AllTransactionsExceptEarningsToEquity",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap()
			.balances;

		// Get sorted list of accounts
		let mut accounts = balances.keys().collect::<Vec<_>>();
		accounts.sort();

		// Init report
		let mut report = DynamicReport {
			title: "Trial balance".to_string(),
			columns: vec!["Dr".to_string(), "Cr".to_string()],
			entries: Vec::new(),
		};

		// Add entry for each account
		let mut section = Section {
			text: None,
			id: None,
			visible: true,
			entries: Vec::new(),
		};
		for account in accounts {
			section.entries.push(
				Row {
					text: account.clone(),
					quantity: vec![
						// Dr cell
						if balances[account] >= 0 {
							balances[account]
						} else {
							0
						},
						// Cr cell
						if balances[account] < 0 {
							-balances[account]
						} else {
							0
						},
					],
					id: None,
					visible: true,
					link: Some(format!("/transactions/{}", account)),
					heading: false,
					bordered: false,
				}
				.into(),
			);
		}
		let totals_row = section.subtotal(&report);
		report.entries.push(section.into());

		// Add total row
		report.entries.push(
			Row {
				text: "Totals".to_string(),
				quantity: totals_row,
				id: Some("totals".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);

		// Store result
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: "TrialBalance",
				kind: ReportingProductKind::DynamicReport,
				args: Box::new(self.args.clone()),
			},
			Box::new(report),
		);
		Ok(result)
	}
}

/// Combines the transactions of all dependencies and returns [Transactions] as [ReportingProducts] for the given step
///
/// Used to implement [CombineOrdinaryTransactions] and [AllTransactionsExceptEarningsToEquity].
async fn combine_transactions_of_all_dependencies(
	step_id: ReportingStepId,
	dependencies: &ReportingGraphDependencies,
	products: &RwLock<ReportingProducts>,
) -> Result<ReportingProducts, ReportingExecutionError> {
	let products = products.read().await;

	// Combine transactions of all dependencies

	let mut transactions = Transactions {
		transactions: Vec::new(),
	};

	for dependency in dependencies.dependencies_for_step(&step_id) {
		let dependency_transactions = &products
			.get_or_err(&dependency.product)?
			.downcast_ref::<Transactions>()
			.unwrap()
			.transactions;

		for transaction in dependency_transactions.iter() {
			transactions.transactions.push(transaction.clone());
		}
	}

	// Store result
	let mut result = ReportingProducts::new();
	result.insert(
		ReportingProductId {
			name: step_id.name,
			kind: ReportingProductKind::Transactions,
			args: step_id.args,
		},
		Box::new(transactions),
	);

	Ok(result)
}
