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

use chrono::{NaiveDate, NaiveDateTime};
use sqlx::sqlite::SqliteRow;
use sqlx::{Connection, Row, SqliteConnection};

use crate::account_config::AccountConfiguration;
use crate::statements::StatementLine;
use crate::transaction::{Posting, Transaction, TransactionWithPostings};
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

	/// Get transactions from the database
	pub async fn get_transactions(&self) -> Vec<TransactionWithPostings> {
		let mut connection = self.connect().await;

		let rows = sqlx::query(
			"SELECT transaction_id, dt, transaction_description, id, description, account, quantity, commodity, quantity_ascost
			FROM transactions_with_quantity_ascost
			ORDER BY dt, transaction_id, id"
		).fetch_all(&mut connection).await.expect("SQL error");

		// Un-flatten transaction list
		let mut transactions: Vec<TransactionWithPostings> = Vec::new();

		for row in rows {
			if transactions.is_empty()
				|| transactions.last().unwrap().transaction.id != row.get("transaction_id")
			{
				// New transaction
				transactions.push(TransactionWithPostings {
					transaction: Transaction {
						id: row.get("transaction_id"),
						dt: NaiveDateTime::parse_from_str(row.get("dt"), "%Y-%m-%d %H:%M:%S.%6f")
							.expect("Invalid transactions.dt"),
						description: row.get("transaction_description"),
					},
					postings: Vec::new(),
				});
			}

			transactions.last_mut().unwrap().postings.push(Posting {
				id: row.get("id"),
				transaction_id: row.get("transaction_id"),
				description: row.get("description"),
				account: row.get("account"),
				quantity: row.get("quantity"),
				commodity: row.get("commodity"),
				quantity_ascost: row.get("quantity_ascost"),
			});
		}

		transactions
	}

	/// Get unreconciled statement lines from the database
	pub async fn get_unreconciled_statement_lines(&self) -> Vec<StatementLine> {
		let mut connection = self.connect().await;

		let rows = sqlx::query(
			// On testing, JOIN is much faster than WHERE NOT EXISTS
			"SELECT statement_lines.* FROM statement_lines
			LEFT JOIN statement_line_reconciliations ON statement_lines.id = statement_line_reconciliations.statement_line_id
			WHERE statement_line_reconciliations.id IS NULL"
		).map(|r: SqliteRow| StatementLine {
			id: Some(r.get("id")),
			source_account: r.get("source_account"),
			dt: NaiveDateTime::parse_from_str(r.get("dt"), "%Y-%m-%d").expect("Invalid statement_lines.dt"),
			description: r.get("description"),
			quantity: r.get("quantity"),
			balance: r.get("balance"),
			commodity: r.get("commodity"),
		}).fetch_all(&mut connection).await.expect("SQL error");

		rows
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
