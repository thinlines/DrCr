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

<script lang="ts">
	export class BalanceSheetReport extends DynamicReport {
		constructor() {
			super('Balance sheet');
		}
		
		async generate(balances: Map<string, number>, incomeStatementReport: IncomeStatementReport) {
			this.entries = [
				new Section(
					'Assets',
					[
						...await DynamicReport.entriesForKind(balances, 'drcr.asset'),
						new Subtotal('Total assets', 'total_assets', true /* visible */, true /* bordered */)
					]
				),
				new Spacer(),
				new Section(
					'Liabilities',
					[
						...await DynamicReport.entriesForKind(balances, 'drcr.liability', true),
						new Subtotal('Total liabilities', 'total_liabilities', true /* visible */, true /* bordered */)
					]
				),
				new Spacer(),
				new Section(
					'Equity',
					[
						...await DynamicReport.entriesForKind(balances, 'drcr.equity', true),
						new Entry('Current year surplus (deficit)', (incomeStatementReport.byId('net_surplus') as Computed).quantity, null /* id */, true /* visible */, false /* autoHide */, '/income-statement'),
						new Entry('Accumulated surplus (deficit)', -(balances.get('Accumulated surplus (deficit)') ?? 0), null /* id */, true /* visible */, false /* autoHide */, '/transactions/Accumulated surplus (deficit)'),
						new Subtotal('Total equity', 'total_equity', true /* visible */, true /* bordered */)
					]
				)
			];
			
			this.calculate();
		}
	}
	
	
</script>

<!-- Report display -->

<template>
	<DynamicReportComponent :report="report">
		<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2" v-if="!doesBalance">
			<div class="flex">
				<div class="flex-shrink-0">
					<ExclamationCircleIcon class="h-5 w-5 text-red-400" />
				</div>
				<div class="ml-3 flex-1">
					<p class="text-sm text-red-700">Total assets do not equal total liabilities and equity. This may occur if not all accounts have been classified in the chart of accounts.</p>
				</div>
			</div>
		</div>
	</DynamicReportComponent>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	
	import { ExclamationCircleIcon } from '@heroicons/vue/20/solid';
	
	import { Computed, DynamicReport, Entry, Section, Spacer, Subtotal } from './base.ts';
	import { IncomeStatementReport}  from './IncomeStatementReport.vue';
	import { db } from '../db.ts';
	import DynamicReportComponent from '../components/DynamicReportComponent.vue';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	
	const report = ref(null as BalanceSheetReport | null);
	
	async function load() {
		const session = await db.load();
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session);
		
		report.value = reportingWorkflow.getReportAtStage(ReportingStage.FINAL_STAGE, BalanceSheetReport) as BalanceSheetReport;
	}
	load();
	
	const doesBalance = computed(function() {
		if (report.value === null) {
			return true;
		}
		const totalAssets = (report.value.byId('total_assets') as Computed).quantity;
		const totalLiabilities = (report.value.byId('total_liabilities') as Computed).quantity;
		const totalEquity = (report.value.byId('total_equity') as Computed).quantity;
		return totalAssets === totalLiabilities + totalEquity;
	});
</script>
