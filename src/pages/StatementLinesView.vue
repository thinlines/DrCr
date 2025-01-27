<!--
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
-->

<template>
	<h1 class="page-heading">
		Statement lines
	</h1>
	
	<div class="my-2 py-2 flex bg-white sticky top-0">
		<div class="grow flex gap-x-2 items-baseline">
			<button @click="reconcileAsTransfer" class="btn-secondary text-emerald-700 ring-emerald-600">
				Reconcile selected as transfer
			</button>
			<RouterLink :to="{ name: 'import-statement' }" class="btn-secondary">
				Import statement
			</RouterLink>
			<div class="flex items-baseline">
				<input id="only-unclassified" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="showOnlyUnclassified">
				<label for="only-unclassified" class="text-gray-900">Show only unclassified lines</label>
			</div>
		</div>
	</div>
	
	<div id="statement-line-list" class="max-h-[100vh] overflow-y-scroll wk-aa">
		<table class="min-w-full">
			<thead>
				<tr class="border-b border-gray-300">
					<th></th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Source account</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Description</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold text-start">Charged to</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-end">Dr</th>
					<th class="py-0.5 px-1 align-bottom text-gray-900 font-semibold lg:w-[12ex] text-end">Cr</th>
					<th class="py-0.5 pl-1 align-bottom text-gray-900 font-semibold text-end">Balance</th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td></td>
					<td class="py-0.5 px-1" colspan="7">Loading data…</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import dayjs from 'dayjs';
	
	import { CheckIcon, PencilIcon } from '@heroicons/vue/24/outline';
	
	import { onUnmounted, ref, watch } from 'vue';
	
	import { db } from '../db.ts';
	import { renderComponent } from '../webutil.ts';
	import { ppWithCommodity } from '../display.ts';
	
	interface StatementLine {
		id: number,
		source_account: string,
		dt: string,
		description: string,
		quantity: number,
		balance: number | null,
		commodity: string,
		transaction_id: number,
		posting_accounts: string[]
	}
	
	const showOnlyUnclassified = ref(false);
	const statementLines = ref([] as StatementLine[]);
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		
		const joinedStatementLines: any[] = await session.select(
			`SELECT statement_lines.*, p2.transaction_id, p2.account AS posting_account
			FROM statement_lines
			LEFT JOIN statement_line_reconciliations ON statement_lines.id = statement_line_reconciliations.statement_line_id
			LEFT JOIN postings ON statement_line_reconciliations.posting_id = postings.id
			LEFT JOIN transactions ON postings.transaction_id = transactions.id
			LEFT JOIN postings p2 ON transactions.id = p2.transaction_id
			ORDER BY statement_lines.dt DESC, statement_lines.id DESC, p2.id`
		);
		
		// Unflatten statement lines
		const newStatementLines: StatementLine[] = [];
		
		for (const joinedStatementLine of joinedStatementLines) {
			if (newStatementLines.length === 0 || newStatementLines.at(-1)!.id !== joinedStatementLine.id) {
				newStatementLines.push({
					id: joinedStatementLine.id,
					source_account: joinedStatementLine.source_account,
					dt: joinedStatementLine.dt,
					description: joinedStatementLine.description,
					quantity: joinedStatementLine.quantity,
					balance: joinedStatementLine.balance,
					commodity: joinedStatementLine.commodity,
					transaction_id: joinedStatementLine.transaction_id,
					posting_accounts: []
				});
			}
			if (joinedStatementLine.posting_account !== null) {
				newStatementLines.at(-1)!.posting_accounts.push(joinedStatementLine.posting_account);
			}
		}
		
		statementLines.value = newStatementLines;
	}
	
	// TODO: Could probably avoid polluting global scope by using clusterize clusterChanged callback
	(window as any).showClassifyLinePanel = function(el: HTMLAnchorElement) {
		const CheckIconHTML = renderComponent(CheckIcon, { 'class': 'w-5 h-5' });
		
		const td = el.closest('td')!;
		td.className = 'relative';  // CSS trickery so as to not expand the height of the tr
		td.innerHTML =
			`<div class="flex items-stretch absolute top-[-4px]">
				<input type="text" class="bordered-field min-w-[8em]">
				<button type="button" class="relative -ml-px inline-flex items-center gap-x-1.5 px-3 py-1 text-gray-800 shadow-sm ring-1 ring-inset ring-gray-400 bg-white hover:bg-gray-50">${ CheckIconHTML }</button>
			</div>`;
		
		td.querySelector('input')!.addEventListener('keydown', async function(event: KeyboardEvent) {
			if (event.key === 'Enter') {
				await onLineClassified(event);
			}
		})
		td.querySelector('button')!.addEventListener('click', onLineClassified);
		
		td.querySelector('input')!.focus();
		
		return false;
	};
	
	async function onLineClassified(event: Event) {
		// Callback when clicking OK or pressing enter to classify a statement line
		if ((event.target as HTMLInputElement).disabled) {
			return;
		}
		
		const td = (event.target as Element).closest('td')!;
		const tr = td.closest('tr')!;
		const lineId = parseInt(tr.dataset.lineId!);
		const chargeAccount = (td.querySelector('input')! as HTMLInputElement).value;
		
		if (!chargeAccount) {
			return;
		}
		
		// Disable further submissions
		td.querySelector('input')!.disabled = true;
		td.querySelector('button')!.disabled = true;
		
		const statementLine = statementLines.value.find((l) => l.id === lineId)!;
		
		if (statementLine.posting_accounts.length !== 0) {
			await alert('Cannot reconcile already reconciled statement line');
			return;
		}
		
		// Check if account exists
		const session = await db.load();
		const countResult = await session.select('SELECT COUNT(*) FROM postings WHERE account = $1', [chargeAccount]) as any[];
		const doesAccountExist = countResult[0]['COUNT(*)'] > 0;
		if (!doesAccountExist) {
			// Prompt for confirmation
			if (!await confirm('Account "' + chargeAccount + '" does not exist. Continue to reconcile this transaction and create a new account?')) {
				td.querySelector('input')!.disabled = false;
				td.querySelector('button')!.disabled = false;
				return;
			}
		}
		
		// Insert transaction and statement line reconciliation atomically
		const dbTransaction = await session.begin();
		
		// Insert transaction
		const transactionResult = await dbTransaction.execute(
			`INSERT INTO transactions (dt, description)
			VALUES ($1, $2)`,
			[statementLine.dt, statementLine.description]
		);
		const transactionId = transactionResult.lastInsertId;
		
		// Insert posting for this account
		const accountPostingResult = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity, running_balance)
			VALUES ($1, NULL, $2, $3, $4, NULL)`,
			[transactionId, statementLine.source_account, statementLine.quantity, statementLine.commodity]
		);
		const accountPostingId = accountPostingResult.lastInsertId;
		
		// Insert posting for the charge account - no need to remember this ID
		await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity, running_balance)
			VALUES ($1, NULL, $2, $3, $4, NULL)`,
			[transactionId, chargeAccount, -statementLine.quantity, statementLine.commodity]
		);
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[statementLine.id, accountPostingId]
		);
		
		// Invalidate running balances
		await dbTransaction.execute(
			`UPDATE postings
			SET running_balance = NULL
			FROM (
				SELECT postings.id
				FROM transactions
				JOIN postings ON transactions.id = postings.transaction_id
				WHERE DATE(dt) >= DATE($1) AND account IN ($2, $3)
			) p
			WHERE postings.id = p.id`,
			[statementLine.dt, statementLine.source_account, chargeAccount]
		);
		
		dbTransaction.commit();
		
		// Reload transactions and re-render the table
		await load();
	}
	
	async function reconcileAsTransfer() {
		const selectedCheckboxes = document.querySelectorAll('.statement-line-checkbox:checked');
		
		if (selectedCheckboxes.length !== 2) {
			await alert('Must select exactly 2 statement lines');
			return;
		}
		
		const selectedLineIds = [...selectedCheckboxes].map((el) => parseInt(el.closest('tr')?.dataset.lineId!));
		
		const line1 = statementLines.value.find((l) => l.id === selectedLineIds[0])!;
		const line2 = statementLines.value.find((l) => l.id === selectedLineIds[1])!;
		
		// Sanity checks
		if (line1.quantity + line2.quantity !== 0 || line1.commodity !== line2.commodity) {
			await alert('Selected statement line debits/credits must equal');
			return;
		}
		if (line1.posting_accounts.length !== 0 || line2.posting_accounts.length !== 0) {
			await alert('Cannot reconcile already reconciled statement lines');
			return;
		}
		
		// Insert transaction and statement line reconciliation atomically
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		// Insert transaction
		const transactionResult = await dbTransaction.execute(
			`INSERT INTO transactions (dt, description)
			VALUES ($1, $2)`,
			[line1.dt, line1.description]
		);
		const transactionId = transactionResult.lastInsertId;
		
		// Insert posting for line1
		const postingResult1 = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity, running_balance)
			VALUES ($1, $2, $3, $4, $5, NULL)`,
			[transactionId, line1.description, line1.source_account, line1.quantity, line1.commodity]
		);
		const postingId1 = postingResult1.lastInsertId;
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[line1.id, postingId1]
		);
		
		// Insert posting for line2
		const postingResult2 = await dbTransaction.execute(
			`INSERT INTO postings (transaction_id, description, account, quantity, commodity, running_balance)
			VALUES ($1, $2, $3, $4, $5, NULL)`,
			[transactionId, line2.description, line2.source_account, line2.quantity, line2.commodity]
		);
		const postingId2 = postingResult2.lastInsertId;
		
		// Insert statement line reconciliation
		await dbTransaction.execute(
			`INSERT INTO statement_line_reconciliations (statement_line_id, posting_id)
			VALUES ($1, $2)`,
			[line2.id, postingId2]
		);
		
		// Invalidate running balances
		await dbTransaction.execute(
			`UPDATE postings
			SET running_balance = NULL
			FROM (
				SELECT postings.id
				FROM transactions
				JOIN postings ON transactions.id = postings.transaction_id
				WHERE DATE(dt) >= DATE($1) AND account IN ($2, $3)
			) p
			WHERE postings.id = p.id`,
			[line1.dt, line1.source_account, line2.source_account]
		);
		
		dbTransaction.commit();
		
		// Reload transactions and re-render the table
		await load();
	}
	
	function renderTable() {
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
		for (const line of statementLines.value) {
			let reconciliationCell, checkboxCell;
			if (line.posting_accounts.length === 0) {
				// Unreconciled
				reconciliationCell =
					`<a href="#" class="classify-link text-red-500 hover:text-red-600 hover:underline" onclick="return showClassifyLinePanel(this);">Unclassified</a>`;
				checkboxCell = `<input class="checkbox-primary statement-line-checkbox" type="checkbox">`;  // Only show checkbox for unreconciled lines
			} else if (line.posting_accounts.length === 2) {
				// Simple reconciliation
				const otherAccount = line.posting_accounts.find((a) => a !== line.source_account);
				reconciliationCell =
					`<span>${ otherAccount }</span>
					<a href="/journal/edit/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
				checkboxCell = '';
				
				if (showOnlyUnclassified.value) { continue; }
			} else {
				// Complex reconciliation
				reconciliationCell =
					`<i>(Complex)</i>
					<a href="/journal/edit/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
				checkboxCell = '';
				
				if (showOnlyUnclassified.value) { continue; }
			}
			
			rows.push(
				`<tr data-line-id="${ line.id }">
					<td class="py-0.5 pr-1 align-baseline">${ checkboxCell }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900"><a href="/transactions/${ encodeURIComponent(line.source_account) }" class="hover:text-blue-700 hover:underline">${ line.source_account }</a></td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex]">${ dayjs(line.dt).format('YYYY-MM-DD') }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900">${ line.description }</td>
					<td class="charge-account py-0.5 px-1 align-baseline text-gray-900">${ reconciliationCell }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex] text-end">${ line.quantity >= 0 ? ppWithCommodity(line.quantity, line.commodity) : '' }</td>
					<td class="py-0.5 px-1 align-baseline text-gray-900 lg:w-[12ex] text-end">${ line.quantity < 0 ? ppWithCommodity(-line.quantity, line.commodity) : '' }</td>
					<td class="py-0.5 pl-1 align-baseline text-gray-900 text-end">${ line.balance ?? '' }</td>
				</tr>`
			)
		}
		
		if (clusterize === null) {
			clusterize = new Clusterize({
				'rows': rows,
				scrollElem: document.getElementById('statement-line-list')!,
				contentElem: document.querySelector('#statement-line-list tbody')!,
				show_no_data_row: false,
			});
		} else {
			clusterize.update(rows);
		}
	}
	
	watch(showOnlyUnclassified, renderTable);
	watch(statementLines, renderTable);
	
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
