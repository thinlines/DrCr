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
		Balance assertions
	</h1>
	
	<div class="my-4 flex gap-x-2">
		<a :href="$router.resolve({name: 'balance-assertions-new'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New assertion
		</a>
	</div>
	
	<table class="min-w-full">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Balance</th>
				<th></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Status</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="assertion of balanceAssertions">
				<td class="py-0.5 pr-1 text-gray-900">{{ dayjs(assertion.dt).format('YYYY-MM-DD') }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ assertion.description }}</td>
				<td class="py-0.5 px-1 text-gray-900"><RouterLink :to="{ name: 'transactions', params: { account: assertion.account } }" class="text-gray-900 hover:text-blue-700 hover:underline">{{ assertion.account }}</RouterLink></td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(Math.abs(assertion.quantity)) }}</td>
				<td class="py-0.5 pr-1 text-gray-900">{{ assertion.quantity >= 0 ? 'Dr' : 'Cr' }}</td>
				<td class="py-0.5 px-1 text-gray-900">
					<CheckIcon class="w-4 h-4" v-if="assertion.isValid" />
					<XMarkIcon class="w-4 h-4 text-red-500" v-if="!assertion.isValid" />
				</td>
				<td class="py-0.5 pl-1 text-gray-900 text-end">
					<a :href="'/balance-assertions/edit/' + assertion.id" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">
						<PencilIcon class="w-4 h-4" />
					</a>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	
	import { ref } from 'vue';
	
	import { db, totalBalancesAtDate } from '../db.ts';
	import { pp } from '../display.ts';
	import { CheckIcon, PencilIcon, XMarkIcon } from '@heroicons/vue/24/outline';
	import { PlusIcon } from '@heroicons/vue/16/solid';
	
	const balanceAssertions = ref([] as ValidatedBalanceAssertion[]);
	
	interface ValidatedBalanceAssertion {
		id: number,
		dt: string,
		description: string,
		account: string,
		quantity: number,
		commodity: string,
		isValid: boolean,
	}
	
	async function load() {
		const session = await db.load();
		
		const rawBalanceAssertions: any[] = await session.select(
			`SELECT *
			FROM balance_assertions
			ORDER BY dt DESC, id DESC`
		);
		
		/*
		// Cache trial balances in case there are multiple assertions per date
		const trialBalanceForDate = new Map<string, TrialBalanceReport>();
		
		for (const balanceAssertion of rawBalanceAssertions) {
			// Check assertion status
			// TODO: This is very inefficient because API transactions are generated multiple times
			if (!trialBalanceForDate.has(balanceAssertion.dt)) {
				const reportingWorkflow = new ReportingWorkflow();
				await reportingWorkflow.generate(session, balanceAssertion.dt);
				const trialBalance = reportingWorkflow.getReportAtStage(ReportingStage.OrdinaryAPITransactions, TrialBalanceReport) as TrialBalanceReport;
				trialBalanceForDate.set(balanceAssertion.dt, trialBalance);
			}
			
			const trialBalance = trialBalanceForDate.get(balanceAssertion.dt)!;
			balanceAssertion.isValid = balanceAssertion.quantity === trialBalance.balances.get(balanceAssertion.account) && balanceAssertion.commodity === db.metadata.reporting_commodity;
		}
		*/
		
		// Check assertion status
		const balancesForDate = new Map<string, Map<string, number>>();
		
		for (const balanceAssertion of rawBalanceAssertions) {
			if (!balancesForDate.has(balanceAssertion.dt)) {
				// FIXME: This is quite slow
				balancesForDate.set(balanceAssertion.dt, await totalBalancesAtDate(session, balanceAssertion.dt));
			}
			balanceAssertion.isValid = balanceAssertion.quantity === balancesForDate.get(balanceAssertion.dt)!.get(balanceAssertion.account) && balanceAssertion.commodity === db.metadata.reporting_commodity;
		}
		
		balanceAssertions.value = rawBalanceAssertions as ValidatedBalanceAssertion[];
	}
	
	load();
</script>
