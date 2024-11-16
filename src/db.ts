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

import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Database from '@tauri-apps/plugin-sql';

import { reactive } from 'vue';

import { asCost, Balance } from './amounts.ts';

export const db = reactive({
	filename: null as (string | null),
	
	// Cached
	metadata: {
		version: null! as number,
		eofy_date: null! as string,
		reporting_commodity: null! as string,
		dps: null! as number,
	},
	
	init: async function(filename: string) {
		// Set the DB filename and initialise cached data
		this.filename = filename;
		
		await invoke('set_open_filename', { 'filename': filename });
		await getCurrentWindow().setTitle('DrCr – ' + filename.replaceAll('\\', '/').split('/').at(-1));
		
		// Initialise cached data
		const session = await this.load();
		const metadataRaw: {key: string, value: string}[] = await session.select("SELECT * FROM metadata");
		const metadataObject = Object.fromEntries(metadataRaw.map((x) => [x.key, x.value]));
		this.metadata.version = parseInt(metadataObject.version);
		this.metadata.eofy_date = metadataObject.eofy_date;
		this.metadata.reporting_commodity = metadataObject.reporting_commodity;
		this.metadata.dps = parseInt(metadataObject.amount_dps);
	},
	
	load: async function() {
		return await Database.load('sqlite:' + this.filename);
	},
});

export async function totalBalances(session: Database): Promise<{account: string, quantity: number}[]> {
	await updateRunningBalances();
	
	return await session.select(`
		SELECT p3.account AS account, running_balance AS quantity FROM
		(
			SELECT p1.account, max(p2.transaction_id) AS max_tid FROM
			(
				SELECT account, max(dt) AS max_dt FROM postings JOIN transactions ON postings.transaction_id = transactions.id GROUP BY account
			) p1
			JOIN postings p2 ON p1.account = p2.account AND p1.max_dt = transactions.dt JOIN transactions ON p2.transaction_id = transactions.id GROUP BY p2.account
		) p3
		JOIN postings p4 ON p3.account = p4.account AND p3.max_tid = p4.transaction_id ORDER BY account
	`);
}

export async function updateRunningBalances() {
	// TODO: This is very slow - it would be faster to do this in Rust
	
	// Recompute any required running balances
	const session = await db.load();
	const staleAccountsRaw: {account: string}[] = await session.select('SELECT DISTINCT account FROM postings WHERE running_balance IS NULL');
	const staleAccounts: string[] = staleAccountsRaw.map((x) => x.account);
	
	if (staleAccounts.length === 0) {
		return;
	}
	
	// Get all relevant Postings in database in correct order
	// FIXME: Recompute balances only from the last non-stale balance to be more efficient
	const arraySQL = '(?' + ', ?'.repeat(staleAccounts.length - 1) + ')';
	const joinedTransactionPostings: JoinedTransactionPosting[] = await session.select(
		`SELECT postings.id, account, quantity, commodity, running_balance
		FROM transactions
		JOIN postings ON transactions.id = postings.transaction_id
		WHERE postings.account IN ${arraySQL}
		ORDER BY dt, transaction_id, postings.id`,
		staleAccounts
	);
	
	const runningBalances = new Map();
	for (const posting of joinedTransactionPostings) {
		const openingBalance = runningBalances.get(posting.account) ?? 0;
		const quantityCost = asCost(posting.quantity, posting.commodity);
		const runningBalance = openingBalance + quantityCost;
		
		runningBalances.set(posting.account, runningBalance);
		
		// Update running balance of posting
		// Only perform this update if required, to avoid expensive call to DB
		if (posting.running_balance !== runningBalance) {
			await session.execute(
				`UPDATE postings
				SET running_balance = $1
				WHERE id = $2`,
				[runningBalance, posting.id]
			);
		}
	}
}

export function joinedToTransactions(joinedTransactionPostings: JoinedTransactionPosting[]): Transaction[] {
	// Group postings into transactions
	const transactions: Transaction[] = [];
	
	for (const joinedTransactionPosting of joinedTransactionPostings) {
		if (transactions.length === 0 || transactions.at(-1)!.id !== joinedTransactionPosting.transaction_id) {
			transactions.push(new Transaction(
				joinedTransactionPosting.transaction_id,
				joinedTransactionPosting.dt,
				joinedTransactionPosting.transaction_description,
				[]
			));
		}
		
		transactions.at(-1)!.postings.push({
			id: joinedTransactionPosting.id,
			description: joinedTransactionPosting.description,
			account: joinedTransactionPosting.account,
			quantity: joinedTransactionPosting.quantity,
			commodity: joinedTransactionPosting.commodity,
			running_balance: joinedTransactionPosting.running_balance
		});
	}
	
	return transactions;
}

export function serialiseAmount(quantity: number, commodity: string): string {
	// Pretty print the amount for an editable input
	if (quantity < 0) {
		return '-' + serialiseAmount(-quantity, commodity);
	}
	
	// Scale quantity by decimal places
	const factor = Math.pow(10, db.metadata.dps);
	const wholePart = Math.floor(quantity / factor);
	const fracPart = quantity % factor;
	const quantityString = wholePart.toString() + '.' + fracPart.toString().padStart(db.metadata.dps, '0');
	
	if (commodity === db.metadata.reporting_commodity) {
		return quantityString;
	}
	
	if (commodity.length === 1) {
		return commodity + quantityString;
	}
	
	return quantityString + ' ' + commodity;
}

export function deserialiseAmount(amount: string): { quantity: number, commodity: string } {
	const factor = Math.pow(10, db.metadata.dps);
	
	if (amount.indexOf(' ') < 0) {
		// Default commodity
		const quantity = Math.round(parseFloat(amount) * factor)
		
		if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
		
		return {
			'quantity': quantity,
			commodity: db.metadata.reporting_commodity
		};
	}
	
	// FIXME: Parse single letter commodities
	
	const quantityStr = amount.substring(0, amount.indexOf(' '));
	const quantity = Math.round(parseFloat(quantityStr) * factor)
	
	if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
	
	const commodity = amount.substring(amount.indexOf(' ') + 1);
	
	return {
		'quantity': quantity,
		'commodity': commodity
	};
}

// Type definitions

export class Transaction {
	constructor(
		public id: number = null!,
		public dt: string = null!,
		public description: string = null!,
		public postings: Posting[] = [],
	) {}
	
	doesBalance(): boolean {
		const balance = new Balance();
		for (const posting of this.postings) {
			balance.add(posting.quantity, posting.commodity);
		}
		balance.clean();
		return balance.amounts.length === 0;
	}
}

export interface Posting {
	id: number,
	description: string | null,
	account: string,
	quantity: number,
	commodity: string,
	running_balance?: number
}

export interface JoinedTransactionPosting {
	transaction_id: number,
	dt: string,
	transaction_description: string,
	id: number,
	description: string,
	account: string,
	quantity: number,
	commodity: string,
	running_balance?: number
}
