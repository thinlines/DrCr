<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2024  Lee Yingtong Li (RunasSudo)
	
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
	<template v-if="entry instanceof Entry">
		<!-- NB: Subtotal and Calculated are subclasses of Entry -->
		<tr :class="entry.bordered ? 'border-y border-gray-300' : null">
			<component :is="entry.heading ? 'th' : 'td'" class="py-0.5 pr-1 text-gray-900 text-start" :class="{ 'font-semibold': entry.heading }">
				<a :href="entry.link" class="hover:text-blue-700 hover:underline" v-if="entry.link !== null">{{ entry.text }}</a>
				<template v-if="entry.link === null">{{ entry.text }}</template>
			</component>
			<component :is="entry.heading ? 'th' : 'td'" class="py-0.5 pl-1 text-gray-900 text-end" :class="{ 'font-semibold': entry.heading }" v-html="ppBracketed(entry.quantity, entry.link ?? undefined)" />
		</tr>
	</template>
	<template v-if="entry instanceof Section">
		<tr v-if="entry.title !== null">
			<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">{{ entry.title }}</th>
			<th></th>
		</tr>
		<DynamicReportEntry :entry="child" v-for="child of entry.entries" />
	</template>
	<template v-if="entry instanceof Spacer">
		<tr><td colspan="2" class="py-0.5">&nbsp;</td></tr>
	</template>
</template>

<script setup lang="ts">
	import { defineProps } from 'vue';
	
	import { ppBracketed } from '../display.ts';
	import { DynamicReportNode, Entry, Section, Spacer } from '../reports/base.ts';
	
	const { entry } = defineProps<{ entry: DynamicReportNode }>();
</script>
