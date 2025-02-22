<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2025  Lee Yingtong Li (RunasSudo)
	
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
	<ComparativeDynamicReportComponent :reports="reports" :labels="reportLabels">
		<div class="my-2 py-2 flex">
			<div class="grow flex gap-x-2 items-baseline">
				<span class="whitespace-nowrap">As at</span>
				<input type="date" class="bordered-field" v-model.lazy="dt">
				<span>Compare</span>
				<div class="relative flex flex-grow items-stretch shadow-sm">
					<input type="number" min="1" class="bordered-field w-[9.5em] pr-[6em]" v-model.lazy="comparePeriods">
					<div class="absolute inset-y-0 right-0 flex items-center z-10">
						<select class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-emerald-600" v-model="compareUnit">
							<option value="years">years</option>
							<option value="months">months</option>
						</select>
					</div>
				</div>
			</div>
		</div>
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
	</ComparativeDynamicReportComponent>
</template>

<script setup lang="ts">
	import { computed, ref, watch } from 'vue';
	import dayjs from 'dayjs';
	
	import { ExclamationCircleIcon } from '@heroicons/vue/20/solid';
	
	import { Computed, DynamicReport, Entry, Section, Spacer, Subtotal } from './base.ts';
	import { IncomeStatementReport}  from './IncomeStatementReport.vue';
	import { db } from '../db.ts';
	import { ExtendedDatabase } from '../dbutil.ts';
	import ComparativeDynamicReportComponent from '../components/ComparativeDynamicReportComponent.vue';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	
	const reports = ref([] as BalanceSheetReport[]);
	const reportLabels = ref([] as string[]);
	
	const dt = ref(null as string | null);
	const comparePeriods = ref(1);
	const compareUnit = ref('years');
	
	async function load() {
		const session = await db.load();
		
		dt.value = db.metadata.eofy_date;
		
		await updateReport(session);
		
		// Update report when dates etc. changed
		// We initialise the watcher here only after dt and dtStart are initialised above
		watch([dt, comparePeriods, compareUnit], async () => {
			const session = await db.load();
			await updateReport(session);
		});
	}
	load();
	
	async function updateReport(session: ExtendedDatabase) {
		const newReportPromises = [];
		const newReportLabels = [];
		for (let i = 0; i < comparePeriods.value; i++) {
			let thisReportDt, thisReportLabel;
			
			// Get period end date
			if (compareUnit.value === 'years') {
				thisReportDt = dayjs(dt.value!).subtract(i, 'year');
				thisReportLabel = dayjs(dt.value!).subtract(i, 'year').format('YYYY');
			} else if (compareUnit.value === 'months') {
				if (dayjs(dt.value!).add(1, 'day').isSame(dayjs(dt.value!).set('date', 1).add(1, 'month'))) {
					// If dt is the end of a calendar month, then fix each prior dt to be the end of the calendar month
					thisReportDt = dayjs(dt.value!).subtract(i, 'month').set('date', 1).add(1, 'month').subtract(1, 'day');
				} else {
					thisReportDt = dayjs(dt.value!).subtract(i, 'month');
				}
				thisReportLabel = dayjs(dt.value!).subtract(i, 'month').format('YYYY-MM');
			} else {
				throw new Error('Unexpected compareUnit');
			}
			
			// Get start of financial year date
			let sofyDayjs = dayjs(db.metadata.eofy_date).subtract(1, 'year').add(1, 'day');
			let thisReportDtStart = thisReportDt.set('date', sofyDayjs.get('date')).set('month', sofyDayjs.get('month'));
			if (thisReportDtStart.isAfter(thisReportDt)) {
				thisReportDtStart = thisReportDtStart.subtract(1, 'year');
			}
			
			console.log([thisReportDt, thisReportDtStart]);
			
			// Generate reports asynchronously
			newReportPromises.push((async () => {
				const reportingWorkflow = new ReportingWorkflow();
				await reportingWorkflow.generate(session, thisReportDt.format('YYYY-MM-DD'), thisReportDtStart.format('YYYY-MM-DD'));
				return reportingWorkflow.getReportAtStage(ReportingStage.FINAL_STAGE, BalanceSheetReport) as BalanceSheetReport;
			})());
			
			if (comparePeriods.value === 1) {
				// If only 1 report, the heading is simply "$"
				newReportLabels.push(db.metadata.reporting_commodity);
			} else {
				newReportLabels.push(thisReportLabel);
			}
		}
		
		reports.value = await Promise.all(newReportPromises);
		reportLabels.value = newReportLabels;
	}
	
	const doesBalance = computed(function() {
		let doesBalance = true;
		for (const report of reports.value) {
			const totalAssets = (report.byId('total_assets') as Computed).quantity;
			const totalLiabilities = (report.byId('total_liabilities') as Computed).quantity;
			const totalEquity = (report.byId('total_equity') as Computed).quantity;
			if (totalAssets !== totalLiabilities + totalEquity) {
				doesBalance = false;
			}
		}
		return doesBalance;
	});
</script>
