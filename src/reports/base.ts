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

export interface DrcrReport {
}

export interface DynamicReportNode {
	id: string | null;
	calculate(parent: DynamicReport | DynamicReportNode): void;
}

export class DynamicReport implements DrcrReport {
	constructor(
		public title: string,
		public entries: DynamicReportNode[] = [],
	) {}
	
	byId(id: string): DynamicReportNode | null {
		// Get the DynamicReportNode with the given ID
		for (const entry of this.entries) {
			if (entry.id === id) {
				return entry;
			}
			if (entry instanceof Section) {
				const result = entry.byId(id);
				if (result) {
					return result;
				}
			}
		}
		return null;
	}
	
	calculate() {
		// Compute all subtotals
		for (const entry of this.entries) {
			entry.calculate(this);
		}
	}
	
	static async entriesForKind(balances: Map<string, number>, kind: string, negate = false) {
		// Get accounts associated with this kind
		const accountsForKind = await getAccountsForKind(await db.load(), kind);
		
		// Return one entry for each such account
		const entries = [];
		for (const account of accountsForKind) {
			if (balances.has(account)) {
				const quantity = balances.get(account)!;
				if (quantity === 0) {
					continue;
				}
				
				entries.push(new Entry(
					account,
					negate ? -quantity : quantity,
				));
			}
		}
		
		return entries;
	}
}

export class Entry implements DynamicReportNode {
	constructor(
		public text: string,
		public quantity: number,
		public id: string | null = null,
		public visible = true,
		public autoHide = false,
		public link: string | null = null,
		public heading = false,
		public bordered = false,
	) {}
	
	calculate(_parent: DynamicReport | DynamicReportNode) {}
}

export class Computed extends Entry {
	constructor(
		public text: string,
		public calc: Function,
		public id: string | null = null,
		public visible = true,
		public autoHide = false,
		public link: string | null = null,
		public heading = false,
		public bordered = false,
	) {
		super(text, null!, id, visible, autoHide, link, heading, bordered);
	}
	
	calculate(_parent: DynamicReport | DynamicReportNode) {
		// Calculate the value of this entry
		this.quantity = this.calc();
	}
}

export class Section implements DynamicReportNode {
	constructor(
		public title: string | null,
		public entries: DynamicReportNode[] = [],
		public id: string | null = null,
		public visible = true,
		public autoHide = false,
	) {}
	
	calculate(_parent: DynamicReport | DynamicReportNode) {
		for (const entry of this.entries) {
			entry.calculate(this);
		}
	}
	
	byId(id: string): DynamicReportNode | null {
		// Get the DynamicReportNode with the given ID
		for (const entry of this.entries) {
			if (entry.id === id) {
				return entry;
			}
			if (entry instanceof Section) {
				const result = entry.byId(id);
				if (result) {
					return result;
				}
			}
		}
		return null;
	}
}

export class Spacer implements DynamicReportNode {
	id = null;
	
	calculate(_parent: DynamicReport | DynamicReportNode) {}
}

export class Subtotal extends Entry {
	constructor(
		public text: string,
		public id: string | null = null,
		public visible = true,
		public bordered = false,
		public floor = 0,
	) {
		super(text, null!, id, visible, false /* autoHide */, null /* link */, true /* heading */, bordered);
	}
	
	calculate(parent: DynamicReport | DynamicReportNode) {
		// Calculate total amount
		if (!(parent instanceof Section)) {
			throw new Error('Attempt to calculate Subtotal not in Section');
		}
		
		this.quantity = 0;
		for (const entry of parent.entries) {
			if (entry instanceof Entry) {
				this.quantity += entry.quantity;
			}
		}
	}
}
