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
		General ledger
	</h1>
	
	<div class="my-4 flex">
		<button v-if="commodityDetail" class="btn-secondary" @click="commodityDetail = false; renderTable();">Hide commodity detail</button>
		<button v-if="!commodityDetail" class="btn-secondary" @click="commodityDetail = true; renderTable();">Show commodity detail</button>
	</div>
	
	<div id="transaction-list" class="max-h-[100vh] overflow-y-scroll wk-aa">
		<table class="min-w-full">
			<thead>
				<tr>
					<th class="py-0.5 pr-1 text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start" colspan="3">Description</th>
					<template v-if="commodityDetail">
						<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
						<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Cr</th>
					</template>
					<template v-if="!commodityDetail">
						<th class="py-0.5 px-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Dr</th>
						<th class="py-0.5 pl-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Cr</th>
					</template>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td colspan="4">Loading data…</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import { onUnmounted, ref } from 'vue';
	
	import { asCost } from '../commodities.ts';
	import { JoinedTransactionPosting, Transaction, db, joinedToTransactions } from '../db.ts';
	import { pp, ppWithCommodity } from '../display.ts';
	
	const commodityDetail = ref(false);
	
	let transactions: Transaction[] = [];
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		
		const joinedTransactionPostings: JoinedTransactionPosting[] = await session.select(
			`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity
			FROM transactions
			JOIN postings ON transactions.id = postings.transaction_id
			ORDER BY dt DESC, transaction_id DESC, postings.id`
		);
		
		transactions = joinedToTransactions(joinedTransactionPostings);
		
		renderTable();
	}
	
	function renderTable() {
		const rows = [];
		
		for (const transaction of transactions) {
			rows.push(
				`<tr class="border-t border-gray-300">
					<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ transaction.dt.split(' ')[0] }</td>
					<td class="py-0.5 px-1 text-gray-900" colspan="3">${ transaction.description }</td>
					<td></td>
					<td></td>
				</tr>`
			);
			for (const posting of transaction.postings) {
				if (commodityDetail.value) {
					rows.push(
						`<tr>
							<td class=""></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]">${ posting.description ?? '' }</td>
							<td class="py-0.5 px-1 text-gray-900 text-end"><i>${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</i></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]"><a href="/transactions/${ encodeURIComponent(posting.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ posting.account }</a></td>
							<td class="py-0.5 px-1 text-gray-900 text-end">
								${ posting.quantity >= 0 ? ppWithCommodity(posting.quantity, posting.commodity) : '' }
							</td>
							<td class="py-0.5 pl-1 text-gray-900 text-end">
								${ posting.quantity < 0 ? ppWithCommodity(-posting.quantity, posting.commodity) : '' }
							</td>
						</tr>`
					);
				} else {
					rows.push(
						`<tr>
							<td class=""></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]">${ posting.description ?? '' }</td>
							<td class="py-0.5 px-1 text-gray-900 text-end"><i>${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</i></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]"><a href="/transactions/${ encodeURIComponent(posting.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ posting.account }</a></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">
								${ posting.quantity >= 0 ? pp(asCost(posting.quantity, posting.commodity)) : '' }
							</td>
							<td class="py-0.5 pl-1 text-gray-900 lg:w-[12ex] text-end">
								${ posting.quantity < 0 ? pp(asCost(-posting.quantity, posting.commodity)) : '' }
							</td>
						</tr>`
					);
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
	
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
