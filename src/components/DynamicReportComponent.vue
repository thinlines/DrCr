<!--
	DrCr: Web-based double-entry bookkeeping framework
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
	<div v-if="report !== null" class="flex flex-col h-full min-h-0">
		<h1 class="page-heading">
			{{ report.title }}
		</h1>
		
		<slot />
		
		<div class="flex-1 min-h-0 overflow-y-auto wk-aa print:h-auto print:overflow-visible">
			<table class="min-w-full sticky-table">
				<thead class="sticky-header">
					<tr class="border-b border-gray-300">
						<th></th>
						<th v-for="column of (columns ?? report.columns)" class="py-0.5 pl-1 text-gray-900 font-semibold text-end">{{ column }}&nbsp;</th>
					</tr>
				</thead>
				<tbody>
					<DynamicReportEntryComponent :entry="entry" v-for="entry of report.entries" />
				</tbody>
			</table>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { DynamicReport } from '../reports/base.ts';
	import DynamicReportEntryComponent from './DynamicReportEntryComponent.vue';
	
	const { report, columns } = defineProps<{ report: DynamicReport | null, columns?: string[] }>();
</script>
