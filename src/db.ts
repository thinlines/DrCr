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

import { invoke } from '@tauri-apps/api/core';
import { resolveResource } from '@tauri-apps/api/path';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { readTextFile } from '@tauri-apps/plugin-fs';
import Database from '@tauri-apps/plugin-sql';
import { reactive } from 'vue';

import { Balance } from './amounts.ts';
import { ExtendedDatabase } from './dbutil.ts';
import { CriticalError } from './error.ts';

export const DB_VERSION = 3;  // Should match schema.sql
export const DT_FORMAT = 'YYYY-MM-DD HH:mm:ss.SSS000';

export const db = reactive({
	filename: null as (string | null),
	
	// Cached
	metadata: {
		version: null! as number,
		eofy_date: null! as string,
		reporting_commodity: null! as string,
		dps: null! as number,
	},
	
	init: async function(filename: string | null): Promise<void> {
		// Set the DB filename and initialise cached data
		this.filename = filename;
		
		await invoke('set_open_filename', { 'filename': filename });
		
		if (filename !== null) {
			await getCurrentWindow().setTitle('DrCr – ' + filename?.replaceAll('\\', '/').split('/').at(-1));
		} else {
			await getCurrentWindow().setTitle('DrCr');
		}
		
		if (filename !== null) {
			const session = await this.load();
			
			// Validate database version
			let dbVersion: {value: string}[];
			try {
				dbVersion = await session.select("SELECT value FROM metadata WHERE key = 'version'");
			} catch (err) {
				throw new CriticalError('Unable to parse database (SQL error getting metadata.version)', err);
			}
			if (dbVersion.length === 0) {
				throw new CriticalError('Unable to parse database (no metadata.version)');
			}
			if (dbVersion[0].value !== DB_VERSION.toString()) {
				throw new CriticalError('Unsupported database version ' + dbVersion[0].value + ' (expected ' + DB_VERSION + ')');
			}
			
			// Initialise cached data
			const metadataRaw: {key: string, value: string}[] = await session.select("SELECT * FROM metadata");
			const metadataObject = Object.fromEntries(metadataRaw.map((x) => [x.key, x.value]));
			this.metadata.version = parseInt(metadataObject.version);
			this.metadata.eofy_date = metadataObject.eofy_date;
			this.metadata.reporting_commodity = metadataObject.reporting_commodity;
			this.metadata.dps = parseInt(metadataObject.amount_dps);
		}
	},
	
	load: async function(): Promise<ExtendedDatabase> {
		return new ExtendedDatabase(await Database.load('sqlite:' + this.filename));
	},
});

export async function createNewDatabase(filename: string, eofy_date: string, reporting_commodity: string, dps: number) {
	// Open new SQLite database
	const session = new ExtendedDatabase(await Database.load('sqlite:' + filename));
	
	// Read SQL schema
	const schemaPath = await resolveResource('schema.sql');
	const schemaSql = await readTextFile(schemaPath);
	
	// Execute SQL
	const transaction = await session.begin();
	await transaction.execute(schemaSql);
	
	// Init metadata
	await transaction.execute(
		`INSERT INTO metadata (key, value) VALUES (?, ?)`,
		['version', DB_VERSION.toString()]  // Manually call .toString() to format as int, otherwise sqlx formats as float
	);
	await transaction.execute(
		`INSERT INTO metadata (key, value) VALUES (?, ?)`,
		['eofy_date', eofy_date]
	);
	await transaction.execute(
		`INSERT INTO metadata (key, value) VALUES (?, ?)`,
		['reporting_commodity', reporting_commodity]
	);
	await transaction.execute(
		`INSERT INTO metadata (key, value) VALUES (?, ?)`,
		['amount_dps', dps.toString()]  // Manually call .toString() to format as int, otherwise sqlx formats as float
	);
	
	await transaction.commit();
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
			quantity_ascost: joinedTransactionPosting.quantity_ascost,
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
		public id: number | null = null,
		public dt: string = '',
		public description: string = '',
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
	id: number | null,
	description: string | null,
	account: string,
	quantity: number,
	commodity: string,
	quantity_ascost?: number,
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
	quantity_ascost?: number,
	running_balance?: number
}

export interface StatementLine {
	id: number | null,
	source_account: string,
	dt: string,
	description: string,
	quantity: number,
	balance: number | null,
	commodity: string
}
