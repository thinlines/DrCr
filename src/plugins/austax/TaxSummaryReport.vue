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
		<div class="relative">
			<div class="absolute -top-10 right-0">
				<DynamicReportMenu :report="report" />
			</div>
		</div>
	</DynamicReportComponent>
</template>

<script setup lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { ref } from 'vue';
	
	import DynamicReportComponent from '../../components/DynamicReportComponent.vue';
	import DynamicReportMenu from '../../components/DynamicReportMenu.vue';
	import { DynamicReport } from '../../reports/base.ts';
	
	const report = ref(null as DynamicReport | null);
	
	async function load() {
		report.value = DynamicReport.fromJSON(await invoke('get_tax_summary'));
	}
	load();
</script>
