/*
	DrCr: Double-entry bookkeeping framework
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

mod libdrcr_austax;
mod libdrcr_bridge;
mod sql;

use gtk::prelude::{BinExt, Cast, GtkWindowExt, HeaderBarExt};
use gtk::{EventBox, HeaderBar};
use tauri::{AppHandle, Builder, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

use std::fs;

struct AppState {
	db_filename: Option<String>,
	sql_transactions: Vec<Option<crate::sql::SqliteTransaction>>,
}

// Filename state

#[tauri::command]
async fn get_open_filename(state: State<'_, Mutex<AppState>>) -> Result<Option<String>, ()> {
	let state = state.lock().await;
	Ok(state.db_filename.clone())
}

#[tauri::command]
async fn set_open_filename(
	state: State<'_, Mutex<AppState>>,
	app: AppHandle,
	filename: Option<String>,
) -> Result<(), ()> {
	let mut state = state.lock().await;
	state.db_filename = filename.clone();

	// Persist in store
	let store = app.store("store.json").expect("Error opening store");
	store.set("db_filename", filename);

	Ok(())
}

#[tauri::command]
async fn set_window_title(app: AppHandle, label: &str, title: &str) -> Result<(), ()> {
	// First call Tauri
	let tauri_window = app.get_webview_window(label).unwrap();
	tauri_window.set_title(title).unwrap();

	// Then work around https://github.com/tauri-apps/tauri/issues/13749
	let gtk_window = tauri_window.gtk_window().unwrap();
	match gtk_window.titlebar() {
		Some(titlebar) => {
			let event_box = titlebar
				.downcast::<EventBox>()
				.expect("ApplicationWindow.titlebar not EventBox");

			let header_bar = event_box
				.child()
				.expect("ApplicationWindow.titlebar has null child")
				.downcast::<HeaderBar>()
				.expect("ApplicationWindow.titlebar.child not HeaderBar");

			header_bar.set_title(Some(title));
		}
		None => (),
	}

	Ok(())
}

// Main method

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	Builder::default()
		.setup(|app| {
			// Get open filename
			let store = app.store("store.json")?;
			let db_filename = match store.get("db_filename") {
				None => None,
				Some(serde_json::Value::Null) => None,
				Some(serde_json::Value::String(s)) => {
					if fs::exists(&s)? {
						Some(s)
					} else {
						None
					}
				}
				_ => panic!(
					"Unexpected db_filename in store: {:?}",
					store.get("db_filename")
				),
			};

			app.manage(Mutex::new(AppState {
				db_filename: db_filename,
				sql_transactions: Vec::new(),
			}));

			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_fs::init())
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_sql::Builder::new().build())
		.plugin(tauri_plugin_store::Builder::new().build())
		.invoke_handler(tauri::generate_handler![
			get_open_filename,
			set_open_filename,
			set_window_title,
			libdrcr_austax::get_tax_summary,
			libdrcr_bridge::get_all_transactions_except_earnings_to_equity,
			libdrcr_bridge::get_all_transactions_except_earnings_to_equity_for_account,
			libdrcr_bridge::get_balance_sheet,
			libdrcr_bridge::get_income_statement,
			libdrcr_bridge::get_trial_balance,
			libdrcr_bridge::get_validated_balance_assertions,
			sql::sql_transaction_begin,
			sql::sql_transaction_execute,
			sql::sql_transaction_select,
			sql::sql_transaction_rollback,
			sql::sql_transaction_commit
		])
		.run(tauri::generate_context!())
		.expect("Error while running tauri application");
}
