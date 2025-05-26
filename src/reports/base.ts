/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2024  Lee Yingtong Li (RunasSudo)
	
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

import { db, getAccountsForKind } from '../db.ts';

export interface DynamicReport {
	title: string;
	columns: string[];
	entries: DynamicReportEntry[];
}

// serde_json serialises an enum like this
export type DynamicReportEntry = {Section: Section} | {LiteralRow: LiteralRow} | 'Spacer';

export interface Section {
	text: string;
	id: string | null;
	visible: bool;
	auto_hide: bool;
	entries: DynamicReportEntry[];
}

export interface LiteralRow {
	text: string;
	quantity: number[];
	id: string;
	visible: bool;
	auto_hide: bool;
	link: string | null;
	heading: bool;
	bordered: bool;
}

export interface Spacer {
}
