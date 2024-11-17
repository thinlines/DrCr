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
import Database, { QueryResult } from '@tauri-apps/plugin-sql';

export class ExtendedDatabase {
	db: Database;
	
	constructor(db: Database) {
		this.db = db;
	}
	
	async execute(query: string, bindValues?: unknown[]): Promise<QueryResult> {
		return await this.db.execute(query, bindValues);
	}
	
	async select<T>(query: string, bindValues?: unknown[]): Promise<T> {
		return await this.db.select(query, bindValues);
	}
	
	async begin(): Promise<DatabaseTransaction> {
		const transactionInstanceId: number = await invoke('sql_transaction_begin', {
			db: this.db.path
		});
		const db_transaction = new DatabaseTransaction(this, transactionInstanceId);
		registry.register(db_transaction, transactionInstanceId, db_transaction);  // Remember to rollback and close connection on finalization
		return db_transaction;
	}
}

export class DatabaseTransaction {
	db: ExtendedDatabase;
	transactionInstanceId: number;
	
	constructor(db: ExtendedDatabase, transactionInstanceId: number) {
		this.db = db;
		this.transactionInstanceId = transactionInstanceId;
	}
	
	async execute(query: string, bindValues?: unknown[]): Promise<QueryResult> {
		const [rowsAffected, lastInsertId] = await invoke('sql_transaction_execute', {
			transactionInstanceId: this.transactionInstanceId,
			query,
			values: bindValues ?? []
		}) as [number, number];
		
		return {
			lastInsertId: lastInsertId,
			rowsAffected: rowsAffected
		};
	}
	
	async select<T>(query: string, bindValues?: unknown[]): Promise<T> {
		const result: T = await invoke('sql_transaction_select', {
			transactionInstanceId: this.transactionInstanceId,
			query,
			values: bindValues ?? []
		});
		return result;
	}
	
	async rollback(): Promise<void> {
		registry.unregister(this);
		await invoke('sql_transaction_rollback', {
			transactionInstanceId: this.transactionInstanceId
		});
	}
	
	async commit(): Promise<void> {
		registry.unregister(this);
		await invoke('sql_transaction_commit', {
			transactionInstanceId: this.transactionInstanceId
		});
	}
}

const registry = new FinalizationRegistry(async (transactionInstanceId) => {
	// Remember to rollback and close connection on finalization
	await invoke('sql_transaction_rollback', {
		transactionInstanceId: transactionInstanceId
	});
});
