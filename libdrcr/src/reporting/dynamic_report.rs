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

// FIXME: Tidy up this file

use std::cell::RefCell;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::QuantityInt;

use super::types::ReportingProduct;

/// Represents a dynamically generated report composed of [CalculatableDynamicReportEntry]
#[derive(Clone, Debug)]
pub struct CalculatableDynamicReport {
	pub title: String,
	pub columns: Vec<String>,
	// This must use RefCell as, during calculation, we iterate while mutating the report
	pub entries: Vec<RefCell<CalculatableDynamicReportEntry>>,
}

impl CalculatableDynamicReport {
	pub fn new(
		title: String,
		columns: Vec<String>,
		entries: Vec<CalculatableDynamicReportEntry>,
	) -> Self {
		Self {
			title,
			columns,
			entries: entries.into_iter().map(|e| RefCell::new(e)).collect(),
		}
	}

	/// Recursively calculate all [CalculatedRow] entries
	pub fn calculate(self) -> DynamicReport {
		let mut calculated_entries = Vec::new();

		for (entry_idx, entry) in self.entries.iter().enumerate() {
			let entry_ref = entry.borrow();

			match &*entry_ref {
				CalculatableDynamicReportEntry::CalculatableSection(section) => {
					// Clone first, in case calculation needs to take reference to the section
					let updated_section = section.clone().calculate(&self);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = CalculatableDynamicReportEntry::Section(updated_section.clone());

					calculated_entries.push(DynamicReportEntry::Section(updated_section));
				}
				CalculatableDynamicReportEntry::Section(section) => {
					calculated_entries.push(DynamicReportEntry::Section(section.clone()));
				}
				CalculatableDynamicReportEntry::LiteralRow(row) => {
					calculated_entries.push(DynamicReportEntry::LiteralRow(row.clone()));
				}
				CalculatableDynamicReportEntry::CalculatedRow(row) => {
					let updated_row = row.calculate(&self);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = CalculatableDynamicReportEntry::LiteralRow(updated_row.clone());

					calculated_entries.push(DynamicReportEntry::LiteralRow(updated_row));
				}
				CalculatableDynamicReportEntry::Spacer => {
					calculated_entries.push(DynamicReportEntry::Spacer);
				}
			}
		}

		DynamicReport {
			title: self.title,
			columns: self.columns,
			entries: calculated_entries,
		}
	}

