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
use libdrcr::reporting::dynamic_report::DynamicReport;
use libdrcr::reporting::generate_report;
use libdrcr::reporting::steps::register_lookup_fns;
use libdrcr::reporting::types::{
	DateArgs, MultipleDateArgs, ReportingContext, ReportingProductId, ReportingProductKind,
	VoidArgs,
};
use tauri::State;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

use crate::AppState;

#[tauri::command]
pub(crate) async fn get_balance_sheet(state: State<'_, Mutex<AppState>>) -> Result<String, ()> {
	let state = state.lock().await;
	let db_filename = state.db_filename.clone().unwrap();

	spawn_blocking(move || {
		// Connect to database
		let db_connection =
			DbConnection::connect(format!("sqlite:{}", db_filename.as_str()).as_str());

		// Initialise ReportingContext
		let mut context = ReportingContext::new(
			db_connection,
			NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
			"$".to_string(),
		);
		register_lookup_fns(&mut context);
		register_dynamic_builders(&mut context);

		// Get balance sheet

		let targets = vec![
			ReportingProductId {
				name: "CalculateIncomeTax",
				kind: ReportingProductKind::Transactions,
				args: Box::new(VoidArgs {}),
			},
			ReportingProductId {
				name: "BalanceSheet",
				kind: ReportingProductKind::Generic,
				args: Box::new(MultipleDateArgs {
					dates: vec![DateArgs {
						date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
					}],
				}),
			},
		];

		let products = generate_report(targets, &context).unwrap();
		let result = products
			.get_or_err(&ReportingProductId {
				name: "BalanceSheet",
				kind: ReportingProductKind::Generic,
				args: Box::new(MultipleDateArgs {
					dates: vec![DateArgs {
						date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
					}],
				}),
			})
			.unwrap();

		let balance_sheet = result.downcast_ref::<DynamicReport>().unwrap().to_json();

		Ok(balance_sheet)
	})
	.await
	.unwrap()
}
