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

use chrono::NaiveDate;
use libdrcr::db::DbConnection;
use libdrcr::reporting::builders::register_dynamic_builders;
use libdrcr::reporting::generate_report;
use libdrcr::reporting::steps::{
	register_lookup_fns, AllTransactionsExceptRetainedEarnings,
	AllTransactionsIncludingRetainedEarnings, CalculateIncomeTax,
};
use libdrcr::reporting::types::{
	DateArgs, DateStartDateEndArgs, ReportingContext, ReportingProductId, ReportingProductKind,
	ReportingStep,
};

fn main() {
	// Connect to database
	let db_connection = DbConnection::connect("sqlite:drcr_testing.db");

	// Initialise ReportingContext
	let mut context = ReportingContext::new(
		db_connection,
		NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
		"$".to_string(),
	);

	register_lookup_fns(&mut context);
	register_dynamic_builders(&mut context);

	// Get income statement

	let targets: Vec<Box<dyn ReportingStep>> = vec![
		Box::new(CalculateIncomeTax {}),
		Box::new(AllTransactionsExceptRetainedEarnings {
			product_kinds: &[ReportingProductKind::BalancesBetween],
			args: Box::new(DateStartDateEndArgs {
				date_start: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
				date_end: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			}),
		}),
	];

	let products = generate_report(targets, &context).unwrap();
	let result = products
		.get_or_err(&ReportingProductId {
			name: "AllTransactionsExceptRetainedEarnings",
			kind: ReportingProductKind::BalancesBetween,
			args: Box::new(DateStartDateEndArgs {
				date_start: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
				date_end: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			}),
		})
		.unwrap();

	println!("Income statement:");
	println!("{:?}", result);

	// Get balance sheet

	let targets: Vec<Box<dyn ReportingStep>> = vec![
		Box::new(CalculateIncomeTax {}),
		Box::new(AllTransactionsIncludingRetainedEarnings {
			args: DateArgs {
				date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			},
		}),
	];

	let products = generate_report(targets, &context).unwrap();
	let result = products
		.get_or_err(&ReportingProductId {
			name: "AllTransactionsIncludingRetainedEarnings",
			kind: ReportingProductKind::BalancesAt,
			args: Box::new(DateArgs {
				date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			}),
		})
		.unwrap();

	println!("Balance sheet:");
	println!("{:?}", result);
}
