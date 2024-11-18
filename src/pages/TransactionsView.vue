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
		<!-- Use a rather than RouterLink because RouterLink adds its own event handler -->
		<a href="/journal/new-transaction" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New transaction
		</a>
		<button v-if="commodityDetail" class="btn-secondary" @click="commodityDetail = false">Hide commodity detail</button>
		<button v-if="!commodityDetail" class="btn-secondary" @click="commodityDetail = true">Show commodity detail</button>
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
		
		const transactionsRaw = reportingWorkflow.getTransactionsAtStage(ReportingStage.OrdinaryAPITransactions);
		
		// Filter only transactions affecting this account
		transactions.value = transactionsRaw.filter((t) => t.postings.some((p) => p.account === route.params.account));
		
		// Display transactions in reverse chronological order
		// We must sort here because they are returned by reportingWorkflow in order of ReportingStage
		transactions.value.sort((a, b) => (b.dt.localeCompare(a.dt)) || ((b.id ?? 0) - (a.id ?? 0)));
	}
	load();
</script>
