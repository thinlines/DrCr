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

pub struct AccountConfiguration {
	pub id: Option<u64>,
	pub account: String,
	pub kind: String,
	pub data: Option<String>,
}

/// Convert [`Vec<AccountConfiguration>`] into a [HashMap] mapping account names to account kinds
pub fn kinds_for_account(
	account_configurations: Vec<AccountConfiguration>,
) -> HashMap<String, Vec<String>> {
	let mut result = HashMap::new();

	for account_configuration in account_configurations {
		// Record the account kind
		result
			.entry(account_configuration.account)
			.or_insert_with(|| Vec::new())
			.push(account_configuration.kind);
	}

	result
}
