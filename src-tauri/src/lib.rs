/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2024  Lee Yingtong Li (RunasSudo)
	
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

mod sql;

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
async fn get_open_filename(state: State<'_, Mutex<AppState>>) -> Result<Option<String>, tauri_plugin_sql::Error> {
	let state = state.lock().await;
	Ok(state.db_filename.clone())
}

#[tauri::command]
async fn set_open_filename(state: State<'_, Mutex<AppState>>, app: AppHandle, filename: Option<String>) -> Result<(), tauri_plugin_sql::Error> {
	let mut state = state.lock().await;
	state.db_filename = filename.clone();
	
	// Persist in store
	let store = app.store("store.json").expect("Error opening store");
	store.set("db_filename", filename);
	
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
				Some(serde_json::Value::String(s)) => {
					if fs::exists(&s)? {
						Some(s)
					} else {
						None
					}
				},
				_ => panic!("Unexpected db_filename in store")
			};
			
			app.manage(Mutex::new(AppState {
				db_filename: db_filename,
				sql_transactions: Vec::new(),
			}));
			
			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_sql::Builder::new().build())
		.plugin(tauri_plugin_store::Builder::new().build())
		.invoke_handler(tauri::generate_handler![
			get_open_filename, set_open_filename,
			sql::sql_transaction_begin, sql::sql_transaction_execute, sql::sql_transaction_select, sql::sql_transaction_rollback, sql::sql_transaction_commit
		])
		.run(tauri::generate_context!())
		.expect("Error while running tauri application");
}
