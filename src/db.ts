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

import { invoke } from '@tauri-apps/api/core';
import { resolveResource } from '@tauri-apps/api/path';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { readTextFile } from '@tauri-apps/plugin-fs';
import Database from '@tauri-apps/plugin-sql';
import { reactive } from 'vue';

import { asCost } from './amounts.ts';
import { ExtendedDatabase } from './dbutil.ts';
import { CriticalError } from './error.ts';

export const DB_VERSION = 4;  // Should match schema.sql
export const DT_FORMAT = 'YYYY-MM-DD HH:mm:ss.SSS000';

export const db = reactive({
	filename: null as (string | null),
	
	// Cached
	metadata: {
		version: null! as number,
		eofy_date: null! as string,
		reporting_commodity: null! as string,
		dps: null! as number,
		plugins: null! as string[],
	},
	
	init: async function(filename: string | null): Promise<void> {
		// Set the DB filename and initialise cached data
		this.filename = filename;
		
		await invoke('set_open_filename', { 'filename': filename });
		
		if (filename !== null) {
			await invoke('set_window_title', {
				'label': await getCurrentWindow().label,
				'title': 'DrCr – ' + filename?.replaceAll('\\', '/').split('/').at(-1)
			});
		} else {
			await invoke('set_window_title', {
				'label': await getCurrentWindow().label,
				'title': 'DrCr'
			});
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
			const currentVersion = parseInt(dbVersion[0].value);
			if (currentVersion > DB_VERSION) {
				throw new CriticalError('Unsupported database version ' + dbVersion[0].value + ' (expected ' + DB_VERSION + ' or lower)');
			}
			if (currentVersion < DB_VERSION) {
				await migrateDatabase(await this.load(), currentVersion, DB_VERSION);
			}
			
			// Initialise cached data
			const metadataRaw: {key: string, value: string}[] = await session.select("SELECT * FROM metadata");
			const metadataObject = Object.fromEntries(metadataRaw.map((x) => [x.key, x.value]));
			this.metadata.version = parseInt(metadataObject.version);
			this.metadata.eofy_date = metadataObject.eofy_date;
			this.metadata.reporting_commodity = metadataObject.reporting_commodity;
			this.metadata.dps = parseInt(metadataObject.amount_dps);
			this.metadata.plugins = metadataObject.plugins.length > 0 ? metadataObject.plugins.split(';') : [];
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
	await transaction.execute(
		`INSERT INTO metadata (key, value) VALUES (?, ?)`,
		['plugins', '']
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

function parseFloatStrict(quantity: string): number {
	// Parses quantity as a float, throwing error on invalid input
	if (!/^-?[0-9]+(\.[0-9]+)?$/.test(quantity)) {
		throw new DeserialiseAmountError('Invalid quantity: ' + quantity);
	}
	return parseFloat(quantity);
}

export function validateCommodity(commodity: string) {
	// Validate that the commodity is correctly formed
	const commodityParts = commodity.split(' ');
	if (commodityParts.length > 2) {
		throw new DeserialiseAmountError('Invalid commodity (more spaces than expected): ' + commodity);
	}
	if (commodityParts.length === 2) {
		// Validate that the second part is a cost basis
		if (commodityParts[1].startsWith('{{') && commodityParts[1].endsWith('}}')) {
			const costBase = commodityParts[1].substring(2, commodityParts[1].length - 2);
			parseFloatStrict(costBase);
		} else if (commodityParts[1].startsWith('{') && commodityParts[1].endsWith('}')) {
			const costBase = commodityParts[1].substring(1, commodityParts[1].length - 1);
			parseFloatStrict(costBase);
		} else {
			throw new DeserialiseAmountError('Invalid cost base: ' + commodityParts[1]);
		}
	}
}

export function deserialiseAmount(amount: string): { quantity: number, commodity: string } {
	const factor = Math.pow(10, db.metadata.dps);
	
	if (amount.length === 0) {
		throw new DeserialiseAmountError('Amount cannot be blank');
	}
	
	if (amount.charAt(0) === '-') {
		// Handle negative amount
		const amountAbs = deserialiseAmount(amount.substring(1));
		return {
			quantity: -amountAbs.quantity,
			commodity: amountAbs.commodity
		};
	}
	
	if (amount.charAt(0) < '0' || amount.charAt(0) > '9') {
		// Check for single letter commodity
		if (amount.length === 1) {
			throw new DeserialiseAmountError('Quantity cannot be blank (expected quantity after commodity symbol ' + amount + ')');
		}
		if ((amount.charAt(1) < '0' || amount.charAt(1) > '9') && amount.charAt(1) !== '-') {
			throw new DeserialiseAmountError('Invalid quantity: ' + amount + ' (expected quantity after single-letter commodity symbol ' + amount.charAt(0) + ')');
		}
		
		let quantity, commodity;
		
		if (amount.indexOf(' ') < 0) {
			// No cost base
			quantity = Math.round(parseFloatStrict(amount.substring(1)) * factor);
			commodity = amount.charAt(0);
		} else {
			// Cost base specified
			quantity = Math.round(parseFloatStrict(amount.substring(1, amount.indexOf(' '))) * factor);
			commodity = amount.charAt(0) + amount.substring(amount.indexOf(' '));
		}
		
		if (!Number.isSafeInteger(quantity)) { throw new DeserialiseAmountError('Quantity not representable by safe integer: ' + amount); }
		validateCommodity(commodity);
		
		return {
			'quantity': quantity,
			'commodity': commodity
		};
	}
	
	if (amount.indexOf(' ') < 0) {
		// Default commodity
		const quantity = Math.round(parseFloatStrict(amount) * factor);
		
		if (!Number.isSafeInteger(quantity)) { throw new DeserialiseAmountError('Quantity not representable by safe integer: ' + amount); }
		
		return {
			'quantity': quantity,
			commodity: db.metadata.reporting_commodity
		};
	}
	
	// Must be multi-letter commodity
	const quantityStr = amount.substring(0, amount.indexOf(' '));
	const quantity = Math.round(parseFloatStrict(quantityStr) * factor)
	
	if (!Number.isSafeInteger(quantity)) { throw new DeserialiseAmountError('Quantity not representable by safe integer: ' + amount); }
	
	const commodity = amount.substring(amount.indexOf(' ') + 1);
	validateCommodity(commodity);
	
	return {
		'quantity': quantity,
		'commodity': commodity
	};
}

export class DeserialiseAmountError extends Error {}

// Type definitions

export class Transaction {
	constructor(
		public id: number | null = null,
		public dt: string = '',
		public description: string = '',
		public postings: Posting[] = [],
	) {}
	
	doesBalance(): boolean {
		let total = 0;
		for (const posting of this.postings) {
			total += asCost(posting.quantity, posting.commodity);
		}
		return total === 0;
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

export function postingQuantityAsCost(posting: Posting | JoinedTransactionPosting) {
	// Convert the posting amount to cost price in the reporting commodity
	
	if (posting.quantity_ascost) {
		return posting.quantity_ascost;
	} else {
		// NB: This branch is rarely taken - most conversions are performed in SQL via the transactions_with_quantity_ascost view
		return asCost(posting.quantity, posting.commodity);
	}
}

export interface StatementLine {
	id: number | null,
	source_account: string,
	dt: string,
	name: string,
	memo: string,
	description: string,
	quantity: number,
	balance: number | null,
	commodity: string
}

async function migrateDatabase(session: ExtendedDatabase, fromVersion: number, toVersion: number) {
	// Perform simple in-place migrations
	const tx = await session.begin();
	let v = fromVersion;
	while (v < toVersion) {
		switch (v) {
			case 3:
				// v3 -> v4: add name and memo columns to statement_lines
				await tx.execute(`ALTER TABLE statement_lines ADD COLUMN name VARCHAR`);
				await tx.execute(`ALTER TABLE statement_lines ADD COLUMN memo VARCHAR`);
				break;
			default:
				await tx.rollback();
				throw new CriticalError('No migration path from version ' + v);
		}
		v++;
	}
	await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'version'`, [toVersion.toString()]);
	await tx.commit();
}
