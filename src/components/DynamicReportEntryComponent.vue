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
	<template v-if="entry.LiteralRow">
		<tr :class="entry.LiteralRow.bordered ? 'border-y border-gray-300' : null">
			<component :is="entry.LiteralRow.heading ? 'th' : 'td'" class="py-0.5 pr-1 text-gray-900 text-start" :class="{ 'font-semibold': entry.LiteralRow.heading }">
				<a :href="entry.LiteralRow.link" class="hover:text-blue-700 hover:underline" v-if="entry.LiteralRow.link !== null">{{ entry.LiteralRow.text }}</a>
				<template v-if="entry.LiteralRow.link === null">{{ entry.LiteralRow.text }}</template>
			</component>
			<component :is="entry.LiteralRow.heading ? 'th' : 'td'" class="py-0.5 pl-1 text-gray-900 text-end" :class="{ 'font-semibold': entry.LiteralRow.heading }" v-html="(cell !== 0 || entry.LiteralRow.heading) ? ppBracketed(cell, entry.LiteralRow.link ?? undefined) : ''" v-for="cell of entry.LiteralRow.quantity">
			</component>
		</tr>
	</template>
	<template v-if="entry.Section">
		<tr v-if="entry.Section.text !== null">
			<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">{{ entry.Section.text }}</th>
			<th></th><!-- FIXME: Have correct colspan -->
		</tr>
		<DynamicReportEntryComponent :entry="child" v-for="child of entry.Section.entries" />
	</template>
	<template v-if="entry == 'Spacer'">
		<tr><td colspan="2" class="py-0.5">&nbsp;</td></tr><!-- FIXME: Have correct colspan -->
	</template>
</template>

<script setup lang="ts">
	import { ppBracketed } from '../display.ts';
	import { DynamicReportEntry } from '../reports/base.ts';
	
	const { entry } = defineProps<{ entry: DynamicReportEntry }>();
</script>
