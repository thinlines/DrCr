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

import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';

import App from './App.vue';

import { db } from './db.ts';

async function initApp() {
	// Init router
	const routes = [
		{ path: '/', name: 'index', component: () => import('./HomeView.vue') },
		{ path: '/general-ledger', name: 'general-ledger', component: () => import('./GeneralLedgerView.vue') },
		{ path: '/transactions/:account', name: 'transactions', component: () => import('./TransactionsView.vue') },
		{ path: '/trial-balance', name: 'trial-balance', component: () => import('./TrialBalanceView.vue') },
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

initApp();
