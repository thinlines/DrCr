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
		Account transactions
	</h1>
	
	<div class="my-4 flex gap-x-2">
		<!--<a href="{{ url_for('journal_new_transaction') }}" class="btn-primary pl-2">
			<PlusIcon />
			New transaction
		</a>
		<a href="{{ url_for('account_transactions', account=account, commodity_detail=1) }}" class="btn-secondary">
			Show commodity detail
		</a>-->
	</div>
	
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
	
	//import { PlusIcon } from '@heroicons/vue/24/solid';
	
	import { onUnmounted } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { asCost } from './commodities.ts';
	import { JoinedTransactionPosting, db, joinedToTransactions } from './db.ts';
	import { pp } from './display.ts';
	
	const route = useRoute();
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		
		const joinedTransactionPostings: JoinedTransactionPosting[] = await session.select(
			`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity, running_balance
			FROM transactions
			JOIN postings ON transactions.id = postings.transaction_id
			WHERE transactions.id IN (SELECT transaction_id FROM postings WHERE postings.account = $1)
			ORDER by dt DESC, transaction_id DESC, postings.id`,
			[route.params.account]
		);
		
		const transactions = joinedToTransactions(joinedTransactionPostings);
		
		// Render table
		const rows = [];
		
		for (const transaction of transactions) {
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
						<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ transaction.dt.split(' ')[0] }</td>
						<td class="py-0.5 px-1 text-gray-900">
							${ transaction.description }
							<!-- TODO: Edit button -->
						</td>
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
						<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ transaction.dt.split(' ')[0] }</td>
						<td colspan="2" class="py-0.5 px-1 text-gray-900">
							${ transaction.description }
							<!-- TODO: Edit button -->
						</td>
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
		
		clusterize = new Clusterize({
			'rows': rows,
			scrollElem: document.getElementById('transaction-list')!,
			contentElem: document.querySelector('#transaction-list tbody')!
		});
	}
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
