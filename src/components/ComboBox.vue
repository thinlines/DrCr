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
	<div class="relative">
		<!-- WebKit bug https://bugs.webkit.org/show_bug.cgi?id=142968: Does not align baseline correctly unless some text or placeholder is present -->
		<input type="text" class="bordered-field peer pr-7" :class="inputClass" id="account" v-model="selectedValue" placeholder=" " autocomplete="off" ref="inputField">
		<button type="button" class="absolute inset-y-0 right-0 flex items-center px-2 focus:outline-none" @click="inputField!.focus()">
			<ChevronUpDownIcon class="h-5 w-5 text-gray-400" />
		</button>
		<ul class="hidden peer-focus:block absolute z-20 mt-1 max-h-60 w-full overflow-auto bg-white py-1 text-base shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm" v-if="values.length > 0">
			<li
				v-for="value in values"
				v-show="value.toLowerCase().startsWith(selectedValue.toLowerCase())"
				class="group relative cursor-default select-none py-1 pl-3 pr-9 text-gray-900 hover:text-white hover:bg-emerald-600 wk-aa"
				:data-selected="value === selectedValue ? 'selected': ''"
				@mousedown="selectedValue = value"
			>
				<span class="block truncate group-data-[selected=selected]:font-semibold">{{ value }}</span>
				<span class="hidden group-data-[selected=selected]:flex absolute inset-y-0 right-0 items-center pr-4 text-emerald-600 group-hover:text-white">
					<CheckIcon class="w-5 h-5" />
				</span>
			</li>
		</ul>
	</div>
</template>

<script setup lang="ts">
	import { CheckIcon } from '@heroicons/vue/20/solid';
	import { ChevronUpDownIcon } from '@heroicons/vue/24/outline';
	import { useTemplateRef } from 'vue';
	
	const { values, inputClass } = defineProps<{ values: string[], inputClass?: string }>();
	const inputField = useTemplateRef('inputField');
	
	const selectedValue = defineModel({ default: '' });
</script>
