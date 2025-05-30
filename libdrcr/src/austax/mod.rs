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
	entries_for_kind, CalculatableDynamicReport, CalculatableDynamicReportEntry,
	CalculatableSection, CalculatedRow, DynamicReport, LiteralRow,
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
			"CalculateIncomeTax",
			&[ReportingProductKind::Transactions],
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
			name: "CalculateIncomeTax",
			product_kinds: &[
				ReportingProductKind::DynamicReport,
				ReportingProductKind::Transactions,
			],
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
			if let Some(other) =
				other.downcast_ref::<AllTransactionsExceptEarningsToEquityBalances>()
			{
				// AllTransactionsExceptEarningsToEquity depends on CalculateIncomeTax
				dependencies.add_dependency(
					other.id(),
					ReportingProductId {
						name: self.id().name,
						kind: other.product_kinds[0],
						args: if other.product_kinds[0] == ReportingProductKind::Transactions {
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
				name: "CombineOrdinaryTransactions",
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

		// Pre-compute taxable value of reportable fringe benefits (required for MLS)
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

		// Generate tax summary report
		let report = CalculatableDynamicReport::new(
			"Tax summary".to_string(),
			vec!["$".to_string()],
			vec![
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Salary or wages (1)".to_string(),
					Some("income1".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind_floor(
							"austax.income1",
							true,
							&vec![balances],
							&kinds_for_account,
							100,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 1".to_string(),
									quantity: report.subtotal_for_id("income1").unwrap(),
									id: Some("total_income1".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						// Add spacer as child of the Section so it is hidden if the Section is hidden
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Australian Government allowances and payments (5)".to_string(),
					Some("income5".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.income5",
							true,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 5".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("income5").unwrap(),
										100,
									),
									id: Some("total_income5".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Gross interest (10)".to_string(),
					Some("income10".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.income10",
							true,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 10".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("income10").unwrap(),
										100,
									),
									id: Some("total_income10".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Partnerships and trusts (13)".to_string(),
					Some("income13".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.income13",
							true,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 13".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("income13").unwrap(),
										100,
									),
									id: Some("total_income13".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Foreign source income and foreign assets or property (20)".to_string(),
					Some("income20".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.income20",
							true,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 20".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("income20").unwrap(),
										100,
									),
									id: Some("total_income20".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Other income (24)".to_string(),
					Some("income24".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.income24",
							true,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item 24".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("income24").unwrap(),
										100,
									),
									id: Some("total_income24".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Total assessable income".to_string(),
						quantity: vec![
							report
								.quantity_for_id("total_income1")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_income5")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_income10")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_income13")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_income20")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_income24")
								.map(|v| v[0])
								.unwrap_or(0),
						],
						id: Some("total_income".to_string()),
						visible: true,
						auto_hide: false,
						link: None,
						heading: true,
						bordered: true,
					},
				}),
				CalculatableDynamicReportEntry::Spacer,
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Work-related travel expenses (D2)".to_string(),
					Some("d2".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.d2",
							false,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item D2".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("d2").unwrap(),
										100,
									),
									id: Some("total_d2".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Work-related self-education expenses (D4)".to_string(),
					Some("d4".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.d4",
							false,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item D4".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("d4").unwrap(),
										100,
									),
									id: Some("total_d4".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Other work-related expenses (D5)".to_string(),
					Some("d5".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.d5",
							false,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item D5".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("d5").unwrap(),
										100,
									),
									id: Some("total_d5".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Gifts or donations (D9)".to_string(),
					Some("d9".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.d9",
							false,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item D9".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("d9").unwrap(),
										100,
									),
									id: Some("total_d9".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatableSection(CalculatableSection::new(
					"Other deductions (D15)".to_string(),
					Some("d15".to_string()),
					true,
					true,
					{
						let mut entries = entries_for_kind(
							"austax.d15",
							false,
							&vec![balances],
							&kinds_for_account,
						);
						entries.push(CalculatableDynamicReportEntry::CalculatedRow(
							CalculatedRow {
								calculate_fn: |report| LiteralRow {
									text: "Total item D15".to_string(),
									quantity: floor_quantity(
										report.subtotal_for_id("d15").unwrap(),
										100,
									),
									id: Some("total_d15".to_string()),
									visible: true,
									auto_hide: true,
									link: None,
									heading: true,
									bordered: true,
								},
							},
						));
						entries.push(CalculatableDynamicReportEntry::Spacer);
						entries
					},
				)),
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Total deductions".to_string(),
						quantity: vec![
							report
								.quantity_for_id("total_d2")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_d4")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_d5")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_d9")
								.map(|v| v[0])
								.unwrap_or(0) + report
								.quantity_for_id("total_d15")
								.map(|v| v[0])
								.unwrap_or(0),
						],
						id: Some("total_deductions".to_string()),
						visible: true,
						auto_hide: false,
						link: None,
						heading: true,
						bordered: true,
					},
				}),
				CalculatableDynamicReportEntry::Spacer,
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Net taxable income".to_string(),
						quantity: vec![
							report.quantity_for_id("total_income").unwrap()[0]
								- report.quantity_for_id("total_deductions").unwrap()[0],
						],
						id: Some("net_taxable".to_string()),
						visible: true,
						auto_hide: false,
						link: None,
						heading: true,
						bordered: true,
					},
				}),
				// Precompute RFB amount as this is required for MLS
				CalculatableDynamicReportEntry::LiteralRow(LiteralRow {
					text: "Taxable value of reportable fringe benefits".to_string(),
					quantity: vec![rfb_taxable],
					id: Some("rfb_taxable".to_string()),
					visible: false,
					auto_hide: false,
					link: None,
					heading: false,
					bordered: false,
				}),
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Grossed-up value".to_string(),
						quantity: vec![get_grossedup_rfb(
							report.quantity_for_id("rfb_taxable").unwrap()[0],
						)],
						id: Some("rfb_grossedup".to_string()),
						visible: false,
						auto_hide: false,
						link: None,
						heading: false,
						bordered: false,
					},
				}),
				CalculatableDynamicReportEntry::Spacer,
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Base income tax".to_string(),
						quantity: vec![get_base_income_tax(
							report.quantity_for_id("net_taxable").unwrap()[0],
						)],
						id: Some("tax_base".to_string()),
						visible: true,
						auto_hide: false,
						link: None,
						heading: false,
						bordered: false,
					},
				}),
				// CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
				// 	calculate_fn: |report| LiteralRow {
				// 		text: "Medicare levy".to_string(),
				// 		quantity: vec![get_medicare_levy(
				// 			report.quantity_for_id("net_taxable").unwrap()[0],
				// 		)],
				// 		id: Some("tax_ml".to_string()),
				// 		visible: true,
				// 		auto_hide: true,
				// 		link: None,
				// 		heading: false,
				// 		bordered: false,
				// 	},
				// }),
				// CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
				// 	calculate_fn: |report| LiteralRow {
				// 		text: "Medicare levy".to_string(),
				// 		quantity: vec![get_medicare_levy_surcharge(
				// 			report.quantity_for_id("net_taxable").unwrap()[0],
				// 			report.quantity_for_id("rfb_grossedup").unwrap()[0],
				// 		)],
				// 		id: Some("tax_mls".to_string()),
				// 		visible: true,
				// 		auto_hide: true,
				// 		link: None,
				// 		heading: false,
				// 		bordered: false,
				// 	},
				// }),
				CalculatableDynamicReportEntry::CalculatedRow(CalculatedRow {
					calculate_fn: |report| LiteralRow {
						text: "Total income tax".to_string(),
						quantity: vec![
							report.quantity_for_id("tax_base").unwrap()[0], // + report.quantity_for_id("tax_ml").map(|v| v[0]).unwrap_or(0)
							                                                // + report.quantity_for_id("tax_mls").map(|v| v[0]).unwrap_or(0),
						],
						id: Some("total_tax".to_string()),
						visible: true,
						auto_hide: false,
						link: None,
						heading: true,
						bordered: true,
					},
				}),
			],
		);

		let mut report: DynamicReport = report.calculate();
		report.auto_hide();

		let total_tax = report.quantity_for_id("total_tax").unwrap()[0];

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
						quantity: total_tax,
						commodity: context.db_connection.metadata().reporting_commodity.clone(),
						quantity_ascost: Some(total_tax),
					},
					Posting {
						id: None,
						transaction_id: None,
						description: None,
						account: INCOME_TAX_CONTROL.to_string(),
						quantity: -total_tax,
						commodity: context.db_connection.metadata().reporting_commodity.clone(),
						quantity_ascost: Some(total_tax),
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
) -> Vec<CalculatableDynamicReportEntry> {
	let mut entries_for_kind = entries_for_kind(kind, invert, balances, kinds_for_account);
	entries_for_kind.iter_mut().for_each(|e| match e {
		CalculatableDynamicReportEntry::LiteralRow(row) => row
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
