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

use std::cell::RefCell;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::QuantityInt;

use super::types::{GenericReportingProduct, ReportingProduct};

/// Represents a dynamically generated report composed of [DynamicReportEntry]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DynamicReport {
	pub title: String,
	pub columns: Vec<String>,
	// This must use RefCell as, during calculation, we iterate while mutating the report
	pub entries: Vec<RefCell<DynamicReportEntry>>,
}

impl DynamicReport {
	pub fn new(title: String, columns: Vec<String>, entries: Vec<DynamicReportEntry>) -> Self {
		Self {
			title,
			columns,
			entries: entries.into_iter().map(|e| RefCell::new(e)).collect(),
		}
	}

	/// Remove all entries from the report where auto_hide is enabled and quantity is zero
	pub fn auto_hide(&mut self) {
		self.entries.retain(|e| match &mut *e.borrow_mut() {
			DynamicReportEntry::Section(section) => {
				section.auto_hide_children();
				if section.can_auto_hide_self() {
					false
				} else {
					true
				}
			}
			DynamicReportEntry::LiteralRow(row) => {
				if row.can_auto_hide() {
					false
				} else {
					true
				}
			}
			DynamicReportEntry::CalculatedRow(_) => true,
			DynamicReportEntry::Spacer => true,
		});
	}

	/// Recursively calculate all [CalculatedRow] entries
	pub fn calculate(&mut self) {
		for (entry_idx, entry) in self.entries.iter().enumerate() {
			let entry_ref = entry.borrow();

			match &*entry_ref {
				DynamicReportEntry::Section(section) => {
					// Clone first, in case calculation needs to take reference to the section
					let mut updated_section = section.clone();
					updated_section.calculate(&self);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = DynamicReportEntry::Section(updated_section);
				}
				DynamicReportEntry::LiteralRow(_) => (),
				DynamicReportEntry::CalculatedRow(row) => {
					let updated_row = row.calculate(&self);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = DynamicReportEntry::LiteralRow(updated_row);
				}
				DynamicReportEntry::Spacer => (),
			}
		}
	}

	/// Look up [DynamicReportEntry] by id
	///
	/// Returns a cloned copy of the [DynamicReportEntry]. This is necessary because the entry may be within a [Section], and [RefCell] semantics cannot express this type of nested borrow.
	pub fn by_id(&self, id: &str) -> Option<DynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry.try_borrow() {
				Ok(entry) => match &*entry {
				DynamicReportEntry::Section(section) => {
					if let Some(i) = &section.id {
						if i == id {
								return Some(entry.clone());
						}
					}
					if let Some(e) = section.by_id(id) {
						return Some(e);
					}
				}
				DynamicReportEntry::LiteralRow(row) => {
					if let Some(i) = &row.id {
						if i == id {
								return Some(entry.clone());
						}
					}
				}
				DynamicReportEntry::CalculatedRow(_) => (),
				DynamicReportEntry::Spacer => (),
				},
				Err(err) => panic!(
					"Attempt to call by_id on DynamicReportEntry which is mutably borrowed: {}",
					err
				),
			}
		}

		None
	}

	/// Calculate the subtotals for the [Section] with the given id
	pub fn subtotal_for_id(&self, id: &str) -> Vec<QuantityInt> {
		let entry = self.by_id(id).expect("Invalid id");
		if let DynamicReportEntry::Section(section) = entry {
			section.subtotal(&self)
		} else {
			panic!("Called subtotal_for_id on non-Section");
		}
	}

	/// Serialise the report (as JSON) using serde
	pub fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}
}

impl GenericReportingProduct for DynamicReport {}
impl ReportingProduct for DynamicReport {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DynamicReportEntry {
	Section(Section),
	LiteralRow(LiteralRow),
	#[serde(skip)]
	CalculatedRow(CalculatedRow),
	Spacer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Section {
	pub text: String,
	pub id: Option<String>,
	pub visible: bool,
	pub auto_hide: bool,
	pub entries: Vec<RefCell<DynamicReportEntry>>,
}

impl Section {
	pub fn new(
		text: String,
		id: Option<String>,
		visible: bool,
		auto_hide: bool,
		entries: Vec<DynamicReportEntry>,
	) -> Self {
		Self {
			text,
			id,
			visible,
			auto_hide,
			entries: entries.into_iter().map(|e| RefCell::new(e)).collect(),
		}
	}

