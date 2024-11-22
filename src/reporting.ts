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

import { asCost } from './amounts.ts';
import { JoinedTransactionPosting, StatementLine, Transaction, joinedToTransactions, totalBalances, totalBalancesAtDate } from './db.ts';
import { ExtendedDatabase } from './dbutil.ts';

export enum ReportingStage {
	// Load transactions from database
	TransactionsFromDatabase = 100,
	
	// Load unreconciled statement lines and other ordinary API transactions
	OrdinaryAPITransactions = 200,
}

export class ReportingWorkflow {
	transactionsForStage: Map<ReportingStage, Transaction[]> = new Map();
	reportsForStage: Map<ReportingStage, Report[]> = new Map();
	
	async generate(session: ExtendedDatabase, dt?: string) {
		// ------------------------
		// TransactionsFromDatabase
		
		let balances: Map<string, number>;
		
		{
			// Load balances from database
			if (dt) {
				balances = await totalBalancesAtDate(session, dt);
			} else {
				balances = await totalBalances(session);
			}
			this.reportsForStage.set(ReportingStage.TransactionsFromDatabase, [new TrialBalanceReport(balances)]);
			
			// Load transactions from database
			let joinedTransactionPostings: JoinedTransactionPosting[];
			if (dt) {
				joinedTransactionPostings = await session.select(
					`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity, running_balance
					FROM transactions
					JOIN postings ON transactions.id = postings.transaction_id
					WHERE DATE(dt) <= DATE($1)
					ORDER BY dt, transaction_id, postings.id`,
					[dt]
				);
			} else {
				joinedTransactionPostings = await session.select(
					`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity, running_balance
					FROM transactions
					JOIN postings ON transactions.id = postings.transaction_id
					ORDER BY dt, transaction_id, postings.id`
				);
			}
			const transactions = joinedToTransactions(joinedTransactionPostings);
			this.transactionsForStage.set(ReportingStage.TransactionsFromDatabase, transactions);
		}
		
		// -----------------------
		// OrdinaryAPITransactions
		
		{
			// Get unreconciled statement lines
			let unreconciledStatementLines: StatementLine[];
			if (dt) {
				unreconciledStatementLines = await session.select(
					// On testing, JOIN is much faster than WHERE NOT EXISTS
					`SELECT statement_lines.* FROM statement_lines
					LEFT JOIN statement_line_reconciliations ON statement_lines.id = statement_line_reconciliations.statement_line_id
					WHERE statement_line_reconciliations.id IS NULL AND DATE(dt) <= DATE($1)`,
					[dt]
				);
			} else {
				unreconciledStatementLines = await session.select(
					`SELECT statement_lines.* FROM statement_lines
					LEFT JOIN statement_line_reconciliations ON statement_lines.id = statement_line_reconciliations.statement_line_id
					WHERE statement_line_reconciliations.id IS NULL`
				);
			}
			
			const transactions = [];
			for (const line of unreconciledStatementLines) {
				const unclassifiedAccount = line.quantity >= 0 ? 'Unclassified Statement Line Debits' : 'Unclassified Statement Line Credits';
				transactions.push(new Transaction(
					null,
					line.dt,
					line.description,
					[
						{
							id: null,
							description: null,
							account: line.source_account,
							quantity: line.quantity,
							commodity: line.commodity
						},
						{
							id: null,
							description: null,
							account: unclassifiedAccount,
							quantity: -line.quantity,
							commodity: line.commodity
						}
					]
				));
			}
			this.transactionsForStage.set(ReportingStage.OrdinaryAPITransactions, transactions);
			
			// Recompute balances
			balances = applyTransactionsToBalances(balances, transactions);
			this.reportsForStage.set(ReportingStage.OrdinaryAPITransactions, [new TrialBalanceReport(balances)]);
		}
	}
	
	getReportAtStage(stage: ReportingStage, reportType: any): Report {
		// TODO: This function needs generics
		const reportsForTheStage = this.reportsForStage.get(stage);
		if (!reportsForTheStage) {
			throw new Error('Attempt to get report for unavailable stage');
		}
		
		const report = reportsForTheStage.find((r) => r instanceof reportType);
		if (!report) {
			throw new Error('Report does not exist');
		}
		
		return report;
	}
	
	getTransactionsAtStage(stage: ReportingStage): Transaction[] {
		const transactions: Transaction[] = [];
		for (const [curStage, curTransactions] of this.transactionsForStage.entries()) {
			if (curStage <= stage) {
				transactions.push(...curTransactions);
			}
		}
		return transactions;
	}
}

interface Report {
}

export class TrialBalanceReport implements Report {
	constructor(
		public balances: Map<string, number>
	) {}
}

function applyTransactionsToBalances(balances: Map<string, number>, transactions: Transaction[]): Map<string, number> {
	// Initialise new balances
	const newBalances: Map<string, number> = new Map([...balances.entries()]);
	
	// Apply transactions
	for (const transaction of transactions) {
		for (const posting of transaction.postings) {
			const openingBalance = newBalances.get(posting.account) ?? 0;
			const quantityCost = asCost(posting.quantity, posting.commodity);
			const runningBalance = openingBalance + quantityCost;
			
			newBalances.set(posting.account, runningBalance);
		}
	}
	
	// Sort accounts
	return new Map([...newBalances.entries()].sort((a, b) => a[0].localeCompare(b[0])));
}
