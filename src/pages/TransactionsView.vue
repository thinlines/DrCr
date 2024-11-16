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
		{{ route.params.account }}
	</h1>
	
	<div class="my-4 flex gap-x-2">
		<!--<a href="{{ url_for('journal_new_transaction') }}" class="btn-primary pl-2">
			<PlusIcon />
			New transaction
		</a>-->
		<button v-if="commodityDetail" class="btn-secondary" @click="commodityDetail = false">Hide commodity detail</button>
		<button v-if="!commodityDetail" class="btn-secondary" @click="commodityDetail = true">Show commodity detail</button>
	</div>
	
	<TransactionsWithCommodityView v-if="commodityDetail" :transactions="transactions"/>
	<TransactionsWithoutCommodityView v-if="!commodityDetail" :transactions="transactions"/>
</template>

<script setup lang="ts">
	//import { PlusIcon } from '@heroicons/vue/24/solid';
	
	import { ref } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { JoinedTransactionPosting, Transaction, db, joinedToTransactions, updateRunningBalances } from '../db.ts';
	import TransactionsWithCommodityView from './TransactionsWithCommodityView.vue';
	import TransactionsWithoutCommodityView from './TransactionsWithoutCommodityView.vue';
	
	const route = useRoute();
	
	const commodityDetail = ref(false);
	const transactions = ref([] as Transaction[]);
	
	async function load() {
		const session = await db.load();
		
		// Ensure running balances are up to date because we use these
		await updateRunningBalances();
		
		const joinedTransactionPostings: JoinedTransactionPosting[] = await session.select(
			`SELECT transaction_id, dt, transactions.description AS transaction_description, postings.id, postings.description, account, quantity, commodity, running_balance
			FROM transactions
			JOIN postings ON transactions.id = postings.transaction_id
			WHERE transactions.id IN (SELECT transaction_id FROM postings WHERE postings.account = $1)
			ORDER by dt DESC, transaction_id DESC, postings.id`,
			[route.params.account]
		);
		
		transactions.value = joinedToTransactions(joinedTransactionPostings);
	}
	load();
</script>
