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
		<div class="my-2 py-2 flex gap-x-2 items-baseline print:hidden">
			<input type="date" class="bordered-field" v-model.lazy="dtStart">
			<span>to</span>
			<input type="date" class="bordered-field" v-model.lazy="dt">
			<div class="print:hidden flex items-center gap-2">
				<span>Compare</span>
				<div class="relative flex flex-grow items-stretch shadow-sm">
					<input type="number" min="1" class="bordered-field w-[9.5em] pr-[6em]"
						v-model.lazy="comparePeriods">
					<div class="absolute inset-y-0 right-0 flex items-center z-10">
						<select
							class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-emerald-600"
							v-model="compareUnit"
							@change="onCompareUnitChange">
							<option value="years">years</option>
							<option value="months">months</option>
						</select>
					</div>
				</div>
			</div>
            <DynamicReportMenu :report="report" :columns="reportColumns" :subtitle="menuSubtitle" />
        </div>
    </DynamicReportComponent>
</template>

<script setup lang="ts">
import dayjs from 'dayjs';
import { invoke } from '@tauri-apps/api/core';
import { ref, watch, computed } from 'vue';

import { DynamicReport } from './base.ts';
import { db } from '../db.ts';
import DynamicReportComponent from '../components/DynamicReportComponent.vue';
import DynamicReportMenu from '../components/DynamicReportMenu.vue';
import { fmtDate, fmtDateRange, labelForReportMonth } from '../dates.ts';

const report = ref(null as DynamicReport | null);
const reportColumns = ref([] as string[]);

const dt = ref(null as string | null);
const dtStart = ref(null as string | null);

const comparePeriods = ref(1);
const compareUnit = ref('years');

// Single source of truth for the page/menu subtitle
const reportSubtitle = computed(() => {
    // Show explicit range when not comparing
    if (comparePeriods.value === 1 && dtStart.value && dt.value) {
        return fmtDateRange(dtStart.value, dt.value);
    }
    // Show monthly periods description when comparing months
    if (compareUnit.value === 'months' && comparePeriods.value > 1 && dt.value) {
        const dayjsDt = dayjs(dt.value!);
        const isEom = dayjsDt.add(1, 'day').isSame(dayjsDt.set('date', 1).add(1, 'month'));
        if (isEom) {
            return 'For calendar months ending on the last day of the month';
        }
        const endDay = parseInt(dayjsDt.format('D'));
        return `For monthly periods ending on the ${ordinalSuffix(endDay)}`;
    }
    return undefined;
});

// Use the same subtitle for both the on-page text and the menu/CSV
const menuSubtitle = reportSubtitle;
const pageSubtitle = reportSubtitle;

function ordinalSuffix(n: number): string {
    const j = n % 10, k = n % 100;
    if (j == 1 && k != 11) return `${n}st`;
    if (j == 2 && k != 12) return `${n}nd`;
    if (j == 3 && k != 13) return `${n}rd`;
    return `${n}th`;
}

async function load() {
	await db.load();

	dt.value = db.metadata.eofy_date;
	dtStart.value = dayjs(db.metadata.eofy_date).subtract(1, 'year').add(1, 'day').format('YYYY-MM-DD');

	await updateReport();

	// Update report when dates etc. changed
	// We initialise the watcher here only after dt and dtStart are initialised above
	watch([dt, dtStart, comparePeriods, compareUnit], updateReport);
}

async function updateReport() {
    const dayjsDt = dayjs(dt.value!);
    let dayjsDtStart = dayjs(dtStart.value!);

    // Auto-correct dtStart when changing dt so each period is well-formed
    // - Ensure start <= end
    // - Snap to expected span for selected compare unit (1 month or 1 year)
    if (compareUnit.value === 'months') {
        const expectedStart = dayjsDt.add(1, 'day').subtract(1, 'month');
        if (!expectedStart.isSame(dayjsDtStart)) {
            // Keep the UX simple: in monthly compare, enforce 1-month window
            dtStart.value = expectedStart.format('YYYY-MM-DD');
            dayjsDtStart = expectedStart;
        }
    } else if (compareUnit.value === 'years') {
        // For yearly compare, only ensure start <= end; do not override custom spans
        if (dayjsDtStart.isAfter(dayjsDt)) {
            const expectedStart = dayjsDt.add(1, 'day').subtract(1, 'year');
            dtStart.value = expectedStart.format('YYYY-MM-DD');
            dayjsDtStart = expectedStart;
        }
    } else {
        throw new Error('Unexpected compareUnit');
    }

    const reportDates = [];
	let newReportColumns = [];

    for (let i = 0; i < comparePeriods.value; i++) {
        let thisReportDt, thisReportDtStart;

		// Get period start and end dates
        if (compareUnit.value === 'years') {
            thisReportDt = dayjsDt.subtract(i, 'year');
            thisReportDtStart = dayjsDtStart.subtract(i, 'year');
            // Keep yearly labels concise
            newReportColumns.push(thisReportDt.format('YYYY'));
        } else if (compareUnit.value === 'months') {
            if (dayjsDt.add(1, 'day').isSame(dayjsDt.set('date', 1).add(1, 'month'))) {
                // If dt is the end of a calendar month, then fix each prior dt to be the end of the calendar month
                thisReportDt = dayjsDt.subtract(i, 'month').set('date', 1).add(1, 'month').subtract(1, 'day');
                thisReportDtStart = dayjsDtStart.subtract(i, 'month');
            } else {
                thisReportDt = dayjsDt.subtract(i, 'month');
                thisReportDtStart = dayjsDtStart.subtract(i, 'month');
            }
            // Simplify labels for calendar-month spans; otherwise show end date in preferred style
            const isCalendarMonth = thisReportDtStart.date() === 1 && thisReportDt.isSame(thisReportDtStart.add(1, 'month').subtract(1, 'day'), 'day');
            newReportColumns.push(labelForReportMonth(thisReportDt, isCalendarMonth));
        } else {
            throw new Error('Unexpected compareUnit');
        }

		reportDates.push([thisReportDtStart.format('YYYY-MM-DD'), thisReportDt.format('YYYY-MM-DD')]);
	}

	if (comparePeriods.value === 1) {
		// Override column headers if only one column
		newReportColumns = ['$'];
	}

	report.value = DynamicReport.fromJSON(await invoke('get_income_statement', { dates: reportDates }));
	reportColumns.value = newReportColumns;  // Wait until report available to update this
}

load();

function onCompareUnitChange() {
	const dayjsDt = dayjs(dt.value!);
	const dayjsDtStart = dayjs(dtStart.value!);

	if (compareUnit.value === 'years') {
		if (dayjsDt.add(1, 'day').subtract(1, 'month').isSame(dayjsDtStart)) {
			// Dates were previously set to one month - now compareUnit changed to years
			// Automatically change dates to one year
			dtStart.value = dayjsDt.add(1, 'day').subtract(1, 'year').format('YYYY-MM-DD');
		}
	} else if (compareUnit.value === 'months') {
		if (dayjsDt.add(1, 'day').subtract(1, 'year').isSame(dayjsDtStart)) {
			// Dates were previously set to one year - now compareUnit changed to months
			// Automatically change dates to one month
			dtStart.value = dayjsDt.add(1, 'day').subtract(1, 'month').format('YYYY-MM-DD');
		}
	} else {
		throw new Error('Unexpected compareUnit');
	}
}
</script>