	/// Look up [CalculatableDynamicReportEntry] by id
	///
	/// Returns a cloned copy of the [CalculatableDynamicReportEntry]. This is necessary because the entry may be within a [Section], and [RefCell] semantics cannot express this type of nested borrow.
	pub fn by_id(&self, id: &str) -> Option<CalculatableDynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry.try_borrow() {
				Ok(entry) => match &*entry {
					CalculatableDynamicReportEntry::CalculatableSection(section) => {
						if let Some(i) = &section.id {
							if i == id {
								return Some(entry.clone());
							}
						}
						if let Some(e) = section.by_id(id) {
							return Some(e);
						}
					}
					CalculatableDynamicReportEntry::Section(section) => {
						if let Some(i) = &section.id {
							if i == id {
								return Some(entry.clone());
							}
						}
						if let Some(e) = section.by_id(id) {
							return Some(match e {
								DynamicReportEntry::Section(section) => {
									CalculatableDynamicReportEntry::Section(section.clone())
								}
								DynamicReportEntry::LiteralRow(row) => {
									CalculatableDynamicReportEntry::LiteralRow(row.clone())
								}
								DynamicReportEntry::Spacer => {
									CalculatableDynamicReportEntry::Spacer
								}
							});
						}
					}
					CalculatableDynamicReportEntry::LiteralRow(row) => {
						if let Some(i) = &row.id {
							if i == id {
								return Some(entry.clone());
							}
						}
					}
					CalculatableDynamicReportEntry::CalculatedRow(_) => (),
					CalculatableDynamicReportEntry::Spacer => (),
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
	pub fn subtotal_for_id(&self, id: &str) -> Option<Vec<QuantityInt>> {
		if let Some(entry) = self.by_id(id) {
			if let CalculatableDynamicReportEntry::CalculatableSection(section) = entry {
				Some(section.subtotal(&self))
			} else {
				panic!("Called subtotal_for_id on non-Section");
			}
		} else {
			None
		}
	}

	// Return the quantities for the [LiteralRow] with the given id
	pub fn quantity_for_id(&self, id: &str) -> Option<Vec<QuantityInt>> {
		if let Some(entry) = self.by_id(id) {
			if let CalculatableDynamicReportEntry::LiteralRow(row) = entry {
				Some(row.quantity)
			} else {
				panic!("Called quantity_for_id on non-LiteralRow");
			}
		} else {
			None
		}
	}
}

/// Represents a dynamically generated report composed of [DynamicReportEntry], with no [CalculatedRow]s
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

	/// Remove all entries from the report where auto_hide is enabled and quantity is zero
	pub fn auto_hide(&mut self) {
		self.entries.retain_mut(|e| match e {
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
			DynamicReportEntry::Spacer => true,
		});
	}

	/// Serialise the report (as JSON) using serde
	pub fn to_json(&self) -> String {
		serde_json::to_string(self).unwrap()
	}
}

impl ReportingProduct for DynamicReport {}

#[derive(Clone, Debug)]
pub enum CalculatableDynamicReportEntry {
	CalculatableSection(CalculatableSection),
	Section(Section),
	LiteralRow(LiteralRow),
	CalculatedRow(CalculatedRow),
	Spacer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DynamicReportEntry {
	Section(Section),
	LiteralRow(LiteralRow),
	Spacer,
}

#[derive(Clone, Debug)]
pub struct CalculatableSection {
	pub text: String,
	pub id: Option<String>,
	pub visible: bool,
	pub auto_hide: bool,
	pub entries: Vec<RefCell<CalculatableDynamicReportEntry>>,
}

impl CalculatableSection {
	pub fn new(
		text: String,
		id: Option<String>,
		visible: bool,
		auto_hide: bool,
		entries: Vec<CalculatableDynamicReportEntry>,
	) -> Self {
		Self {
			text,
			id,
			visible,
			auto_hide,
			entries: entries.into_iter().map(|e| RefCell::new(e)).collect(),
		}
	}

	/// Recursively calculate all [CalculatedRow] entries
	pub fn calculate(&mut self, report: &CalculatableDynamicReport) -> Section {
		let mut calculated_entries = Vec::new();

		for (entry_idx, entry) in self.entries.iter().enumerate() {
			let entry_ref = entry.borrow();

			match &*entry_ref {
				CalculatableDynamicReportEntry::CalculatableSection(section) => {
					let updated_section = section.clone().calculate(&report);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = CalculatableDynamicReportEntry::Section(updated_section.clone());

					calculated_entries.push(DynamicReportEntry::Section(updated_section));
				}
				CalculatableDynamicReportEntry::Section(section) => {
					calculated_entries.push(DynamicReportEntry::Section(section.clone()));
				}
				CalculatableDynamicReportEntry::LiteralRow(row) => {
					calculated_entries.push(DynamicReportEntry::LiteralRow(row.clone()));
				}
				CalculatableDynamicReportEntry::CalculatedRow(row) => {
					let updated_row = row.calculate(&report);

					drop(entry_ref); // Drop entry_ref so we can borrow mutably
					let mut entry_mut = self.entries[entry_idx].borrow_mut();
					*entry_mut = CalculatableDynamicReportEntry::LiteralRow(updated_row.clone());

					calculated_entries.push(DynamicReportEntry::LiteralRow(updated_row));
				}
				CalculatableDynamicReportEntry::Spacer => {
					calculated_entries.push(DynamicReportEntry::Spacer);
				}
			}
		}

		Section {
			text: self.text.clone(),
			id: self.id.clone(),
			visible: self.visible,
			auto_hide: self.auto_hide,
			entries: calculated_entries,
		}
	}