	fn auto_hide_children(&mut self) {
		self.entries.retain(|e| match &mut *e.borrow_mut() {
			DynamicReportEntry::Section(section) => {
				section.auto_hide_children();
				if section.can_auto_hide_self() {
					false
				} else {
					true
				}
			}
			DynamicReportEntry::LiteralRow(row) => {
				if row.can_auto_hide() {
					false
				} else {
					true
				}
			}
			DynamicReportEntry::CalculatedRow(_) => true,
			DynamicReportEntry::Spacer => true,
		});
	}

	fn can_auto_hide_self(&self) -> bool {
		self.auto_hide
			&& self.entries.iter().all(|e| match &*e.borrow() {
				DynamicReportEntry::Section(section) => section.can_auto_hide_self(),
				DynamicReportEntry::LiteralRow(row) => row.can_auto_hide(),
				DynamicReportEntry::CalculatedRow(_) => false,
				DynamicReportEntry::Spacer => true,
			})
	}

	/// Recursively calculate all [CalculatedRow] entries
	pub fn calculate(&mut self, report: &DynamicReport) {
		for (entry_idx, entry) in self.entries.iter().enumerate() {
			let entry_ref = entry.borrow();

			match &*entry_ref {
				DynamicReportEntry::Section(section) => {
					// Clone first, in case calculation needs to take reference to the section
					let mut updated_section = section.clone();
					updated_section.calculate(&report);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = DynamicReportEntry::Section(updated_section);
				}
				DynamicReportEntry::LiteralRow(_) => (),
				DynamicReportEntry::CalculatedRow(row) => {
					let updated_row = row.calculate(&report);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = DynamicReportEntry::LiteralRow(updated_row);
				}
				DynamicReportEntry::Spacer => (),
			}
		}
	}

	/// Look up [DynamicReportEntry] by id
	///
	/// Returns a cloned copy of the [DynamicReportEntry].
	pub fn by_id(&self, id: &str) -> Option<DynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry.try_borrow() {
				Ok(entry) => match &*entry {
				DynamicReportEntry::Section(section) => {
					if let Some(i) = &section.id {
						if i == id {
								return Some(entry.clone());
						}
					}
					if let Some(e) = section.by_id(id) {
						return Some(e);
					}
				}
				DynamicReportEntry::LiteralRow(row) => {
					if let Some(i) = &row.id {
						if i == id {
								return Some(entry.clone());
						}
					}
				}
				DynamicReportEntry::CalculatedRow(_) => (),
				DynamicReportEntry::Spacer => (),
				},
				Err(err) => panic!(
					"Attempt to call by_id on DynamicReportEntry which is mutably borrowed: {}",
					err
				),
			}
		}

		None
	}

	/// Calculate the subtotals for this [Section]
	pub fn subtotal(&self, report: &DynamicReport) -> Vec<QuantityInt> {
		let mut subtotals = vec![0; report.columns.len()];
		for entry in self.entries.iter() {
			match &*entry.borrow() {
				DynamicReportEntry::Section(section) => {
					for (col_idx, subtotal) in section.subtotal(report).into_iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				DynamicReportEntry::LiteralRow(row) => {
					for (col_idx, subtotal) in row.quantity.iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				DynamicReportEntry::CalculatedRow(_) => (),
				DynamicReportEntry::Spacer => (),
			}
		}
		subtotals
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LiteralRow {
	pub text: String,
	pub quantity: Vec<QuantityInt>,
	pub id: Option<String>,
	pub visible: bool,
	pub auto_hide: bool,
	pub link: Option<String>,
	pub heading: bool,
	pub bordered: bool,
}

impl LiteralRow {
	/// Returns whether the row has auto_hide enabled and all quantities are zero
	fn can_auto_hide(&self) -> bool {
		self.auto_hide && self.quantity.iter().all(|q| *q == 0)
	}
}

#[derive(Clone, Debug)]
pub struct CalculatedRow {
	//pub text: String,
	pub calculate_fn: fn(report: &DynamicReport) -> LiteralRow,
	//pub id: Option<String>,
	//pub visible: bool,
	//pub auto_hide: bool,
	//pub link: Option<String>,
	//pub heading: bool,
	//pub bordered: bool,
}

impl CalculatedRow {
	fn calculate(&self, report: &DynamicReport) -> LiteralRow {
		(self.calculate_fn)(report)
	}
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

		let entry = LiteralRow {
			text: account.to_string(),
			quantity: quantities,
			id: None,
			visible: true,
			auto_hide: true,
			link: None,
			heading: false,
			bordered: false,
		};
		entries.push(DynamicReportEntry::LiteralRow(entry));
	}

	entries
}
