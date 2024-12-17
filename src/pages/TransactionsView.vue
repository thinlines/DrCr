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
	
	<div class="my-4 flex gap-x-2 items-center">
		<!-- Use a rather than RouterLink because RouterLink adds its own event handler -->
		<a :href="$router.resolve({name: 'journal-new-transaction'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New transaction
		</a>
		<div class="flex items-baseline">
			<input id="only-unclassified" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="commodityDetail">
			<label for="only-unclassified" class="text-gray-900">Show commodity detail</label>
		</div>
	</div>
	
	<TransactionsWithCommodityView v-if="commodityDetail" :transactions="transactions"/>
	<TransactionsWithoutCommodityView v-if="!commodityDetail" :transactions="transactions"/>
</template>

<script setup lang="ts">
	import { PlusIcon } from '@heroicons/vue/24/outline';
	
	import { ref } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { Transaction, db } from '../db.ts';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	import TransactionsWithCommodityView from './TransactionsWithCommodityView.vue';
	import TransactionsWithoutCommodityView from './TransactionsWithoutCommodityView.vue';
	
	const route = useRoute();
	
	const commodityDetail = ref(false);
	const transactions = ref([] as Transaction[]);
	
	async function load() {
		const session = await db.load();
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session);  // This also ensures running balances are up to date
		
		const transactionsRaw = reportingWorkflow.getTransactionsAtStage(ReportingStage.FINAL_STAGE);
		
		// Filter only transactions affecting this account
		const filteredTransactions = transactionsRaw.filter((t) => t.postings.some((p) => p.account === route.params.account));
		
		// In order to correctly sort API transactions, we need to remember their indexes
		const filteredTxnsWithIndexes = filteredTransactions.map((t, index) => [t, index] as [Transaction, number]);
		
		// Sort transactions in reverse chronological order
		// We must sort here because they are returned by reportingWorkflow in order of ReportingStage
		// Use Number.MAX_SAFE_INTEGER as ID for API transactions
		filteredTxnsWithIndexes.sort(([t1, i1], [t2, i2]) => (t2.dt.localeCompare(t1.dt)) || ((t2.id ?? Number.MAX_SAFE_INTEGER) - (t1.id ?? Number.MAX_SAFE_INTEGER) || (i2 - i1)));
		
		transactions.value = filteredTxnsWithIndexes.map(([t, _idx]) => t);
	}
	load();
</script>
