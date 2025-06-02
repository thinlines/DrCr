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
use libdrcr::reporting::dynamic_report::DynamicReport;
use libdrcr::reporting::generate_report;
use libdrcr::reporting::types::{
	BalancesAt, DateArgs, DateStartDateEndArgs, MultipleDateArgs, MultipleDateStartDateEndArgs,
	ReportingContext, ReportingProduct, ReportingProductId, ReportingProductKind,
	ReportingStepArgs, Transactions,
};
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Mutex;

use crate::AppState;

fn prepare_reporting_context(context: &mut ReportingContext) {
	libdrcr::reporting::steps::register_lookup_fns(context);
	libdrcr::reporting::builders::register_dynamic_builders(context);
	libdrcr::plugin::register_lookup_fns(context);
}

fn get_plugins() -> Vec<String> {
	// FIXME: Dynamically get this
	vec!["austax.plugin".to_string()]
}

pub(crate) async fn get_report(
	app: AppHandle,
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
	let mut context = ReportingContext::new(
		db_connection,
		app.path()
			.resolve("plugins", BaseDirectory::Resource)
			.unwrap()
			.to_str()
			.unwrap()
			.to_string(),
		get_plugins(),
		eofy_date,
		"$".to_string(),
	);
	prepare_reporting_context(&mut context);

	// Get dynamic report
	let targets = vec![
		// FIXME: Make this configurable
		ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::VoidArgs,
		},
		target.clone(),
	];
	let products = generate_report(targets, Arc::new(context)).await.unwrap();
	let result = products.get_owned_or_err(&target).unwrap();

	result
}

#[tauri::command]
pub(crate) async fn get_all_transactions_except_earnings_to_equity(
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
) -> Result<String, ()> {
	let transactions = get_report(
		app,
		state,
		&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::DateArgs(DateArgs {
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
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
	account: String,
) -> Result<String, ()> {
	let transactions = get_report(
		app,
		state,
		&ReportingProductId {
			name: "AllTransactionsExceptEarningsToEquity".to_string(),
			kind: ReportingProductKind::Transactions,
			args: ReportingStepArgs::DateArgs(DateArgs {
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
	app: AppHandle,
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
		app,
		state,
		&ReportingProductId {
			name: "BalanceSheet".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateArgs(MultipleDateArgs {
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
	app: AppHandle,
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
		app,
		state,
		&ReportingProductId {
			name: "IncomeStatement".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::MultipleDateStartDateEndArgs(MultipleDateStartDateEndArgs {
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
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
	date: String,
) -> Result<String, ()> {
	let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").expect("Invalid date");

	Ok(get_report(
		app,
		state,
		&ReportingProductId {
			name: "TrialBalance".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::DateArgs(DateArgs { date }),
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
	app: AppHandle,
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
	let mut context = ReportingContext::new(
		db_connection,
		app.path()
			.resolve("plugins", BaseDirectory::Resource)
			.unwrap()
			.to_str()
			.unwrap()
			.to_string(),
		get_plugins(),
		eofy_date,
		"$".to_string(),
	);
	prepare_reporting_context(&mut context);

	// Get report targets
	let mut targets = vec![ReportingProductId {
		name: "CalculateIncomeTax".to_string(),
		kind: ReportingProductKind::Transactions,
		args: ReportingStepArgs::VoidArgs,
	}];
	for dt in dates {
		// Request ordinary transaction balances at each balance assertion date
		targets.push(ReportingProductId {
			name: "CombineOrdinaryTransactions".to_string(),
			kind: ReportingProductKind::BalancesAt,
			args: ReportingStepArgs::DateArgs(DateArgs { date: dt.date() }),
		});
	}

	// Run report
	let products = generate_report(targets, Arc::new(context)).await.unwrap();

	// Validate each balance assertion
	let mut validated_assertions = Vec::new();
	for balance_assertion in balance_assertions {
		let balances_at_date = products
			.get_or_err(&ReportingProductId {
				name: "CombineOrdinaryTransactions".to_string(),
				kind: ReportingProductKind::BalancesAt,
				args: ReportingStepArgs::DateArgs(DateArgs {
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
