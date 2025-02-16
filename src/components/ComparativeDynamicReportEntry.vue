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

<template>
	<template v-if="row[0] instanceof Entry">
		<!-- NB: Subtotal and Calculated are subclasses of Entry -->
		<tr :class="row[0].bordered ? 'border-y border-gray-300' : null">
			<component :is="row[0].heading ? 'th' : 'td'" class="py-0.5 pr-1 text-gray-900 text-start" :class="{ 'font-semibold': row[0].heading }">
				<a :href="row[0].link" class="hover:text-blue-700 hover:underline" v-if="row[0].link !== null">{{ row[0].text }}</a>
				<template v-if="row[0].link === null">{{ row[0].text }}</template>
			</component>
			<template v-for="entry of row[1]">
				<component :is="row[0].heading ? 'th' : 'td'" class="py-0.5 pl-1 text-gray-900 text-end" :class="{ 'font-semibold': row[0].heading }" v-html="entry ? ppBracketed((entry as Entry).quantity, (entry as Entry).link ?? undefined) : ''" />
			</template>
		</tr>
	</template>
	<template v-if="row[0] instanceof Section">
		<tr v-if="row[0].title !== null">
			<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">{{ row[0].title }}</th>
			<th></th>
		</tr>
		<ComparativeDynamicReportEntry :row="childRow" v-for="childRow of joinedChildren" />
	</template>
	<template v-if="row[0] instanceof Spacer">
		<tr><td :colspan="row[1].length + 1" class="py-0.5">&nbsp;</td></tr>
	</template>
</template>

<script setup lang="ts">
	import { computed, defineProps } from 'vue';
	
	import { ppBracketed } from '../display.ts';
	import { DynamicReportNode, Entry, Section, Spacer, Subtotal } from '../reports/base.ts';
	
	const { row } = defineProps<{ row: [DynamicReportNode, (DynamicReportNode | null)[]] }>();
	
	const joinedChildren = computed(() => {
		// First get all children's names
		const joinedNames: string[] = [];
		for (let cell of row[1]) {
			for (let entry of (cell as any).entries) {
				if (entry instanceof Subtotal) {  // Handle Subtotal separately
					continue;
				}
				if (!joinedNames.includes((entry as any).text)) {
					joinedNames.push((entry as any).text);
				}
			}
		}
		joinedNames.sort();
		
		// Then return joined children in order of sorted names
		const result: [DynamicReportNode, (DynamicReportNode | null)[]][] = [];
		for (let name of joinedNames) {
			const thisRow: DynamicReportNode[] = [];
			let thisRowExample = null;
			for (let cell of row[1]) {
				let thisCell = null;
				for (let entry of (cell as any).entries) {
					if ((entry as any).text === name) {
						thisCell = entry;
						thisRowExample = entry;
						break;
					}
				}
				thisRow.push(thisCell);
			}
			result.push([thisRowExample, thisRow]);
		}
		
		// Add Subtotal
		const subtotalRow = [];
		let subtotalExample = null;
		for (let cell of row[1]) {
			let thisCell = null;
			for (let entry of (cell as any).entries) {
				if (entry instanceof Subtotal) {
					thisCell = entry;
					subtotalExample = entry;
					break;
				}
			}
			subtotalRow.push(thisCell);
		}
		if (subtotalExample) {
			result.push([subtotalExample, subtotalRow]);
		}
		
		return result;
	});
</script>
