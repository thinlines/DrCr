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

// Cannot be a class as these are directly deserialised from JSON
export interface DynamicReport {
	title: string;
	columns: string[];
	entries: DynamicReportEntry[];
}

// serde_json serialises an enum like this
export type DynamicReportEntry = { Section: Section } | { LiteralRow: LiteralRow } | 'Spacer';

export interface Section {
	text: string;
	id: string | null;
	visible: boolean;
	auto_hide: boolean;
	entries: DynamicReportEntry[];
}

export interface LiteralRow {
	text: string;
	quantity: number[];
	id: string;
	visible: boolean;
	auto_hide: boolean;
	link: string | null;
	heading: boolean;
	bordered: boolean;
}

export interface Spacer {
}

export function reportEntryById(report: DynamicReport | Section, id: string): DynamicReportEntry | null {
	for (const entry of report.entries) {
		if ((entry as { Section: Section }).Section) {
			const result = reportEntryById((entry as { Section: Section }).Section, id);
			if (result !== null) {
				return result;
			}
		} else if ((entry as { LiteralRow: LiteralRow }).LiteralRow) {
			if ((entry as { LiteralRow: LiteralRow }).LiteralRow.id === id) {
				return entry;
			}
		}
	}
	return null;
}
