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

use std::collections::HashSet;
use std::sync::Arc;

use chrono::NaiveDate;
use libdrcr::db::DbConnection;
use libdrcr::model::assertions::BalanceAssertion;
use libdrcr::reporting::builders::register_dynamic_builders;
use libdrcr::reporting::dynamic_report::DynamicReport;
use libdrcr::reporting::generate_report;
use libdrcr::reporting::steps::register_lookup_fns;
use libdrcr::reporting::types::{
	BalancesAt, DateArgs, DateStartDateEndArgs, MultipleDateArgs, MultipleDateStartDateEndArgs,
	ReportingContext, ReportingProduct, ReportingProductId, ReportingProductKind, Transactions,
	VoidArgs,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

use crate::AppState;

async fn get_report(
	state: State<'_, Mutex<AppState>>,
	target: &ReportingProductId,
) -> Box<dyn ReportingProduct> {
	let state = state.lock().await;
	let db_filename = state.db_filename.clone().unwrap();

	// Connect to database
	let db_connection =
		DbConnection::new(format!("sqlite:{}", db_filename.as_str()).as_str()).await;

	// Initialise ReportingContext
	let eofy_date = db_connection.metadata().eofy_date;
	let mut context = ReportingContext::new(db_connection, eofy_date, "$".to_string());
	register_lookup_fns(&mut context);
	register_dynamic_builders(&mut context);

	// Get dynamic report
	let targets = vec![
		ReportingProductId {
			name: "CalculateIncomeTax",
			kind: ReportingProductKind::Transactions,
			args: Box::new(VoidArgs {}),
		},
		target.clone(),
	];
	let products = generate_report(targets, Arc::new(context)).await.unwrap();
	let result = products.get_owned_or_err(&target).unwrap();

	result
}

#[tauri::command]
pub(crate) async fn get_all_transactions_except_earnings_to_equity(
	state: State<'_, Mutex<AppState>>,
) -> Result<String, ()> {
	let transactions = get_report(
		state,
		&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::Transactions,
			args: Box::new(DateArgs {
				date: NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
			}),
		},
	)
	.await
	.downcast::<Transactions>()
	.unwrap()
	.transactions;

	Ok(serde_json::to_string(&transactions).unwrap())
}

#[tauri::command]
pub(crate) async fn get_all_transactions_except_earnings_to_equity_for_account(
	state: State<'_, Mutex<AppState>>,
	account: String,
) -> Result<String, ()> {
	let transactions = get_report(
		state,
		&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity",
			kind: ReportingProductKind::Transactions,
			args: Box::new(DateArgs {
				date: NaiveDate::from_ymd_opt(9999, 12, 31).unwrap(),
			}),
		},
	)
	.await
	.downcast::<Transactions>()
	.unwrap()
	.transactions;

	// Filter only transactions affecting this account
	let filtered_transactions = transactions
		.into_iter()
		.filter(|t| t.postings.iter().any(|p| p.account == account))
		.collect::<Vec<_>>();

	Ok(serde_json::to_string(&filtered_transactions).unwrap())
}

#[tauri::command]
pub(crate) async fn get_balance_sheet(
	state: State<'_, Mutex<AppState>>,
	dates: Vec<String>,
) -> Result<String, ()> {
	let mut date_args = Vec::new();
	for date in dates.iter() {
		date_args.push(DateArgs {
			date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("Invalid date"),
		})
	}

	Ok(get_report(
		state,
		&ReportingProductId {
			name: "BalanceSheet",
			kind: ReportingProductKind::Generic,
			args: Box::new(MultipleDateArgs {
				dates: date_args.clone(),
			}),
		},
	)
	.await
	.downcast_ref::<DynamicReport>()
	.unwrap()
	.to_json())
}