	/// Look up [CalculatableDynamicReportEntry] by id
	///
	/// Returns a cloned copy of the [CalculatableDynamicReportEntry].
	pub fn by_id(&self, id: &str) -> Option<CalculatableDynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry.try_borrow() {
				Ok(entry) => match &*entry {
					CalculatableDynamicReportEntry::CalculatableSection(section) => {
						if let Some(i) = &section.id {
							if i == id {
								return Some(entry.clone());
							}
						}
						if let Some(e) = section.by_id(id) {
							return Some(e);
						}
					}
					CalculatableDynamicReportEntry::Section(_) => todo!(),
					CalculatableDynamicReportEntry::LiteralRow(row) => {
						if let Some(i) = &row.id {
							if i == id {
								return Some(entry.clone());
							}
						}
					}
					CalculatableDynamicReportEntry::CalculatedRow(_) => (),
					CalculatableDynamicReportEntry::Spacer => (),
				},
				Err(err) => panic!(
					"Attempt to call by_id on DynamicReportEntry which is mutably borrowed: {}",
					err
				),
			}
		}

		None
	}

	/// Calculate the subtotals for this [CalculatableSection]
	pub fn subtotal(&self, report: &CalculatableDynamicReport) -> Vec<QuantityInt> {
		let mut subtotals = vec![0; report.columns.len()];
		for entry in self.entries.iter() {
			match &*entry.borrow() {
				CalculatableDynamicReportEntry::CalculatableSection(section) => {
					for (col_idx, subtotal) in section.subtotal(report).into_iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				CalculatableDynamicReportEntry::Section(section) => {
					for (col_idx, subtotal) in section.subtotal(report).into_iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				CalculatableDynamicReportEntry::LiteralRow(row) => {
					for (col_idx, subtotal) in row.quantity.iter().enumerate() {
						subtotals[col_idx] += subtotal;
					}
				}
				CalculatableDynamicReportEntry::CalculatedRow(_) => (),
				CalculatableDynamicReportEntry::Spacer => (),
			}
		}
		subtotals
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Section {
	pub text: String,
	pub id: Option<String>,
	pub visible: bool,
	pub auto_hide: bool,
	pub entries: Vec<DynamicReportEntry>,
}

impl Section {
	fn auto_hide_children(&mut self) {
		self.entries.retain_mut(|e| match e {
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
			DynamicReportEntry::Spacer => true,
		});
	}

	fn can_auto_hide_self(&self) -> bool {
		self.auto_hide
			&& self.entries.iter().all(|e| match e {
				DynamicReportEntry::Section(section) => section.can_auto_hide_self(),
				DynamicReportEntry::LiteralRow(row) => row.can_auto_hide(),
				DynamicReportEntry::Spacer => true,
			})
	}

	/// Look up [DynamicReportEntry] by id
	///
	/// Returns a cloned copy of the [DynamicReportEntry].
	pub fn by_id(&self, id: &str) -> Option<DynamicReportEntry> {
		// Manually iterate over self.entries rather than self.entries()
		// To catch the situation where entry is already mutably borrowed
		for entry in self.entries.iter() {
			match entry {
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
				DynamicReportEntry::Spacer => (),
			}
		}

		None
	}

	/// Calculate the subtotals for this [Section]
	pub fn subtotal(&self, report: &CalculatableDynamicReport) -> Vec<QuantityInt> {
		let mut subtotals = vec![0; report.columns.len()];
		for entry in self.entries.iter() {
			match entry {
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
	pub calculate_fn: fn(report: &CalculatableDynamicReport) -> LiteralRow,
	//pub id: Option<String>,
	//pub visible: bool,
	//pub auto_hide: bool,
	//pub link: Option<String>,
	//pub heading: bool,
	//pub bordered: bool,
}

impl CalculatedRow {
	fn calculate(&self, report: &CalculatableDynamicReport) -> LiteralRow {
		(self.calculate_fn)(report)
	}
}

pub fn entries_for_kind(
	kind: &str,
	invert: bool,
	balances: &Vec<&HashMap<String, QuantityInt>>,
	kinds_for_account: &HashMap<String, Vec<String>>,
) -> Vec<CalculatableDynamicReportEntry> {
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

		// Some exceptions for the link
		let link;
		if account == crate::CURRENT_YEAR_EARNINGS {
			link = Some("/income-statement".to_string());
		} else if account == crate::RETAINED_EARNINGS {
			link = None
		} else {
			link = Some(format!("/transactions/{}", account));
		}

		let entry = LiteralRow {
			text: account.to_string(),
			quantity: quantities,
			id: None,
			visible: true,
			auto_hide: true,
			link,
			heading: false,
			bordered: false,
		};
		entries.push(CalculatableDynamicReportEntry::LiteralRow(entry));
	}

	entries
}
