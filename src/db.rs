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

use std::collections::HashMap;
use std::ops::DerefMut;
use std::{cell::RefCell, future::Future};

use chrono::NaiveDate;
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, Row, SqliteConnection};
use tokio::runtime::Runtime;

use crate::account_config::AccountConfiguration;
use crate::{util::format_date, QuantityInt};

pub struct DbConnection {
	sqlx_connection: RefCell<SqliteConnection>,
}

fn run_blocking<F: Future>(future: F) -> F::Output {
	let rt = Runtime::new().unwrap();
	rt.block_on(future)
}

impl DbConnection {
	/// Connect to the given Sqlite database
	pub fn connect(url: &str) -> Self {
		Self {
			sqlx_connection: RefCell::new(run_blocking(Self::connect_async(url))),
		}
	}

	async fn connect_async(url: &str) -> SqliteConnection {
		SqliteConnection::connect(url).await.expect("SQL error")
	}

	/// Get account balances from the database
	pub fn get_balances(&self, date: NaiveDate) -> HashMap<String, QuantityInt> {
		run_blocking(self.get_balances_async(date))
	}

	async fn get_balances_async(&self, date: NaiveDate) -> HashMap<String, QuantityInt> {
		let mut connection = self.sqlx_connection.borrow_mut();

		let rows = sqlx::query(
		"-- Get last transaction for each account
			WITH max_dt_by_account AS (
				SELECT account, max(dt) AS max_dt
				FROM joined_transactions
				WHERE DATE(dt) <= DATE($1)
				GROUP BY account
			),
			max_tid_by_account AS (
				SELECT max_dt_by_account.account, max(transaction_id) AS max_tid
				FROM max_dt_by_account
				JOIN joined_transactions ON max_dt_by_account.account = joined_transactions.account AND max_dt_by_account.max_dt = joined_transactions.dt
				GROUP BY max_dt_by_account.account
			)
			-- Get running balance at last transaction for each account
			SELECT max_tid_by_account.account, running_balance AS quantity
			FROM max_tid_by_account
			JOIN transactions_with_running_balances ON max_tid = transactions_with_running_balances.transaction_id AND max_tid_by_account.account = transactions_with_running_balances.account"
		).bind(format_date(date)).fetch_all(connection.deref_mut()).await.expect("SQL error");

		let mut balances = HashMap::new();
		for row in rows {
			balances.insert(row.get("account"), row.get("quantity"));
		}

		balances
	}

	/// Get account configurations from the database
	pub fn get_account_configurations(&self) -> Vec<AccountConfiguration> {
		run_blocking(self.get_account_configurations_async())
	}

	async fn get_account_configurations_async(&self) -> Vec<AccountConfiguration> {
		let mut connection = self.sqlx_connection.borrow_mut();

		let account_configurations =
			sqlx::query("SELECT id, account, kind, data FROM account_configurations")
				.map(|r: SqliteRow| AccountConfiguration {
					id: r.get("id"),
					account: r.get("account"),
					kind: r.get("kind"),
					data: r.get("data"),
				})
				.fetch_all(connection.deref_mut())
				.await
				.expect("SQL error");

		account_configurations
	}
}
