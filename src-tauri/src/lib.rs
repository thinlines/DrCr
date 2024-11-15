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

use tauri::{Builder, Manager, State};

use std::sync::Mutex;

struct AppState {
	db_filename: Option<String>,
}

#[tauri::command]
fn get_open_filename(state: State<'_, Mutex<AppState>>) -> Option<String> {
	let state = state.lock().unwrap();
	state.db_filename.clone()
}

#[tauri::command]
fn set_open_filename(state: State<'_, Mutex<AppState>>, filename: Option<String>) {
	let mut state = state.lock().unwrap();
	state.db_filename = filename;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	Builder::default()
		.setup(|app| {
			app.manage(Mutex::new(AppState {
				db_filename: None
			}));
			Ok(())
		})
		.plugin(tauri_plugin_dialog::init())
		.plugin(tauri_plugin_shell::init())
		.plugin(tauri_plugin_sql::Builder::new().build())
		.invoke_handler(tauri::generate_handler![get_open_filename, set_open_filename])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
