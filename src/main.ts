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
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';

import App from './App.vue';

import { db } from './db.ts';

async function initApp() {
	// Init router
	const routes = [
		{ path: '/', name: 'index', component: () => import('./pages/HomeView.vue') },
		{ path: '/balance-assertions', name: 'balance-assertions', component: () => import('./pages/BalanceAssertionsView.vue') },
		{ path: '/balance-assertions/edit/:id', name: 'balance-assertions-edit', component: () => import('./pages/EditBalanceAssertionView.vue') },
		{ path: '/balance-assertions/new', name: 'balance-assertions-new', component: () => import('./pages/NewBalanceAssertionView.vue') },
		{ path: '/balance-sheet', name: 'balance-sheet', component: () => import('./reports/BalanceSheetReport.vue') },
		{ path: '/chart-of-accounts', name: 'chart-of-accounts', component: () => import('./pages/ChartOfAccountsView.vue') },
		{ path: '/general-ledger', name: 'general-ledger', component: () => import('./pages/GeneralLedgerView.vue') },
		{ path: '/income-statement', name: 'income-statement', component: () => import('./reports/IncomeStatementReport.vue') },
		{ path: '/journal', name: 'journal', component: () => import('./pages/JournalView.vue') },
		{ path: '/journal/edit/:id', name: 'journal-edit-transaction', component: () => import('./pages/EditTransactionView.vue') },
		{ path: '/journal/new', name: 'journal-new-transaction', component: () => import('./pages/NewTransactionView.vue') },
		{ path: '/statement-lines', name: 'statement-lines', component: () => import('./pages/StatementLinesView.vue') },
		{ path: '/statement-lines/import', name: 'import-statement', component: () => import('./pages/ImportStatementView.vue') },
		{ path: '/transactions/:account', name: 'transactions', component: () => import('./pages/TransactionsView.vue') },
		{ path: '/trial-balance', name: 'trial-balance', component: () => import('./reports/TrialBalanceReport.vue') },
		// TODO: Generate this list dynamically
		{ path: '/austax/cgt-adjustments', name: 'cgt-adjustments', component: () => import('./plugins/austax/CGTAdjustmentsView.vue') },
		{ path: '/austax/tax-summary', name: 'tax-summary', component: () => import('./plugins/austax/TaxSummaryReport.vue') },
	];
	const router = createRouter({
		history: createWebHistory(),
		routes,
	});
	
	// Init state
	const dbFilename: string = await invoke('get_open_filename');
	if (dbFilename !== null) {
		await db.init(dbFilename);  // Ensure all metadata cached before loading Vue
	}
	
	// Create Vue app
	createApp(App).use(router).mount('#app');
}

(window as any).openLinkInNewWindow = function(link: HTMLAnchorElement) {
	const webview = new WebviewWindow('dialog' + +new Date(), {
		url: link.href,
	});
	webview.once('tauri://error', function(e) {
		console.error(e);
	});
	return false;
}

initApp();
