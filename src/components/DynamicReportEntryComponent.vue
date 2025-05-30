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
		import { db } from '../db.ts';

	You should have received a copy of the GNU Affero General Public License
	along with this program.  If not, see <https://www.gnu.org/licenses/>.
-->

<template>
	<template v-if="literalRow">
		<template v-if="literalRow.visible">
			<tr :class="literalRow.bordered ? 'border-y border-gray-300' : null">
				<component :is="literalRow.heading ? 'th' : 'td'" class="py-0.5 pr-1 text-gray-900 text-start" :class="{ 'font-semibold': literalRow.heading }">
					<a :href="literalRow.link as string" class="hover:text-blue-700 hover:underline" v-if="literalRow.link !== null">{{ literalRow.text }}</a>
					<template v-if="literalRow.link === null">{{ literalRow.text }}</template>
				</component>
				<component :is="literalRow.heading ? 'th' : 'td'" class="py-0.5 pl-1 text-gray-900 text-end" :class="{ 'font-semibold': literalRow.heading }" v-html="(cell !== 0 || literalRow.heading) ? ppBracketed(cell, literalRow.link ?? undefined) : ''" v-for="cell of literalRow.quantity">
				</component>
			</tr>
		</template>
	</template>
	<template v-if="section">
		<template v-if="section.visible">
			<tr v-if="section.text !== null">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">{{ section.text }}</th>
				<th></th><!-- FIXME: Have correct colspan -->
			</tr>
			<DynamicReportEntryComponent :entry="child" v-for="child of section.entries" />
		</template>
	</template>
	<template v-if="entry == 'Spacer'">
		<tr><td colspan="2" class="py-0.5">&nbsp;</td></tr><!-- FIXME: Have correct colspan -->
	</template>
</template>

<script setup lang="ts">
	import { computed } from 'vue';

	import { ppBracketed } from '../display.ts';
	import { DynamicReportEntry, LiteralRow, Section } from '../reports/base.ts';
	
	const { entry } = defineProps<{ entry: DynamicReportEntry }>();
	
	const literalRow = computed(function() {
		return (entry as { LiteralRow: LiteralRow }).LiteralRow;
	});
	const section = computed(function() {
		return (entry as { Section: Section }).Section;
	});
</script>
