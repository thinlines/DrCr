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
	<h1 class="page-heading mb-4">
		Trial balance
	</h1>
	
	<table class="min-w-full" v-if="report">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
				<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Cr</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="[account, quantity] in report.balances.entries()">
				<td class="py-0.5 pr-1 text-gray-900"><RouterLink :to="{ name: 'transactions', params: { account: account } }" class="hover:text-blue-700 hover:underline">{{ account }}</RouterLink></td>
				<td class="py-0.5 px-1 text-gray-900 text-end">
					<template v-if="quantity >= 0">{{ pp(quantity) }}</template>
				</td>
				<td class="py-0.5 pl-1 text-gray-900 text-end">
					<template v-if="quantity < 0">{{ pp(-quantity) }}</template>
				</td>
			</tr>
			<tr>
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Total</th>
				<th class="py-0.5 px-1 text-gray-900 text-end">{{ pp(total_dr!) }}</th>
				<th class="py-0.5 pl-1 text-gray-900 text-end">{{ pp(-total_cr!) }}</th>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	
	import { db } from '../db.ts';
	import { pp } from '../display.ts';
	import { ReportingStage, ReportingWorkflow, TrialBalanceReport } from '../reporting.ts';
	
	const report = ref(null as TrialBalanceReport | null);
	
	// WebKit: Iterator.reduce not supported - https://bugs.webkit.org/show_bug.cgi?id=248650
	const total_dr = computed(() => report.value ?
		[...report.value.balances.values()].reduce((acc, x) => x > 0 ? acc + x : acc, 0)
		: 0
	);
	const total_cr = computed(() => report.value ?
		[...report.value.balances.values()].reduce((acc, x) => x < 0 ? acc + x : acc, 0)
		: 0
	);
	
	async function load() {
		const session = await db.load();
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session);
		
		report.value = reportingWorkflow.getReportAtStage(ReportingStage.OrdinaryAPITransactions, TrialBalanceReport) as TrialBalanceReport;
	}
	load();
</script>
