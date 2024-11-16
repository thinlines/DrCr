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

// Type definitions

export interface Transaction {
	id: number,
	dt: string,
	description: string,
	postings: Posting[]
}

export interface Posting {
	id: number,
	description: string,
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

export function joinedToTransactions(joinedTransactionPostings: JoinedTransactionPosting[]): Transaction[] {
	// Group postings into transactions
	const transactions: Transaction[] = [];
	
	for (const joinedTransactionPosting of joinedTransactionPostings) {
		if (transactions.length === 0 || transactions.at(-1)!.id !== joinedTransactionPosting.transaction_id) {
			transactions.push({
				id: joinedTransactionPosting.transaction_id,
				dt: joinedTransactionPosting.dt,
				description: joinedTransactionPosting.transaction_description,
				postings: []
			});
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
