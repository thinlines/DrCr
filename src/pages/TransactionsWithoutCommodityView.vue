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
	<div id="transaction-list" class="max-h-[100vh] overflow-y-scroll wk-aa">
		<table class="min-w-full">
			<thead>
				<tr>
					<th class="py-0.5 pr-1 text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Related Account</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Dr</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Cr</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Balance</th>
					<th></th>
				</tr>
			</thead>
			<tbody class="min-w-full">
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
	
	import { asCost } from '../amounts.ts';
	import { Transaction } from '../db.ts';
	import { pp } from '../display.ts';
	import { renderComponent } from '../webutil.ts';
	
	const route = useRoute();
	const { transactions } = defineProps<{ transactions: Transaction[] }>();
	
	let clusterize: Clusterize | null = null;
	
	function renderTable() {
		// Recompute running balances
		// This is necessary because running_balance is cached only considering database transactions
		let balance = 0;
		for (let i = transactions.length - 1; i >= 0; i--) {
			const transaction = transactions[i];
			for (const posting of transaction.postings) {
				if (posting.account === route.params.account) {
					balance += asCost(posting.quantity, posting.commodity);
					posting.running_balance = balance;  // We should absolutely not commit this to the database!
				}
			}
		}
		
		// Render table
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
		for (const transaction of transactions) {
			let editLink = '';
			if (transaction.id !== null) {
				editLink = `<a href="/journal/edit-transaction/${ transaction.id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			}
			
			if (transaction.postings.length == 2) {
				// Simple transaction
				let thisAccountPosting, otherAccountPosting;
				
				for (const posting of transaction.postings) {
					if (posting.account === route.params.account) {
						thisAccountPosting = posting;
					} else {
						otherAccountPosting = posting;
					}
				}
				
				rows.push(
					`<tr class="border-t border-gray-300">
						<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ dayjs(transaction.dt).format('YYYY-MM-DD') }</td>
						<td class="py-0.5 px-1 text-gray-900">${ transaction.description } ${ editLink }</td>
						<td class="py-0.5 px-1 text-gray-900"><a href="/transactions/${ encodeURIComponent(otherAccountPosting!.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ otherAccountPosting!.account }</a></td>
						<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ thisAccountPosting!.quantity >= 0 ? pp(asCost(thisAccountPosting!.quantity, thisAccountPosting!.commodity)) : '' }</td>
						<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ thisAccountPosting!.quantity < 0 ? pp(asCost(-thisAccountPosting!.quantity, thisAccountPosting!.commodity)) : '' }</td>
						<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ pp(Math.abs(thisAccountPosting!.running_balance!)) }</td>
						<td class="py-0.5 text-gray-900">${ thisAccountPosting!.running_balance! >= 0 ? 'Dr' : 'Cr' }</td>
					</tr>`
				);
			} else {
				// Complex transaction
				rows.push(
					`<tr class="border-t border-gray-300">
						<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ dayjs(transaction.dt).format('YYYY-MM-DD') }</td>
						<td colspan="2" class="py-0.5 px-1 text-gray-900">${ transaction.description } ${ editLink }</td>
						<td></td>
						<td></td>
						<td></td>
						<td></td>
					</tr>`
				)
				
				for (const posting of transaction.postings) {
					rows.push(
						`<tr>
							<td></td>
							<td class="py-0.5 px-1 text-gray-900 text-end"><i>${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</i></td>
							<td class="py-0.5 px-1 text-gray-900"><a href="/transactions/${ encodeURIComponent(posting.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ posting.account }</a></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ posting.quantity >= 0 ? pp(asCost(posting.quantity, posting.commodity)) : '' }</td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ posting.quantity < 0 ? pp(asCost(-posting.quantity, posting.commodity)) : '' }</td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">${ posting.account === route.params.account ? pp(Math.abs(posting.running_balance!)) : '' }</td>
							<td class="py-0.5 text-gray-900">${ posting.account === route.params.account ? (posting.running_balance! >= 0 ? 'Dr' : 'Cr') : '' }</td>
						</tr>`
					)
				}
			}
		}
		
		if (clusterize === null) {
			clusterize = new Clusterize({
				'rows': rows,
				scrollElem: document.getElementById('transaction-list')!,
				contentElem: document.querySelector('#transaction-list tbody')!
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
