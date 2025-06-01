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

use std::sync::Arc;

use chrono::NaiveDate;
use libdrcr::db::DbConnection;
use libdrcr::reporting::calculator::{steps_as_graphviz, steps_for_targets};
use libdrcr::reporting::dynamic_report::DynamicReport;
use libdrcr::reporting::generate_report;
use libdrcr::reporting::types::{
	DateArgs, DateStartDateEndArgs, MultipleDateArgs, MultipleDateStartDateEndArgs,
	ReportingContext, ReportingProductId, ReportingProductKind, ReportingStepArgs,
};

#[tokio::main]
async fn main() {
	const YEAR: i32 = 2025;

	// Connect to database
	let db_connection = DbConnection::new("sqlite:drcr_testing.db").await;

	// Initialise ReportingContext
	let mut context = ReportingContext::new(
		db_connection,
		"plugins".to_string(),
		vec!["austax.austax".to_string()],
		NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
		"$".to_string(),
	);
	libdrcr::plugin::register_lookup_fns(&mut context);
	libdrcr::reporting::steps::register_lookup_fns(&mut context);
	libdrcr::reporting::builders::register_dynamic_builders(&mut context);

	let context = Arc::new(context);

	// Print Graphviz

	let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::VoidArgs,
		},
		// ReportingProductId {
		// 	name: "AllTransactionsExceptEarningsToEquity",
		// 	kind: ReportingProductKind::Transactions,
		// 	args: Box::new(DateArgs {
		// 		date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
		// 	}),
		// },
		ReportingProductId {
			name: "BalanceSheet".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateArgs(MultipleDateArgs {
				dates: vec![DateArgs {
					date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
				}],
			}),
		},
		ReportingProductId {
			name: "IncomeStatement".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateStartDateEndArgs(MultipleDateStartDateEndArgs {
				dates: vec![DateStartDateEndArgs {
					date_start: NaiveDate::from_ymd_opt(YEAR - 1, 7, 1).unwrap(),
					date_end: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
				}],
			}),
		},
	];
	let (sorted_steps, dependencies) = steps_for_targets(targets, &context).unwrap();

	println!("Graphviz:");
	println!("{}", steps_as_graphviz(&sorted_steps, &dependencies));

	// Get income statement

	let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::VoidArgs,
		},
		ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity".to_string(),
			kind: ReportingProductKind::BalancesBetween,
			args: ReportingStepArgs::DateStartDateEndArgs(DateStartDateEndArgs {
				date_start: NaiveDate::from_ymd_opt(YEAR - 1, 7, 1).unwrap(),
				date_end: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		},
	];

	let products = generate_report(targets, Arc::clone(&context))
		.await
		.unwrap();

	let result = products
		.get_or_err(&ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::VoidArgs,
		})
		.unwrap();

	println!("Tax summary:");
	println!("{:?}", result);
	
	let result = products
		.get_or_err(&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity".to_string(),
			kind: ReportingProductKind::BalancesBetween,
			args: ReportingStepArgs::DateStartDateEndArgs(DateStartDateEndArgs {
				date_start: NaiveDate::from_ymd_opt(YEAR - 1, 7, 1).unwrap(),
				date_end: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		})
		.unwrap();

	println!("Income statement:");
	println!("{:?}", result);

	// Get balance sheet

	let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::VoidArgs,
		},
		ReportingProductId {
			name: "BalanceSheet".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateArgs(MultipleDateArgs {
				dates: vec![DateArgs {
					date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
				}],
			}),
		},
	];

	let products = generate_report(targets, Arc::clone(&context))
		.await
		.unwrap();
	let result = products
		.get_or_err(&ReportingProductId {
			name: "BalanceSheet".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateArgs(MultipleDateArgs {
				dates: vec![DateArgs {
					date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
				}],
			}),
		})
		.unwrap();

	println!("Balance sheet:");
	println!(
		"{}",
		result.downcast_ref::<DynamicReport>().unwrap().to_json()
	);

	// Get trial balance

	let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::VoidArgs,
		},
		ReportingProductId {
			name: "TrialBalance".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::DateArgs(DateArgs {
				date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		},
	];

	let products = generate_report(targets, Arc::clone(&context))
		.await
		.unwrap();
	let result = products
		.get_or_err(&ReportingProductId {
			name: "TrialBalance".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::DateArgs(DateArgs {
				date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		})
		.unwrap();

	println!("Trial balance:");
	println!(
		"{}",
		result.downcast_ref::<DynamicReport>().unwrap().to_json()
	);

	// Get all transactions

	/*let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax",
			kind: ReportingProductKind::Transactions,
			args: Box::new(VoidArgs {}),
		},
		ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::Transactions,
			args: Box::new(DateArgs {
				date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		},
	];

	let products = generate_report(targets, Arc::clone(&context))
		.await
		.unwrap();
	let result = products
		.get_or_err(&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::Transactions,
			args: Box::new(DateArgs {
				date: NaiveDate::from_ymd_opt(YEAR, 6, 30).unwrap(),
			}),
		})
		.unwrap();

	println!("All transactions:");
	println!(
		"{}",
		result.downcast_ref::<Transactions>().unwrap().to_json()
	);*/
}
