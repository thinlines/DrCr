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

use chrono::NaiveDate;
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, Row, SqliteConnection};

use crate::account_config::AccountConfiguration;
use crate::{util::format_date, QuantityInt};

pub struct DbConnection {
	url: String,
	metadata: DbMetadata,
}

impl DbConnection {
	pub async fn new(url: &str) -> Self {
		let mut connection = SqliteConnection::connect(url).await.expect("SQL error");
		let metadata = DbMetadata::from_database(&mut connection).await;

		Self {
			url: url.to_string(),
			metadata,
		}
	}

	pub fn metadata(&self) -> &DbMetadata {
		&self.metadata
	}

	pub async fn connect(&self) -> SqliteConnection {
		SqliteConnection::connect(&self.url)
			.await
			.expect("SQL error")
	}

	/// Get account balances from the database
	pub async fn get_balances(&self, date: NaiveDate) -> HashMap<String, QuantityInt> {
		let mut connection = self.connect().await;

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
		).bind(format_date(date)).fetch_all(&mut connection).await.expect("SQL error");

		let mut balances = HashMap::new();
		for row in rows {
			balances.insert(row.get("account"), row.get("quantity"));
		}

		balances
	}

	/// Get account configurations from the database
	pub async fn get_account_configurations(&self) -> Vec<AccountConfiguration> {
		let mut connection = self.connect().await;

		let mut account_configurations =
			sqlx::query("SELECT id, account, kind, data FROM account_configurations")
				.map(|r: SqliteRow| AccountConfiguration {
					id: r.get("id"),
					account: r.get("account"),
					kind: r.get("kind"),
					data: r.get("data"),
				})
				.fetch_all(&mut connection)
				.await
				.expect("SQL error");

		// System accounts
		account_configurations.push(AccountConfiguration {
			id: None,
			account: "Current Year Earnings".to_string(),
			kind: "drcr.equity".to_string(),
			data: None,
		});
		account_configurations.push(AccountConfiguration {
			id: None,
			account: "Retained Earnings".to_string(),
			kind: "drcr.equity".to_string(),
			data: None,
		});

		account_configurations
	}
}

/// Container for cached database-related metadata
pub struct DbMetadata {
	pub version: u32,
	pub eofy_date: NaiveDate,
	pub reporting_commodity: String,
	pub dps: u32,
}

impl DbMetadata {
	/// Initialise [DbMetadata] with values from the metadata database table
	async fn from_database(connection: &mut SqliteConnection) -> Self {
		let version = sqlx::query("SELECT value FROM metadata WHERE key = 'version'")
			.map(|r: SqliteRow| {
				r.get::<String, _>(0)
					.parse()
					.expect("Invalid metadata.version")
			})
			.fetch_one(&mut *connection)
			.await
			.expect("SQL error");

		let eofy_date = sqlx::query("SELECT value FROM metadata WHERE key ='eofy_date'")
			.map(|r: SqliteRow| {
				NaiveDate::parse_from_str(r.get(0), "%Y-%m-%d").expect("Invalid metadata.eofy_date")
			})
			.fetch_one(&mut *connection)
			.await
			.expect("SQL error");

		let reporting_commodity =
			sqlx::query("SELECT value FROM metadata WHERE key = 'reporting_commodity'")
				.map(|r: SqliteRow| r.get(0))
				.fetch_one(&mut *connection)
				.await
				.expect("SQL error");

		let dps = sqlx::query("SELECT value FROM metadata WHERE key = 'amount_dps'")
			.map(|r: SqliteRow| {
				r.get::<String, _>(0)
					.parse()
					.expect("Invalid metadata.amount_dps")
			})
			.fetch_one(&mut *connection)
			.await
			.expect("SQL error");

		DbMetadata {
			version,
			eofy_date,
			reporting_commodity,
			dps,
		}
	}
}
