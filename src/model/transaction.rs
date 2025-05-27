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

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::QuantityInt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
	pub id: Option<u64>,
	#[serde(with = "crate::serde::naivedatetime_to_js")]
	pub dt: NaiveDateTime,
	pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionWithPostings {
	#[serde(flatten)]
	pub transaction: Transaction,
	pub postings: Vec<Posting>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Posting {
	pub id: Option<u64>,
	pub transaction_id: Option<u64>,
	pub description: Option<String>,
	pub account: String,
	pub quantity: QuantityInt,
	pub commodity: String,
	pub quantity_ascost: Option<QuantityInt>,
	//pub running_balance: Option<QuantityInt>,
}

pub(crate) fn update_balances_from_transactions<
	'a,
	I: Iterator<Item = &'a TransactionWithPostings>,
>(
	balances: &mut HashMap<String, QuantityInt>,
	transactions: I,
) {
	for transaction in transactions {
		for posting in transaction.postings.iter() {
			// FIXME: Do currency conversion
			let running_balance = balances.get(&posting.account).unwrap_or(&0) + posting.quantity;
			balances.insert(posting.account.clone(), running_balance);
		}
	}
}
