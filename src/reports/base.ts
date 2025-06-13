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

import { db, serialiseAmount } from '../db.ts';
import { CriticalError } from '../error.ts';

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
	
	// Convert to report to CSV
	toCSV(columns?: string[], subtitle?: string): string {
		let csv = '';
		
		// Title and subtitle
		csv += escapeCSV(this.title) + '\n';
		if (subtitle) {
			csv += escapeCSV(subtitle) + '\n';
		}
		
		// Columns
		for (const column of columns || this.columns) {
			csv += ',' + escapeCSV(column);
		}
		csv += '\n';
		
		// Entries
		for (const entry of this.entries) {
			csv += entryToCSV(entry);
		}
		
		return csv;
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

// Escape the given text as contents of a single CSV field
function escapeCSV(cell: string): string {
	if (cell.indexOf('"') >= 0) {
		return '"' + cell.replaceAll('"', '""') + '"';
	}
	if (cell.indexOf(',') >= 0) {
		return '"' + cell + '"';
	}
	return cell;
}

function entryToCSV(entry: DynamicReportEntry): string {
	if (entry === 'Spacer') {
		return '\n';
	} else if ((entry as { Section: Section }).Section) {
		const section = (entry as { Section: Section }).Section;
		let csv = '';
		for (const sectionEntry of section.entries) {
			csv += entryToCSV(sectionEntry);
		}
		return csv;
	} else if ((entry as { Row: Row }).Row) {
		const row = (entry as { Row: Row}).Row;
		let csv = escapeCSV(row.text);
		for (const quantity of row.quantity) {
			csv += ',' + escapeCSV(serialiseAmount(quantity, db.metadata.reporting_commodity));
		}
		csv += '\n';
		return csv;
	} else {
		throw new CriticalError('Unexpected DynamicReportEntry');
	}
}
