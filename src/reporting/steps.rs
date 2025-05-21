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

use chrono::Datelike;

use crate::reporting::types::{BalancesAt, DateStartDateEndArgs, ReportingProductId, Transactions};
use crate::transaction::update_balances_from_transactions;
use crate::util::sofy_from_eofy;

use super::calculator::ReportingGraphDependencies;
use super::executor::ReportingExecutionError;
use super::types::{
	BalancesBetween, DateArgs, ReportingContext, ReportingProduct, ReportingProductKind,
	ReportingProducts, ReportingStep, ReportingStepArgs, ReportingStepId, VoidArgs,
};

/// Call [ReportingContext::register_lookup_fn] for all steps provided by this module
pub fn register_lookup_fns(context: &mut ReportingContext) {
	AllTransactionsExceptRetainedEarnings::register_lookup_fn(context);
	AllTransactionsIncludingRetainedEarnings::register_lookup_fn(context);
	CalculateIncomeTax::register_lookup_fn(context);
	CombineOrdinaryTransactions::register_lookup_fn(context);
	DBBalances::register_lookup_fn(context);
	PostUnreconciledStatementLines::register_lookup_fn(context);
	RetainedEarningsToEquity::register_lookup_fn(context);
}

/// Target representing all transactions except charging retained earnings to equity
///
/// By default, this is [CombineOrdinaryTransactions] and, if requested, [CalculateIncomeTax].
///
/// Used as the basis for the income statement.
#[derive(Debug)]
pub struct AllTransactionsExceptRetainedEarnings {
	pub product_kinds: &'static [ReportingProductKind; 1], // Must have single member - represented as static array for compatibility with ReportingStepId
	pub args: Box<dyn ReportingStepArgs>,
}

impl AllTransactionsExceptRetainedEarnings {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"AllTransactionsExceptRetainedEarnings",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			|a| Self::from_args(&[ReportingProductKind::BalancesAt], a),
		);

		context.register_lookup_fn(
			"AllTransactionsExceptRetainedEarnings",
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
		Box::new(AllTransactionsExceptRetainedEarnings {
			product_kinds,
			args,
		})
	}
}

impl Display for AllTransactionsExceptRetainedEarnings {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for AllTransactionsExceptRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsExceptRetainedEarnings",
			product_kinds: self.product_kinds,
			args: self.args.clone(),
		}
	}

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		// Get all dependencies
		let step_dependencies = dependencies.dependencies_for_step(&self.id());

		// Identify the product_kind dependency most recently generated
		let product_kind = self.product_kinds[0];

		for (product_id, product) in products.map().iter().rev() {
			if step_dependencies.iter().any(|d| d.product == *product_id) {
				// Store the result
				products.insert(
					ReportingProductId {
						name: self.id().name,
						kind: product_kind,
						args: self.args.clone(),
					},
					product.clone(),
				);

				return Ok(());
			}
		}

		// No dependencies?! - store empty result
		let product: Box<dyn ReportingProduct> = match self.product_kinds[0] {
			ReportingProductKind::Transactions => Box::new(Transactions {
				transactions: Vec::new(),
			}),
			ReportingProductKind::BalancesAt => Box::new(BalancesAt {
				balances: HashMap::new(),
			}),
			ReportingProductKind::BalancesBetween => Box::new(BalancesBetween {
				balances: HashMap::new(),
			}),
			ReportingProductKind::Generic => panic!("Requested AllTransactionsExceptRetainedEarnings.Generic but no available dependencies to provide it"),
		};

		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: product_kind,
				args: self.args.clone(),
			},
			product,
		);

		Ok(())
	}
}

/// Target representing all transactions including charging retained earnings to equity
///
/// In other words, this is [AllTransactionsExceptRetainedEarnings] and [RetainedEarningsToEquity].
///
/// Used as the basis for the balance sheet.
#[derive(Debug)]
pub struct AllTransactionsIncludingRetainedEarnings {
	pub args: DateArgs,
}

