<!--
	DrCr: Double-entry bookkeeping framework
	Copyright (C) 2022-2025  Lee Yingtong Li (RunasSudo)
	
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
    <DynamicReportComponent :report="report" :columns="reportColumns">
        <p v-if="pageSubtitle" class="text-gray-600 text-sm mt-1">
            {{ pageSubtitle }}
        </p>
		<div class="my-2 py-2 flex print:hidden">
			<div class="grow flex gap-x-2 items-baseline">
				<span class="whitespace-nowrap">As at</span>
				<FormattedDateInput v-model.lazy="dt" />
				<div class="print:hidden flex items-center gap-2">
					<span>Compare</span>
					<div class="relative flex flex-grow items-stretch shadow-sm">
						<input type="number" min="1" class="bordered-field w-[9.5em] pr-[6em]"
							v-model.lazy="comparePeriods">
						<div class="absolute inset-y-0 right-0 flex items-center z-10">
							<select
								class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-emerald-600"
								v-model="compareUnit">
								<option value="years">years</option>
								<option value="months">months</option>
							</select>
						</div>
					</div>
				</div>

    				<DynamicReportMenu :report="report" :columns="reportColumns" :subtitle="menuSubtitle" />
			</div>
		</div>
		<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2" v-if="!doesBalance">
			<div class="flex">
				<div class="flex-shrink-0">
					<ExclamationCircleIcon class="h-5 w-5 text-red-400" />
				</div>
				<div class="ml-3 flex-1">
					<p class="text-sm text-red-700">Total assets do not equal total liabilities and equity. This may
						occur if not all accounts have been classified in the chart of accounts.</p>
				</div>
			</div>
		</div>
	</DynamicReportComponent>
</template>

<script setup lang="ts">
import dayjs from 'dayjs';
import { invoke } from '@tauri-apps/api/core';
import { computed, ref, watch } from 'vue';

import { ExclamationCircleIcon } from '@heroicons/vue/20/solid';

import { DynamicReport, Row } from './base.ts';
import { db } from '../db.ts';
import DynamicReportComponent from '../components/DynamicReportComponent.vue';
import DynamicReportMenu from '../components/DynamicReportMenu.vue';
import { asAtSubtitle, labelForReportMonth } from '../dates.ts';
import FormattedDateInput from '../components/FormattedDateInput.vue';

const report = ref(null as DynamicReport | null);
const reportColumns = ref([] as string[]);

const dt = ref(null as string | null);
const comparePeriods = ref(1);
const compareUnit = ref('years');

const pageSubtitle = computed(() => asAtSubtitle(dt.value, compareUnit.value, comparePeriods.value));
const menuSubtitle = pageSubtitle;

async function load() {
	await db.load();

	dt.value = db.metadata.eofy_date;

	await updateReport();

	// Update report when dates etc. changed
	// We initialise the watcher here only after dt is initialised above
	watch([dt, comparePeriods, compareUnit], updateReport);
}
load();

async function updateReport() {
	const reportDates = [];
	let newReportColumns = [];

	for (let i = 0; i < comparePeriods.value; i++) {
		let thisReportDt;

		// Get period end date
		if (compareUnit.value === 'years') {
			thisReportDt = dayjs(dt.value!).subtract(i, 'year');
			newReportColumns.push(thisReportDt.format('YYYY'));
    } else if (compareUnit.value === 'months') {
        if (dayjs(dt.value!).add(1, 'day').isSame(dayjs(dt.value!).set('date', 1).add(1, 'month'))) {
            // If dt is the end of a calendar month, then fix each prior dt to be the end of the calendar month
            thisReportDt = dayjs(dt.value!).subtract(i, 'month').set('date', 1).add(1, 'month').subtract(1, 'day');
        } else {
            thisReportDt = dayjs(dt.value!).subtract(i, 'month');
        }
        const isCalendarMonth = thisReportDt.add(1, 'day').isSame(thisReportDt.set('date', 1).add(1, 'month'));
        newReportColumns.push(labelForReportMonth(thisReportDt, isCalendarMonth));
    } else {
        throw new Error('Unexpected compareUnit');
    }

		reportDates.push(thisReportDt.format('YYYY-MM-DD'));
	}

	if (comparePeriods.value === 1) {
		// Override column headers if only one column
		newReportColumns = ['$'];
	}

	report.value = DynamicReport.fromJSON(await invoke('get_balance_sheet', { dates: reportDates }));
	reportColumns.value = newReportColumns;  // Wait until report available to update this
}

const doesBalance = computed(function () {
	if (report.value === null) {
		return true;
	}

	const totalAssets = (report.value.byId('total_assets') as { Row: Row }).Row.quantity;
	const totalLiabilities = (report.value.byId('total_liabilities') as { Row: Row }).Row.quantity;
	const totalEquity = (report.value.byId('total_equity') as { Row: Row }).Row.quantity;

	let doesBalance = true;
	for (let column = 0; column < report.value.columns.length; column++) {
		if (totalAssets[column] !== totalLiabilities[column] + totalEquity[column]) {
			doesBalance = false;
			break;
		}
	}
	return doesBalance;
});
</script>
