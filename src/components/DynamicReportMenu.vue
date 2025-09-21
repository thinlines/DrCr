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
	<div class="relative print:hidden">
		<button class="text-gray-400 align-middle hover:text-gray-500" @click="isMenuOpen = !isMenuOpen"><EllipsisHorizontalCircleIcon class="size-6" /></button>
		<ul class="absolute z-20 top-8 right-0 py-1 bg-white w-[11rem] shadow-lg ring-1 ring-black/5 focus:outline-hidden" :class="isMenuOpen ? 'block' : 'hidden'">
			<li class="group cursor-pointer select-none py-1 px-3 text-gray-900 hover:text-white hover:bg-emerald-600" @click="menuPrint">
				<PrinterIcon class="inline size-5 text-gray-500 group-hover:text-white" />
				Print/Save as PDF
			</li>
			<li class="group cursor-pointer select-none py-1 px-3 text-gray-900 hover:text-white hover:bg-emerald-600" @click="menuCsv">
				<DocumentTextIcon class="inline size-5 text-gray-500 group-hover:text-white" />
				Save as CSV
			</li>
		</ul>
	</div>
</template>

<script setup lang="ts">
	import { DocumentTextIcon, EllipsisHorizontalCircleIcon, PrinterIcon } from '@heroicons/vue/24/outline';
	import { save } from '@tauri-apps/plugin-dialog';
	import { writeTextFile } from '@tauri-apps/plugin-fs';
	import { ref } from 'vue';
	
	import { DynamicReport } from '../reports/base.ts';
	
	const { report, columns, subtitle } = defineProps<{ report: DynamicReport | null, columns?: string[], subtitle?: string }>();
	
	const isMenuOpen = ref(false);
	
	async function menuCsv() {
		// Export report to CSV
		const csv = report!.toCSV(columns, subtitle);
		
		// Save to file
		const csvFilename = await save({
			filters: [
				{ name: 'Comma separated values (CSV)', extensions: ['csv'] }
			],
		});
		if (csvFilename !== null) {
			await writeTextFile(csvFilename, csv);
		}
		
		isMenuOpen.value = false;
	}
	
	function menuPrint() {
		window.print();
		isMenuOpen.value = false;
	}
</script>
