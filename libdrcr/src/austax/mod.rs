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

//! Implements Australian individual income tax calculations

// TODO: Ideally this would be separated into its own plugin

use std::collections::HashMap;
use std::fmt::Display;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::account_config::kinds_for_account;
use crate::model::transaction::{Posting, Transaction, TransactionWithPostings};
use crate::reporting::calculator::ReportingGraphDependencies;
use crate::reporting::dynamic_report::{
	entries_for_kind, DynamicReport, DynamicReportEntry, Row, Section,
};
use crate::reporting::executor::ReportingExecutionError;
use crate::reporting::steps::AllTransactionsExceptEarningsToEquityBalances;
use crate::reporting::types::{
	BalancesBetween, DateStartDateEndArgs, ReportingContext, ReportingProductId,
	ReportingProductKind, ReportingProducts, ReportingStep, ReportingStepArgs, ReportingStepId,
	Transactions, VoidArgs,
};
use crate::util::sofy_from_eofy;
use crate::{QuantityInt, INCOME_TAX, INCOME_TAX_CONTROL};

// Constants and tax calculations
#[rustfmt::skip]
const INCOME_TYPES: &[(&str, &str, &str)] = &[
	("income1", "Salary or wages", "1"),
	("income2", "Allowances, earnings, tips, director's fees etc.", "2"),
	("income3", "Employer lump sum payments", "3"),
	("income4", "Employment termination payments", "4"),
	("income5", "Australian Government allowances and payments", "5"),
	("income6", "Australian Government pensions and allowances", "6"),
	("income7", "Australian annuities and superannuation income streams", "7"),
	("income8", "Australian superannuation lump sum payments", "8"),
	("income9", "Attributed personal services income", "9"),
	("income10", "Gross interest", "10"),
	("income11", "Dividends", "11"),
	("income12", "Employee share schemes", "12"),
	("income13", "Partnerships and trusts", "13"),
	("income14", "Personal services income", "14"),
	("income15", "Net income or loss from business", "15"),
	("income16", "Deferred non-commercial business losses", "16"),
	("income17", "Net farm management deposits or repayments", "17"),
	("income18", "Capital gains", "18"),
	("income19", "Foreign entities", "19"),
	("income20", "Foreign source income and foreign assets or property", "20"),
	("income21", "Rent", "21"),
	("income22", "Bonuses from life insurance companies and friendly societies", "22"),
	("income23", "Forestry managed investment scheme income", "23"),
	("income24", "Other income", "24"),
];

#[rustfmt::skip]
const DEDUCTION_TYPES: &[(&str, &str, &str)] = &[
	("d1", "Work-related car expenses", "D1"),
	("d2", "Work-related travel expenses", "D2"),
	("d3", "Work-related clothing, laundry and dry cleaning expenses", "D3"),
	("d4", "Work-related self-education expenses", "D4"),
	("d5", "Other work-related expenses", "D5"),
	("d6", "Low value pool deduction", "D6"),
	("d7", "Interest deductions", "D7"),
	("d8", "Dividend deductions", "D8"),
	("d9", "Gifts or donations", "D9"),
	("d10", "Cost of managing tax affairs", "D10"),
	("d11", "Deductible amount of undeducted purchase price of a foreign pension or annuity", "D11"),
	("d12", "Personal superannuation contributions", "D12"),
	("d13", "Deduction for project pool", "D13"),
	("d14", "Forestry managed investment scheme deduction", "D14"),
	("d15", "Other deductions", "D15"),
];

fn get_grossedup_rfb(taxable_value: QuantityInt) -> QuantityInt {
	// FIXME: May vary from year to year
	((taxable_value as f64) * 2.0802) as QuantityInt
}

fn get_base_income_tax(net_taxable: QuantityInt) -> QuantityInt {
	// FIXME: May vary from year to year
	if net_taxable <= 18200_00 {
		0
	} else if net_taxable <= 45000_00 {
		(0.16 * (net_taxable - 18200_00) as f64) as QuantityInt
	} else if net_taxable <= 135000_00 {
		4288_00 + (0.30 * (net_taxable - 45000_00) as f64) as QuantityInt
	} else if net_taxable <= 190000_00 {
		31288_00 + (0.37 * (net_taxable - 135000_00) as f64) as QuantityInt
	} else {
		51638_00 + (0.45 * (net_taxable - 190000_00) as f64) as QuantityInt
	}
}

// fn get_medicare_levy(net_taxable: QuantityInt) -> QuantityInt {
// 	todo!()
// }

// fn get_medicare_levy_surcharge(
// 	net_taxable: QuantityInt,
// 	rfb_grossedup: QuantityInt,
// ) -> QuantityInt {
// 	todo!()
// }

