/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2025  Lee Yingtong Li (RunasSudo)
	
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

import dayjs from 'dayjs';

import { asCost } from './amounts.ts';
import { DT_FORMAT, JoinedTransactionPosting, StatementLine, Transaction, db, getAccountsForKind, joinedToTransactions, totalBalances, totalBalancesAtDate } from './db.ts';
import { ExtendedDatabase } from './dbutil.ts';

import { BalanceSheetReport } from './reports/BalanceSheetReport.vue';
import { DrcrReport } from './reports/base.ts';
import TrialBalanceReport from './reports/TrialBalanceReport.ts';
import { IncomeStatementReport } from './reports/IncomeStatementReport.vue';

export enum ReportingStage {
	// Load transactions from database
	TransactionsFromDatabase = 100,
	
	// Load unreconciled statement lines and other ordinary API transactions
	OrdinaryAPITransactions = 200,
	
	// Recognise accumulated surplus as equity
	AccumulatedSurplusToEquity = 300,
	
	// Interim income statement considering only DB and ordinary API transactions
	InterimIncomeStatement = 400,
	
	// Income tax estimation
	//Tax = 500,
	
	// Final income statement
	//IncomeStatement = 600,
	
	// Final balance sheet
	BalanceSheet = 700,
	
	FINAL_STAGE = BalanceSheet
}

export class ReportingWorkflow {
	transactionsForStage: Map<ReportingStage, Transaction[]> = new Map();
	reportsForStage: Map<ReportingStage, DrcrReport[]> = new Map();
	
	async generate(session: ExtendedDatabase, dt?: string, dtStart?: string) {
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
					`SELECT transaction_id, dt, transaction_description, id, description, account, quantity, commodity, running_balance
					FROM transactions_with_running_balances
					WHERE DATE(dt) <= DATE($1)
					ORDER BY dt, transaction_id, id`,
					[dt]
				);
			} else {
				joinedTransactionPostings = await session.select(
					`SELECT transaction_id, dt, transaction_description, id, description, account, quantity, commodity, running_balance
					FROM transactions_with_running_balances
					ORDER BY dt, transaction_id, id`
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
		
		// --------------------------
		// AccumulatedSurplusToEquity
		
		{
			// Compute balances at period start for TransactionsFromDatabase
			let dayBeforePeriodStart;
			if (dtStart) {
				dayBeforePeriodStart = dayjs(dtStart).subtract(1, 'day');
			} else {
				dayBeforePeriodStart = dayjs(db.metadata.eofy_date).subtract(1, 'year');
			}
			const balancesAtPeriodStart = await totalBalancesAtDate(session, dayBeforePeriodStart.format('YYYY-MM-DD'));
			
			// Add balances at period start for OrdinaryAPITransactions
			for (const transaction of this.transactionsForStage.get(ReportingStage.OrdinaryAPITransactions)!) {
				if (!dayjs(transaction.dt).isAfter(dayBeforePeriodStart)) {
					for (const posting of transaction.postings) {
						balancesAtPeriodStart.set(posting.account, (balancesAtPeriodStart.get(posting.account) ?? 0) + asCost(posting.quantity, posting.commodity));
					}
				}
			}
			
			// Get income and expense accounts
			const incomeAccounts = await getAccountsForKind(session, 'drcr.income');
			const expenseAccounts = await getAccountsForKind(session, 'drcr.expense');
			const pandlAccounts = [...incomeAccounts, ...expenseAccounts];
			pandlAccounts.sort();
			
			// Prepare transactions
			const transactions = [];
			for (const account of pandlAccounts) {
				if (balancesAtPeriodStart.has(account)) {
					const balanceAtPeriodStart = balancesAtPeriodStart.get(account)!;
					if (balanceAtPeriodStart === 0) {
						continue;
					}
					
					transactions.push(new Transaction(
						null,
						dayBeforePeriodStart.format(DT_FORMAT),
						'Accumulated surplus/deficit',
						[
							{
								id: null,
								description: null,
								account: account,
								quantity: -balanceAtPeriodStart,
								commodity: db.metadata.reporting_commodity
							},
							{
								id: null,
								description: null,
								account: 'Accumulated surplus (deficit)',
								quantity: balanceAtPeriodStart,
								commodity: db.metadata.reporting_commodity
							},
						]
					));
				}
			}
			this.transactionsForStage.set(ReportingStage.AccumulatedSurplusToEquity, transactions);
			
			// Recompute balances
			balances = applyTransactionsToBalances(balances, transactions);
			this.reportsForStage.set(ReportingStage.AccumulatedSurplusToEquity, [new TrialBalanceReport(balances)]);
		}
		
		// ---------------
		// InterimIncomeStatement
		
		let incomeStatementReport;
		{
			incomeStatementReport = new IncomeStatementReport();
			await incomeStatementReport.generate(balances);
			this.reportsForStage.set(ReportingStage.InterimIncomeStatement, [incomeStatementReport]);
		}
		
		// ------------
		// BalanceSheet
		
		{
			const balanceSheetReport = new BalanceSheetReport();
			await balanceSheetReport.generate(balances, incomeStatementReport);
			this.reportsForStage.set(ReportingStage.BalanceSheet, [balanceSheetReport]);
		}
	}
	
	getReportAtStage(stage: ReportingStage, reportType: any): DrcrReport {
		// TODO: This function needs generics
		const reportsForTheStage = this.reportsForStage.get(stage);
		if (!reportsForTheStage) {
			throw new Error('Attempt to get report for unavailable stage');
		}
		
		const report = reportsForTheStage.find((r) => r instanceof reportType);
		if (report) {
			return report;
		}
		
		// Recurse earlier stages
		const stages = [...this.reportsForStage.keys()];
		stages.reverse();
		for (const earlierStage of stages) {
			if (earlierStage >= stage) {
				continue;
			}
			const report = this.reportsForStage.get(earlierStage)!.find((r) => r instanceof reportType);
			if (report) {
				return report;
			}
		}
		
		throw new Error('Report does not exist at requested stage or any earlier stage');
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