impl AllTransactionsIncludingRetainedEarnings {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"AllTransactionsIncludingRetainedEarnings",
			&[ReportingProductKind::BalancesAt],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(AllTransactionsIncludingRetainedEarnings {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for AllTransactionsIncludingRetainedEarnings {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for AllTransactionsIncludingRetainedEarnings {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "AllTransactionsIncludingRetainedEarnings",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, _context: &ReportingContext) -> Vec<ReportingProductId> {
		vec![
			// AllTransactionsIncludingRetainedEarnings requires AllTransactionsExceptRetainedEarnings
			ReportingProductId {
				name: "AllTransactionsExceptRetainedEarnings",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			// AllTransactionsIncludingRetainedEarnings requires RetainedEarningsToEquity
			ReportingProductId {
				name: "RetainedEarningsToEquity",
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
		]
	}

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		// Get opening balances from AllTransactionsExceptRetainedEarnings
		let opening_balances = products
			.get_or_err(&ReportingProductId {
				name: "AllTransactionsExceptRetainedEarnings",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			})?
			.downcast_ref::<BalancesAt>()
			.unwrap();

		// Get RetainedEarningsToEquity transactions
		let transactions = products
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
		update_balances_from_transactions(&mut balances.balances, transactions.transactions.iter());

		// Store result
		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);

		Ok(())
	}
}

/// Calculates income tax
#[derive(Debug)]
pub struct CalculateIncomeTax {}

impl CalculateIncomeTax {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"CalculateIncomeTax",
			&[ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(_args: &Box<dyn ReportingStepArgs>) -> bool {
		true
	}

	fn from_args(_args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(CalculateIncomeTax {})
	}
}

impl Display for CalculateIncomeTax {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for CalculateIncomeTax {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CalculateIncomeTax",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(VoidArgs {}),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// CalculateIncomeTax depends on CombineOrdinaryTransactions
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::BalancesBetween,
			args: Box::new(DateStartDateEndArgs {
				date_start: sofy_from_eofy(context.eofy_date),
				date_end: context.eofy_date.clone(),
			}),
		}]
	}

	fn after_init_graph(
		&self,
		steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &mut ReportingGraphDependencies,
		_context: &ReportingContext,
	) {
		for other in steps {
			if let Some(other) = other.downcast_ref::<AllTransactionsExceptRetainedEarnings>() {
				// AllTransactionsExceptRetainedEarnings depends on CalculateIncomeTax
				dependencies.add_dependency(
					other.id(),
					ReportingProductId {
						name: self.id().name,
						kind: other.product_kinds[0],
						args: other.id().args,
					},
				);
			}
		}
	}

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		eprintln!("Stub: CalculateIncomeTax.execute");

		let transactions = Transactions {
			transactions: Vec::new(),
		};

		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			Box::new(transactions),
		);

		Ok(())
	}
}

/// Combines all steps producing ordinary transactions
///
/// By default, these are [DBBalances] and [PostUnreconciledStatementLines]
#[derive(Debug)]
pub struct CombineOrdinaryTransactions {
	pub args: DateArgs,
}

impl CombineOrdinaryTransactions {
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

impl ReportingStep for CombineOrdinaryTransactions {
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

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
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
		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);

		Ok(())
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

impl ReportingStep for DBBalances {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "DBBalances",
			product_kinds: &[ReportingProductKind::BalancesAt],
			args: Box::new(self.args.clone()),
		}
	}

	fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		// Get balances from DB
		let balances = BalancesAt {
			balances: context.db_connection.get_balances(self.args.date),
		};

		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(self.args.clone()),
			},
			Box::new(balances),
		);

		Ok(())
	}
}

/// Generate transactions for unreconciled statement lines
#[derive(Debug)]
pub struct PostUnreconciledStatementLines {
	pub args: DateArgs,
}

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
		args.is::<DateArgs>()
	}

	fn from_args(args: Box<dyn ReportingStepArgs>) -> Box<dyn ReportingStep> {
		Box::new(PostUnreconciledStatementLines {
			args: *args.downcast().unwrap(),
		})
	}
}

impl Display for PostUnreconciledStatementLines {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.id()))
	}
}

impl ReportingStep for PostUnreconciledStatementLines {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "PostUnreconciledStatementLines",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		eprintln!("Stub: PostUnreconciledStatementLines.execute");

		let transactions = Transactions {
			transactions: Vec::new(),
		};

		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			Box::new(transactions),
		);

		Ok(())
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

impl ReportingStep for RetainedEarningsToEquity {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "RetainedEarningsToEquity",
			product_kinds: &[ReportingProductKind::Transactions],
			args: Box::new(self.args.clone()),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// RetainedEarningsToEquity depends on CombineOrdinaryTransactions for last financial year
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::BalancesAt,
			args: Box::new(DateArgs {
				date: context
					.eofy_date
					.with_year(context.eofy_date.year() - 1)
					.unwrap(),
			}),
		}]
	}

	fn execute(
		&self,
		_context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &mut ReportingProducts,
	) -> Result<(), ReportingExecutionError> {
		eprintln!("Stub: RetainedEarningsToEquity.execute");

		let transactions = Transactions {
			transactions: Vec::new(),
		};

		products.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(self.args.clone()),
			},
			Box::new(transactions),
		);

		Ok(())
	}
}
