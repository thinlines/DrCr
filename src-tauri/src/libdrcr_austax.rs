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

use libdrcr::reporting::dynamic_report::DynamicReport;
use libdrcr::reporting::types::{ReportingProductId, ReportingProductKind, ReportingStepArgs};
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::libdrcr_bridge::get_report;
use crate::AppState;

#[tauri::command]
pub(crate) async fn get_tax_summary(
	app: AppHandle,
	state: State<'_, Mutex<AppState>>,
) -> Result<String, ()> {
	Ok(get_report(
		app,
		state,
		&ReportingProductId {
			name: "CalculateIncomeTax".to_string(),
			kind: ReportingProductKind::DynamicReport,
			args: ReportingStepArgs::VoidArgs,
		},
	)
	.await
	.downcast_ref::<DynamicReport>()
	.unwrap()
	.to_json())
}
