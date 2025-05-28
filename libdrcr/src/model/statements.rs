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

use chrono::NaiveDateTime;

use crate::QuantityInt;

pub struct StatementLine {
	pub id: Option<u64>,
	pub source_account: String,
	pub dt: NaiveDateTime,
	pub description: String,
	pub quantity: QuantityInt,
	pub balance: QuantityInt,
	pub commodity: String,
}
