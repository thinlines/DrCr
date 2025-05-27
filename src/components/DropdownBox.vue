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
		<button type="button" class="relative w-full cursor-default bg-white bordered-field pl-3 pr-10 text-left" @click="isOpen = !isOpen">
			<span class="block truncate">{{ selectedValue[1] }}</span>
			<span class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2">
				<ChevronUpDownIcon class="h-5 w-5 text-gray-400" />
			</span>
		</button>
		<ul class="absolute z-20 mt-1 max-h-60 w-full overflow-auto bg-white py-1 text-sm shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none" :class="isOpen ? 'block' : 'hidden'">
			<template v-for="([categoryName, categoryItems], index) in values">
				<li class="relative cursor-default select-none py-1 pl-3 pr-9 text-gray-500 border-b border-gray-300" :class="{ 'pt-4': index > 0 }" v-if="categoryName">
					<span class="block truncate text-xs font-bold uppercase">{{ categoryName }}</span>
				</li>
				<li v-for="item in categoryItems" class="group relative cursor-default select-none py-1 pl-3 pr-9 text-gray-900 hover:text-white hover:bg-emerald-600" :data-selected="item[0] === selectedValue[0] ? 'selected' : null" @click="selectedValue = item; isOpen = false">
					<span class="block truncate group-data-[selected=selected]:font-semibold">{{ item[1] }}</span>
					<span class="hidden group-data-[selected=selected]:flex absolute inset-y-0 right-0 items-center pr-4 text-emerald-600 group-hover:text-white">
						<CheckIcon class="h-5 w-5" />
					</span>
				</li>
			</template>
		</ul>
	</div>
</template>

<script setup lang="ts">
	import { CheckIcon, ChevronUpDownIcon } from '@heroicons/vue/24/outline';
	
	import { ref } from 'vue';
	
	const { values } = defineProps<{ values: [string | null, [string, string][]][] }>();  // Array of [category name, [internal identifier, pretty name]]
	
	const selectedValue = defineModel({ default: null! as [string, string] });  // Vue bug: Compiler produces broken code if setting default directly here
	if (selectedValue.value === null) {
		selectedValue.value = values[0][1][0];
	}
	
	const isOpen = ref(false);
</script>
