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
	<template v-if="report !== null">
		<h1 class="page-heading">
			{{ report.title }}
		</h1>
		
		<slot />
		
		<table class="min-w-full">
			<thead>
				<tr class="border-b border-gray-300">
					<th></th>
					<th v-for="column of report.columns" class="py-0.5 pl-1 text-gray-900 font-semibold text-end">{{ column }}&nbsp;</th>
				</tr>
			</thead>
			<tbody>
				<DynamicReportEntryComponent :entry="entry" v-for="entry of report.entries" />
			</tbody>
		</table>
	</template>
</template>

<script setup lang="ts">
	import { db } from '../db.ts';
	import { DynamicReport } from '../reports/base.ts';
	import DynamicReportEntryComponent from './DynamicReportEntryComponent.vue';
	
	const { report } = defineProps<{ report: DynamicReport | null }>();
</script>
