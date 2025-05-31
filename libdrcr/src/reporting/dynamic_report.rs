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

use serde::{Deserialize, Serialize};

use crate::QuantityInt;

use super::types::ReportingProduct;

/// Represents a dynamically generated report composed of [DynamicReportEntry]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DynamicReport {
	pub title: String,
	pub columns: Vec<String>,
	pub entries: Vec<DynamicReportEntry>,
}

impl DynamicReport {
	pub fn new(title: String, columns: Vec<String>, entries: Vec<DynamicReportEntry>) -> Self {
		Self {
			title,
			columns,
			entries,
		}
	}

	/// Serialise the report (as JSON) using serde
	pub fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}

	/// Look up [DynamicReportEntry] by id
	pub fn by_id(&self, id: &str) -> Option<&DynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry {
				DynamicReportEntry::Section(section) => {
					if let Some(i) = &section.id {
						if i == id {
							return Some(entry);
						}
					}
					if let Some(e) = section.by_id(id) {
						return Some(e);
					}
				}
				DynamicReportEntry::Row(row) => {
					if let Some(i) = &row.id {
						if i == id {
							return Some(entry);
						}
					}
				}
				DynamicReportEntry::Spacer => (),
			}
		}

		None
	}

	// Return the quantities for the [LiteralRow] with the given id
	pub fn quantity_for_id(&self, id: &str) -> Option<&Vec<QuantityInt>> {
		if let Some(entry) = self.by_id(id) {
			if let DynamicReportEntry::Row(row) = entry {
				Some(&row.quantity)
			} else {
				panic!("Called quantity_for_id on non-LiteralRow");
			}
		} else {
			None
		}
	}
}

impl ReportingProduct for DynamicReport {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DynamicReportEntry {
	Section(Section),
	Row(Row),
	Spacer,
}

impl From<Section> for DynamicReportEntry {
	fn from(value: Section) -> Self {
		DynamicReportEntry::Section(value)
	}
}

impl From<Row> for DynamicReportEntry {
	fn from(value: Row) -> Self {
		DynamicReportEntry::Row(value)
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Section {
	pub text: Option<String>,
	pub id: Option<String>,
	pub visible: bool,
	pub entries: Vec<DynamicReportEntry>,
}

impl Section {
	/// Look up [DynamicReportEntry] by id
	pub fn by_id(&self, id: &str) -> Option<&DynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry {
				DynamicReportEntry::Section(section) => {
					if let Some(i) = &section.id {
						if i == id {
							return Some(entry);
						}
					}
					if let Some(e) = section.by_id(id) {
						return Some(e);
					}
				}
				DynamicReportEntry::Row(row) => {
					if let Some(i) = &row.id {
						if i == id {
							return Some(entry);
						}
					}
				}
				DynamicReportEntry::Spacer => (),
			}
		}

		None
	}

	/// Calculate the subtotals for this [Section]
	pub fn subtotal(&self, report: &DynamicReport) -> Vec<QuantityInt> {
		let mut subtotals = vec![0; report.columns.len()];
		for entry in self.entries.iter() {
			match entry {
				DynamicReportEntry::Section(section) => {
					for (col_idx, subtotal) in section.subtotal(report).into_iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				DynamicReportEntry::Row(row) => {
					for (col_idx, subtotal) in row.quantity.iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				DynamicReportEntry::Spacer => (),
			}
		}
		subtotals
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Row {
	pub text: String,
	pub quantity: Vec<QuantityInt>,
	pub id: Option<String>,
	pub visible: bool,
	pub link: Option<String>,
	pub heading: bool,
	pub bordered: bool,
}

pub fn entries_for_kind(
	kind: &str,
	invert: bool,
	balances: &Vec<&HashMap<String, QuantityInt>>,
	kinds_for_account: &HashMap<String, Vec<String>>,
) -> Vec<DynamicReportEntry> {
	// Get accounts of specified kind
	let mut accounts = kinds_for_account
		.iter()
		.filter_map(|(a, k)| {
			if k.iter().any(|k| k == kind) {
				Some(a)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	accounts.sort();

	let mut entries = Vec::new();
	for account in accounts {
		let quantities = balances
			.iter()
			.map(|b| b.get(account).unwrap_or(&0) * if invert { -1 } else { 1 })
			.collect::<Vec<_>>();

		// Do not show if all quantities are zero
		if quantities.iter().all(|q| *q == 0) {
			continue;
		}

		// Some exceptions for the link
		let link;
		if account == crate::CURRENT_YEAR_EARNINGS {
			link = Some("/income-statement".to_string());
		} else if account == crate::RETAINED_EARNINGS {
			link = None
		} else {
			link = Some(format!("/transactions/{}", account));
		}

		let entry = Row {
			text: account.to_string(),
			quantity: quantities,
			id: None,
			visible: true,
			link,
			heading: false,
			bordered: false,
		};
		entries.push(entry.into());
	}

	entries
}
