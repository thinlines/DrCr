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
		<button v-if="commodityDetail" class="btn-secondary" @click="commodityDetail = false">Hide commodity detail</button>
		<button v-if="!commodityDetail" class="btn-secondary" @click="commodityDetail = true">Show commodity detail</button>
	</div>
	
	<table class="min-w-full" ref="table">
		<thead>
			<tr>
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start" colspan="3">Description</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
				<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Cr</th>
			</tr>
		</thead>
		<tbody id="transaction-list">
			<template v-for="transaction in transactions" :key="transaction.id">
				<tr class="border-t border-gray-300">
					<td class="py-0.5 pr-1 text-gray-900">{{ transaction.dt.split(' ')[0] }}</td>
					<td class="py-0.5 px-1 text-gray-900" colspan="3">{{ transaction.description }}</td>
					<td></td>
					<td></td>
				</tr>
				<template v-for="posting in transaction.postings" :key="posting.id">
					<tr>
						<td></td>
						<td class="py-0.5 px-1 text-gray-900">{{ posting.description }}</td>
						<td class="py-0.5 px-1 text-gray-900 text-end"><i>{{ posting.quantity >= 0 ? 'Dr' : 'Cr' }}</i></td>
						<td class="py-0.5 px-1 text-gray-900">{{ posting.account }}</td>
						<td class="py-0.5 px-1 text-gray-900 text-end">
							{{ posting.quantity >= 0 ? (commodityDetail ? ppWithCommodity(posting.quantity, posting.commodity) : pp(asCost(posting.quantity, posting.commodity))) : '' }}
						</td>
						<td class="py-0.5 pl-1 text-gray-900 text-end">
							{{ posting.quantity < 0 ? (commodityDetail ? ppWithCommodity(-posting.quantity, posting.commodity) : pp(asCost(-posting.quantity, posting.commodity))) : '' }}
						</td>
					</tr>
				</template>
			</template>
		</tbody>
	</table>
	
	<div class="my-4 flex" v-if="transactionsOffset !== null">
		<button class="btn-secondary" @click="load()">Load more…</button>
	</div>
</template>

<script setup lang="ts">
	import { onMounted, onUnmounted, ref, useTemplateRef } from 'vue';
	
	import { asCost } from './commodities.ts';
	import { db } from './db.ts';
	import { pp, ppWithCommodity } from './display.ts';
	
	const commodityDetail = ref(false);
	
	interface _Transaction {
		id: number,
		dt: string,
		description: string,
		postings: _Posting[]
	}
	
	interface _Posting {
		id: number,
		description: string,
		account: string,
		quantity: number,
		commodity: string
	}
	
	const transactions = ref([] as _Transaction[]);
	const transactionsOffset = ref(0 as number | null);
	
	async function load() {
		if (transactionsOffset.value === null) {
			// No more entries
			return;
		}
		
		const session = await db.load();
		
		const transactionsRaw: {transaction_id: number, dt: string, transaction_description: string, id: number, description: string, account: string, quantity: number, commodity: string}[] = await session.select('SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity FROM transactions LEFT JOIN postings ON transactions.id = postings.transaction_id ORDER BY dt DESC, transaction_id DESC, postings.id LIMIT 200 OFFSET ?', [transactionsOffset.value]);
		
		if (transactionsRaw.length === 0) {
			// No more entries
			transactionsOffset.value = null;
			return;
		}
		
		// Group postings into transactions
		for (const transactionRaw of transactionsRaw) {
			if (transactions.value.length === 0 || transactions.value.at(-1)!.id !== transactionRaw.transaction_id) {
				transactions.value.push({
					id: transactionRaw.transaction_id,
					dt: transactionRaw.dt,
					description: transactionRaw.transaction_description,
					postings: []
				});
			}
			
			transactions.value.at(-1)!.postings.push({
				id: transactionRaw.id,
				description: transactionRaw.description,
				account: transactionRaw.account,
				quantity: transactionRaw.quantity,
				commodity: transactionRaw.commodity
			});
		}
		
		transactionsOffset.value += transactionsRaw.length;
	}
	load();
</script>
