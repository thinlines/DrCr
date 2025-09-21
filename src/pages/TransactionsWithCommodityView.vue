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
    <div id="transaction-list" class="h-full overflow-y-auto wk-aa">
        <table class="min-w-full sticky-table">
			<thead class="sticky-header">
				<tr>
					<th class="py-0.5 pr-1 text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
					<th></th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Amount</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Balance</th>
					<th></th>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td colspan="7">Loading data…</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import dayjs from 'dayjs';
	
	import { PencilIcon } from '@heroicons/vue/24/outline';
	
	import { onMounted, onUnmounted, watch } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { Balance } from '../amounts.ts';
	import { Transaction } from '../db.ts';
	import { ppWithCommodity } from '../display.ts';
	import { renderComponent } from '../webutil.ts';
	
	const route = useRoute();
	const { transactions } = defineProps<{ transactions: Transaction[] }>();
	
	const runningBalances = new Map();
	let clusterize: Clusterize | null = null;
	
	function renderTable() {
		// Compute running balances
		// Transactions are returned from DB in reverse order, so we iterate backwards
		const runningBalance = new Balance();
		for (let i = transactions.length - 1; i >= 0; i--) {
			const transaction = transactions[i];
			for (const posting of transaction.postings) {
				if (posting.account === route.params.account) {
					runningBalance.add(posting.quantity, posting.commodity);
				}
			}
			
			// Save the running balance to display in the report
			runningBalances.set(transaction, runningBalance.clone());
			
			runningBalance.clean();
		}
		
		// Render table
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
		for (const transaction of transactions) {
			let editLink = '';
			if (transaction.id !== null) {
				editLink = `<a href="/journal/edit/${ transaction.id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			}
			rows.push(
				`<tr class="border-t border-gray-300">
					<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ dayjs(transaction.dt).format('YYYY-MM-DD') }</td>
					<td class="py-0.5 px-1 text-gray-900">${ transaction.description } ${ editLink }</td>
					<td></td>
					<td></td>
					<td></td>
					<td></td>
				</tr>`
			);
			
			const balanceAmounts = runningBalances.get(transaction).amounts;
			
			// Display balance newest entries at the top
			for (let i = balanceAmounts.length - 1; i >= 0; i--) {
				const amount = balanceAmounts[i];
				
				// Match the amount to the posting(s) in this transaction, if any
				const postingsThisCommodity = transaction.postings.filter((p) => p.commodity === amount.commodity && p.account === route.params.account);
				if (postingsThisCommodity.length === 0) {
					// Just display the balance
					rows.push(
						`<tr>
							<td></td>
							<td></td>
							<td></td>
							<td></td>
							<td class="py-0.5 px-1 text-gray-900 text-end">${ ppWithCommodity(Math.abs(amount.quantity), amount.commodity) }</td>
							<td class="py-0.5 text-gray-900">${ amount.quantity >= 0 ? 'Dr' : 'Cr' }</td>
						</tr>`
					);
				} else {
					// Display all postings - display the balance only at the last posting
					for (let i = 0; i < postingsThisCommodity.length; i++) {
						const posting = postingsThisCommodity[i];
						rows.push(
							`<tr>
								<td></td>
								<td></td>
								<td class="py-0.5 px-1 text-gray-900 text-end">${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</td>
								<td class="py-0.5 px-1 text-gray-900 text-end">${ ppWithCommodity(Math.abs(posting.quantity), posting.commodity) }</td>
								<td class="py-0.5 px-1 text-gray-900 text-end">${ i === postingsThisCommodity.length - 1 ? ppWithCommodity(Math.abs(amount.quantity), amount.commodity) : '' }</td>
								<td class="py-0.5 text-gray-900">${ i === postingsThisCommodity.length - 1 ? (amount.quantity >= 0 ? 'Dr' : 'Cr') : '' }</td>
							</tr>`
						);
					}
				}
			}
		}
		
		if (clusterize === null) {
			clusterize = new Clusterize({
				'rows': rows,
				scrollElem: document.getElementById('transaction-list')!,
				contentElem: document.querySelector('#transaction-list tbody')!,
				show_no_data_row: false,
			});
		} else {
			clusterize.update(rows);
		}
	}
	
	onMounted(renderTable);
	watch(() => transactions, renderTable);
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
