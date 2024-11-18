<!--
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
-->

<template>
	<h1 class="page-heading">
		Statement lines
	</h1>
	
	<div class="my-2 py-2 flex bg-white sticky top-0">
		<div class="grow flex gap-x-2 items-baseline">
			<!--<button class="btn-secondary text-emerald-700 ring-emerald-600">
				Reconcile selected as transfer
			</button>-->
			<RouterLink :to="{ name: 'import-statement' }" class="btn-secondary">
				Import statement
			</RouterLink>
			<!--<a href="{{ url_for('statement_lines', **dict(request.args, unclassified=1)) }}" class="btn-secondary">
				Show only unclassified lines
			</a>-->
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
	
	import { PencilIcon } from '@heroicons/vue/24/outline';
	
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
	
	const statementLines = ref([] as StatementLine[]);
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		
		const joinedStatementLines: any[] = await session.select(
			`SELECT statement_lines.id, source_account, statement_lines.dt, statement_lines.description, statement_lines.quantity, statement_lines.balance, statement_lines.commodity, p2.transaction_id, p2.account AS posting_account
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
	
	function renderTable() {
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
		for (const line of statementLines.value) {
			let reconciliationCell;
			if (line.posting_accounts.length === 0) {
				// Unreconciled
				reconciliationCell =
					`<a href="#" class="text-red-500 hover:text-red-600 hover:underline" onclick="return classifyLine(this);">Unclassified</a>
					<a href="/journal/edit-transaction/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			} else if (line.posting_accounts.length === 2) {
				// Simple reconciliation
				const otherAccount = line.posting_accounts.find((a) => a !== line.source_account);
				reconciliationCell =
					`<a href="#" class="hover:text-blue-700 hover:underline" onclick="return classifyLine(this);">${ otherAccount }</a>
					<a href="/journal/edit-transaction/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			} else {
				// Complex reconciliation
				reconciliationCell =
					`<i>(Complex)</i>
					<a href="/journal/edit-transaction/${ line.transaction_id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			}
			
			rows.push(
				`<tr data-line-id="{{ line.id }}">
					<td class="py-0.5 pr-1 align-baseline"><input class="checkbox-primary" type="checkbox" name="sel-line-id" value="${ line.id }"></td>
					<td class="py-0.5 px-1 align-baseline text-gray-900"><a href="#" class="hover:text-blue-700 hover:underline">${ line.source_account }</a></td>
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
				contentElem: document.querySelector('#statement-line-list tbody')!
			});
		} else {
			clusterize.update(rows);
		}
	}
	
	watch(statementLines, renderTable);
	
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