/// Call [ReportingContext::register_lookup_fn] for all steps provided by this module
pub fn register_lookup_fns(context: &mut ReportingContext) {
	CalculateIncomeTax::register_lookup_fn(context);
}

/// Calculates income tax
///
/// [Transactions] product represents income tax charge for the year.
/// [DynamicReport] product represents the tax summary report.
#[derive(Debug)]
pub struct CalculateIncomeTax {}

impl CalculateIncomeTax {
	fn register_lookup_fn(context: &mut ReportingContext) {
		context.register_lookup_fn(
			"CalculateIncomeTax".to_string(),
			vec![ReportingProductKind::Transactions],
			Self::takes_args,
			Self::from_args,
		);
	}

	fn takes_args(args: &Box<dyn ReportingStepArgs>) -> bool {
		args.is::<VoidArgs>()
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

#[async_trait]
impl ReportingStep for CalculateIncomeTax {
	fn id(&self) -> ReportingStepId {
		ReportingStepId {
			name: "CalculateIncomeTax".to_string(),
			product_kinds: vec![
				ReportingProductKind::DynamicReport,
				ReportingProductKind::Transactions,
			],
			args: Box::new(VoidArgs {}),
		}
	}

	fn requires(&self, context: &ReportingContext) -> Vec<ReportingProductId> {
		// CalculateIncomeTax depends on CombineOrdinaryTransactions
		vec![ReportingProductId {
			name: "CombineOrdinaryTransactions".to_string(),
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
			if let Some(other) =
				other.downcast_ref::<AllTransactionsExceptEarningsToEquityBalances>()
			{
				// AllTransactionsExceptEarningsToEquity depends on CalculateIncomeTax
				dependencies.add_dependency(
					other.id(),
					ReportingProductId {
						name: self.id().name,
						kind: other.product_kind,
						args: if other.product_kind == ReportingProductKind::Transactions {
							Box::new(VoidArgs {})
						} else {
							other.id().args
						},
					},
				);
			}
		}
	}

	async fn execute(
		&self,
		context: &ReportingContext,
		_steps: &Vec<Box<dyn ReportingStep>>,
		_dependencies: &ReportingGraphDependencies,
		products: &RwLock<ReportingProducts>,
	) -> Result<ReportingProducts, ReportingExecutionError> {
		let products = products.read().await;

		// Get balances for current year
		let balances = &products
			.get_or_err(&ReportingProductId {
				name: "CombineOrdinaryTransactions".to_string(),
				kind: ReportingProductKind::BalancesBetween,
				args: Box::new(DateStartDateEndArgs {
					date_start: sofy_from_eofy(context.eofy_date),
					date_end: context.eofy_date.clone(),
				}),
			})?
			.downcast_ref::<BalancesBetween>()
			.unwrap()
			.balances;

		// Get taxable income and deduction accounts
		let kinds_for_account =
			kinds_for_account(context.db_connection.get_account_configurations().await);

		// Generate tax summary report
		let mut report = DynamicReport {
			title: "Tax summary".to_string(),
			columns: vec!["$".to_string()],
			entries: Vec::new(),
		};

		// Add income entries
		let mut total_income: QuantityInt = 0;

		for (code, label, number) in INCOME_TYPES {
			let entries;
			if *code == "income1" {
				// Special case for salary or wages - round each separately
				entries = entries_for_kind_floor(
					&format!("austax.{}", code),
					true,
					&vec![balances],
					&kinds_for_account,
					100,
				);
			} else {
				entries = entries_for_kind(
					&format!("austax.{}", code),
					true,
					&vec![balances],
					&kinds_for_account,
				);
			}

			if entries.is_empty() {
				continue;
			}

			let mut section = Section {
				text: Some(format!("{} ({})", label, number)),
				id: None,
				visible: true,
				entries,
			};

			// Add subtotal row
			let subtotal = floor_quantity(section.subtotal(&report), 100);
			total_income += subtotal[0];

			section.entries.push(
				Row {
					text: format!("Total item {}", number),
					quantity: subtotal,
					id: Some(format!("total_{}", code)),
					visible: true,
					link: None,
					heading: true,
					bordered: true,
				}
				.into(),
			);
			report.entries.push(section.into());
			report.entries.push(DynamicReportEntry::Spacer);
		}

		// Total assessable income
		report.entries.push(
			Row {
				text: "Total assessable income".to_string(),
				quantity: vec![total_income],
				id: Some("total_income".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(DynamicReportEntry::Spacer);

		// Add deduction entries
		let mut total_deductions: QuantityInt = 0;

		for (code, label, number) in DEDUCTION_TYPES {
			let entries = entries_for_kind(
				&format!("austax.{}", code),
				false,
				&vec![balances],
				&kinds_for_account,
			);

			if entries.is_empty() {
				continue;
			}

			let mut section = Section {
				text: Some(format!("{} ({})", label, number)),
				id: None,
				visible: true,
				entries,
			};

			// Add subtotal row
			let subtotal = floor_quantity(section.subtotal(&report), 100);
			total_deductions += subtotal[0];

			section.entries.push(
				Row {
					text: format!("Total item {}", number),
					quantity: subtotal,
					id: Some(format!("total_{}", code)),
					visible: true,
					link: None,
					heading: true,
					bordered: true,
				}
				.into(),
			);
			report.entries.push(section.into());
			report.entries.push(DynamicReportEntry::Spacer);
		}

		// Total deductions
		report.entries.push(
			Row {
				text: "Total deductions".to_string(),
				quantity: vec![total_deductions],
				id: Some("total_deductions".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(DynamicReportEntry::Spacer);

		// Net taxable income
		let net_taxable = total_income - total_deductions;
		report.entries.push(
			Row {
				text: "Net taxable income".to_string(),
				quantity: vec![net_taxable],
				id: Some("net_taxable".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);
		report.entries.push(DynamicReportEntry::Spacer);

		// Precompute RFB amount as this is required for MLS
		let rfb_taxable = balances
			.iter()
			.filter(|(acc, _)| {
				kinds_for_account
					.get(*acc)
					.map(|kinds| kinds.iter().any(|k| k == "austax.rfb"))
					.unwrap_or(false)
			})
			.map(|(_, bal)| *bal)
			.sum();
		let _rfb_grossedup = get_grossedup_rfb(rfb_taxable);

		// Base income tax row
		let tax_base = get_base_income_tax(net_taxable);
		report.entries.push(
			Row {
				text: "Base income tax".to_string(),
				quantity: vec![tax_base],
				id: Some("tax_base".to_string()),
				visible: true,
				link: None,
				heading: false,
				bordered: false,
			}
			.into(),
		);

		// Total income tax row
		let tax_total = tax_base;
		report.entries.push(
			Row {
				text: "Total income tax".to_string(),
				quantity: vec![tax_total],
				id: Some("tax_total".to_string()),
				visible: true,
				link: None,
				heading: true,
				bordered: true,
			}
			.into(),
		);

		// Generate income tax transaction
		let transactions = Transactions {
			transactions: vec![TransactionWithPostings {
				transaction: Transaction {
					id: None,
					dt: context
						.db_connection
						.metadata()
						.eofy_date
						.and_hms_opt(0, 0, 0)
						.unwrap(),
					description: "Estimated income tax".to_string(),
				},
				postings: vec![
					Posting {
						id: None,
						transaction_id: None,
						description: None,
						account: INCOME_TAX.to_string(),
						quantity: tax_total,
						commodity: context.db_connection.metadata().reporting_commodity.clone(),
						quantity_ascost: Some(tax_total),
					},
					Posting {
						id: None,
						transaction_id: None,
						description: None,
						account: INCOME_TAX_CONTROL.to_string(),
						quantity: -tax_total,
						commodity: context.db_connection.metadata().reporting_commodity.clone(),
						quantity_ascost: Some(tax_total),
					},
				],
			}],
		};

		// Store products
		let mut result = ReportingProducts::new();
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			Box::new(transactions),
		);
		result.insert(
			ReportingProductId {
				name: self.id().name,
				kind: ReportingProductKind::DynamicReport,
				args: Box::new(VoidArgs {}),
			},
			Box::new(report),
		);
		Ok(result)
	}
}

/// Call [entries_for_kind] then round results down to next multiple of `floor`
fn entries_for_kind_floor(
	kind: &str,
	invert: bool,
	balances: &Vec<&HashMap<String, QuantityInt>>,
	kinds_for_account: &HashMap<String, Vec<String>>,
	floor: QuantityInt,
) -> Vec<DynamicReportEntry> {
	let mut entries_for_kind = entries_for_kind(kind, invert, balances, kinds_for_account);
	entries_for_kind.iter_mut().for_each(|e| match e {
		DynamicReportEntry::Row(row) => row
			.quantity
			.iter_mut()
			.for_each(|v| *v = (*v / floor) * floor),
		_ => unreachable!(),
	});
	entries_for_kind
}

fn floor_quantity(mut quantity: Vec<QuantityInt>, floor: QuantityInt) -> Vec<QuantityInt> {
	quantity.iter_mut().for_each(|v| *v = (*v / floor) * floor);
	quantity
}
