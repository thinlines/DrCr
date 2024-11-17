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

use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use sqlx::{Column, Executor, Row, Sqlite, Transaction, TypeInfo, Value, ValueRef};
use sqlx::query::Query;
use sqlx::sqlite::{SqliteArguments, SqliteRow, SqliteValueRef};
use sqlx::types::time::{Date, PrimitiveDateTime, Time};
use tokio::sync::Mutex;

use tauri::State;
use tauri_plugin_sql::{DbInstances, DbPool, Error};

use crate::AppState;

pub type SqliteTransaction = Transaction<'static, Sqlite>;

#[tauri::command]
pub async fn sql_transaction_begin(state: State<'_, Mutex<AppState>>, db_instances: State<'_, DbInstances>, db: String) -> Result<usize, Error> {
	let instances = db_instances.0.read().await;
	let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
	
	let pool = match db {
		DbPool::Sqlite(pool) => pool,
		//_ => panic!("Unexpected non-SQLite backend"),
	};
	
	// Open transaction
	let transaction = pool.begin().await?;
	
	// Store transaction in state
	let mut state = state.lock().await;
	let available_index = state.sql_transactions.iter().position(|t| t.is_none());
	match available_index {
		Some(i) => {
			state.sql_transactions[i] = Some(transaction);
			Ok(i)
		}
		None => {
			state.sql_transactions.push(Some(transaction));
			Ok(state.sql_transactions.len() - 1)
		}
	}
}

#[tauri::command]
pub async fn sql_transaction_execute(state: State<'_, Mutex<AppState>>, transaction_instance_id: usize, query: String, values: Vec<JsonValue>) -> Result<(u64, i64), Error> {
	let mut state = state.lock().await;
	let transaction =
		state.sql_transactions.get_mut(transaction_instance_id)
		.expect("Invalid database transaction ID")
		.as_mut()  // Take reference to transaction rather than moving out of the Vec
		.expect("Database transaction ID used after closed");
	
	let query = prepare_query(&query, values);
	let result = transaction.execute(query).await?;
	Ok((
		result.rows_affected(),
		result.last_insert_rowid(),
	))
}

#[tauri::command]
pub async fn sql_transaction_select(state: State<'_, Mutex<AppState>>, transaction_instance_id: usize, query: String, values: Vec<JsonValue>) -> Result<Vec<IndexMap<String, JsonValue>>, Error> {
	let mut state = state.lock().await;
	let transaction =
		state.sql_transactions.get_mut(transaction_instance_id)
		.expect("Invalid database transaction ID")
		.as_mut()  // Take reference to transaction rather than moving out of the Vec
		.expect("Database transaction ID used after closed");
	
	let query = prepare_query(&query, values);
	let rows = transaction.fetch_all(query).await?;
	rows_to_vec(rows)
}

#[tauri::command]
pub async fn sql_transaction_rollback(state: State<'_, Mutex<AppState>>, transaction_instance_id: usize) -> Result<(), Error> {
	let mut state = state.lock().await;
	
	let transaction = state.sql_transactions.get_mut(transaction_instance_id)
		.expect("Invalid database transaction ID")
		.take()  // Remove from Vec
		.expect("Database transaction ID used after closed");
	
	transaction.rollback().await?;
	Ok(())
}

#[tauri::command]
pub async fn sql_transaction_commit(state: State<'_, Mutex<AppState>>, transaction_instance_id: usize) -> Result<(), Error> {
	let mut state = state.lock().await;
	
	let transaction = state.sql_transactions.get_mut(transaction_instance_id)
		.expect("Invalid database transaction ID")
		.take()  // Remove from Vec
		.expect("Database transaction ID used after closed");
	
	transaction.commit().await?;
	Ok(())
}

fn prepare_query<'a, 'b: 'a>(_query: &'b str, _values: Vec<JsonValue>) -> Query<'b, Sqlite, SqliteArguments<'a>> {
	// Copied from tauri_plugin_sql/src/commands.rs
	// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
	// Licensed under MIT/Apache 2.0
	
	let mut query = sqlx::query(_query);
	for value in _values {
		if value.is_null() {
			query = query.bind(None::<JsonValue>);
		} else if value.is_string() {
			query = query.bind(value.as_str().unwrap().to_owned())
		} else if let Some(number) = value.as_number() {
			query = query.bind(number.as_f64().unwrap_or_default())
		} else {
			query = query.bind(value);
		}
	}
	query
}

fn rows_to_vec(rows: Vec<SqliteRow>) -> Result<Vec<IndexMap<String, JsonValue>>, Error> {
	// Copied from tauri_plugin_sql/src/commands.rs
	// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
	// Licensed under MIT/Apache 2.0
	
	let mut values = Vec::new();
	for row in rows {
		let mut value = IndexMap::default();
		for (i, column) in row.columns().iter().enumerate() {
			let v = row.try_get_raw(i)?;
			
			let v = decode_sqlite_to_json(v)?;
			
			value.insert(column.name().to_string(), v);
		}
		
		values.push(value);
	}
	Ok(values)
}

fn decode_sqlite_to_json(v: SqliteValueRef) -> Result<JsonValue, Error> {
	// Copied from tauri_plugin_sql/src/decode/sqlite.rs
	// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
	// Licensed under MIT/Apache 2.0
	
	// Same as tauri_plugin_sql::decode::sqlite::to_json but that function is not exposed
	
	if v.is_null() {
		return Ok(JsonValue::Null);
	}
	
	let res = match v.type_info().name() {
		"TEXT" => {
			if let Ok(v) = v.to_owned().try_decode() {
				JsonValue::String(v)
			} else {
				JsonValue::Null
			}
		}
		"REAL" => {
			if let Ok(v) = v.to_owned().try_decode::<f64>() {
				JsonValue::from(v)
			} else {
				JsonValue::Null
			}
		}
		"INTEGER" | "NUMERIC" => {
			if let Ok(v) = v.to_owned().try_decode::<i64>() {
				JsonValue::Number(v.into())
			} else {
				JsonValue::Null
			}
		}
		"BOOLEAN" => {
			if let Ok(v) = v.to_owned().try_decode() {
				JsonValue::Bool(v)
			} else {
				JsonValue::Null
			}
		}
		"DATE" => {
			if let Ok(v) = v.to_owned().try_decode::<Date>() {
				JsonValue::String(v.to_string())
			} else {
				JsonValue::Null
			}
		}
		"TIME" => {
			if let Ok(v) = v.to_owned().try_decode::<Time>() {
				JsonValue::String(v.to_string())
			} else {
				JsonValue::Null
			}
		}
		"DATETIME" => {
			if let Ok(v) = v.to_owned().try_decode::<PrimitiveDateTime>() {
				JsonValue::String(v.to_string())
			} else {
				JsonValue::Null
			}
		}
		"BLOB" => {
			if let Ok(v) = v.to_owned().try_decode::<Vec<u8>>() {
				JsonValue::Array(v.into_iter().map(|n| JsonValue::Number(n.into())).collect())
			} else {
				JsonValue::Null
			}
		}
		"NULL" => JsonValue::Null,
		_ => return Err(Error::UnsupportedDatatype(v.type_info().name().to_string())),
	};
	
	Ok(res)
}