#[tauri::command]
pub(crate) async fn get_income_statement(
	state: State<'_, Mutex<AppState>>,
	dates: Vec<(String, String)>,
) -> Result<String, ()> {
	let mut date_args = Vec::new();
	for (date_start, date_end) in dates.iter() {
		date_args.push(DateStartDateEndArgs {
			date_start: NaiveDate::parse_from_str(date_start, "%Y-%m-%d").expect("Invalid date"),
			date_end: NaiveDate::parse_from_str(date_end, "%Y-%m-%d").expect("Invalid date"),
		})
	}

	Ok(get_report(
		state,
		&ReportingProductId {
			name: "IncomeStatement",
			kind: ReportingProductKind::Generic,
			args: Box::new(MultipleDateStartDateEndArgs {
				dates: date_args.clone(),
			}),
		},
	)
	.await
	.downcast_ref::<DynamicReport>()
	.unwrap()
	.to_json())
}

#[tauri::command]
pub(crate) async fn get_trial_balance(
	state: State<'_, Mutex<AppState>>,
	date: String,
) -> Result<String, ()> {
	let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").expect("Invalid date");

	Ok(get_report(
		state,
		&ReportingProductId {
			name: "TrialBalance",
			kind: ReportingProductKind::Generic,
			args: Box::new(DateArgs { date }),
		},
	)
	.await
	.downcast_ref::<DynamicReport>()
	.unwrap()
	.to_json())
}

#[derive(Deserialize, Serialize)]
struct ValidatedBalanceAssertion {
	#[serde(flatten)]
	assertion: BalanceAssertion,
	is_valid: bool,
}

#[tauri::command]
pub(crate) async fn get_validated_balance_assertions(
	state: State<'_, Mutex<AppState>>,
) -> Result<String, ()> {
	let state = state.lock().await;
	let db_filename = state.db_filename.clone().unwrap();

	// Connect to database
	let db_connection =
		DbConnection::new(format!("sqlite:{}", db_filename.as_str()).as_str()).await;

	let reporting_commodity = db_connection.metadata().reporting_commodity.clone(); // Needed later

	// First get balance assertions from database
	let balance_assertions = db_connection.get_balance_assertions().await;

	// Get dates of balance assertions
	let dates = balance_assertions
		.iter()
		.map(|b| b.dt)
		.collect::<HashSet<_>>();

	// Initialise ReportingContext
	let eofy_date = db_connection.metadata().eofy_date;
	let mut context = ReportingContext::new(db_connection, eofy_date, "$".to_string());
	register_lookup_fns(&mut context);
	register_dynamic_builders(&mut context);

	// Get report targets
	let mut targets = vec![ReportingProductId {
		name: "CalculateIncomeTax",
		kind: ReportingProductKind::Transactions,
		args: Box::new(VoidArgs {}),
	}];
	for dt in dates {
		// Request ordinary transaction balances at each balance assertion date
		targets.push(ReportingProductId {
			name: "CombineOrdinaryTransactions",
			kind: ReportingProductKind::BalancesAt,
			args: Box::new(DateArgs { date: dt.date() }),
		});
	}

	// Run report
	let products = generate_report(targets, Arc::new(context)).await.unwrap();

	// Validate each balance assertion
	let mut validated_assertions = Vec::new();
	for balance_assertion in balance_assertions {
		let balances_at_date = products
			.get_or_err(&ReportingProductId {
				name: "CombineOrdinaryTransactions",
				kind: ReportingProductKind::BalancesAt,
				args: Box::new(DateArgs {
					date: balance_assertion.dt.date(),
				}),
			})
			.unwrap()
			.downcast_ref::<BalancesAt>()
			.unwrap();

		let account_balance = *balances_at_date
			.balances
			.get(&balance_assertion.account)
			.unwrap_or(&0);

		let is_valid = balance_assertion.quantity == account_balance
			&& balance_assertion.commodity == reporting_commodity;

		validated_assertions.push(ValidatedBalanceAssertion {
			assertion: balance_assertion,
			is_valid,
		});
	}

	Ok(serde_json::to_string(&validated_assertions).unwrap())
}
