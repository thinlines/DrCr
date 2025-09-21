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
    <DynamicReportComponent :report="report">
        <p v-if="pageSubtitle" class="text-gray-600 text-sm">
            {{ pageSubtitle }}
        </p>
		<div class="my-2 py-2 flex gap-x-2 items-baseline print:hidden">
			<span class="whitespace-nowrap">As at</span>
			<FormattedDateInput v-model.lazy="dt" />
			<DynamicReportMenu :report="report" :subtitle="menuSubtitle" />
		</div>
	</DynamicReportComponent>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { invoke } from '@tauri-apps/api/core';
	import { computed, ref, watch } from 'vue';
	
	import { DynamicReport } from './base.ts';
	import { db } from '../db.ts';
	import DynamicReportComponent from '../components/DynamicReportComponent.vue';
	import DynamicReportMenu from '../components/DynamicReportMenu.vue';
	import { asAtSubtitle } from '../dates.ts';
import FormattedDateInput from '../components/FormattedDateInput.vue';
	
	const report = ref(null as DynamicReport | null);
	
	const dt = ref(null as string | null);

	const pageSubtitle = computed(() => asAtSubtitle(dt.value));
	const menuSubtitle = pageSubtitle;
	
	async function load() {
		await db.load();
		
		const endOfCurrentMonth = dayjs().endOf('month').format('YYYY-MM-DD');
		dt.value = endOfCurrentMonth;
		
		await updateReport();
		
		// Update report when dates etc. changed
		// We initialise the watcher here only after dt is initialised above
		watch([dt], updateReport);
	}
	load();
	
	async function updateReport() {
		const reportDate = dayjs(dt.value!).format('YYYY-MM-DD');
		report.value = DynamicReport.fromJSON(await invoke('get_trial_balance', { date: reportDate }));
	}
</script>
