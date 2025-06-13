/*
	DrCr: Double-entry bookkeeping framework
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

export class DynamicReport {
	title!: string;
	columns!: string[];
	entries!: DynamicReportEntry[];
	
	static fromJSON(json: string): DynamicReport {
		return Object.assign(new DynamicReport(), JSON.parse(json));
	}
	
	byId(id: string): DynamicReportEntry | null {
		return reportEntryById(this, id);
	}
}

// serde_json serialises an enum like this
export type DynamicReportEntry = { Section: Section } | { Row: Row } | 'Spacer';

export interface Section {
	text: string;
	id: string | null;
	visible: boolean;
	auto_hide: boolean;
	entries: DynamicReportEntry[];
}

export interface Row {
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
			if ((entry as { Section: Section }).Section.id === id) {
				return entry;
			}
			const result = reportEntryById((entry as { Section: Section }).Section, id);
			if (result !== null) {
				return result;
			}
		} else if ((entry as { Row: Row }).Row) {
			if ((entry as { Row: Row }).Row.id === id) {
				return entry;
			}
		}
	}
	return null;
}
