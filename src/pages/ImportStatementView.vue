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
	<h1 class="page-heading mb-4">
		Import statement
	</h1>
	
	<div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
		<label for="format" class="block text-gray-900 pr-4">File type</label>
		<div>
			<select class="bordered-field" id="format" v-model="format">
				<option value="ofx">OFX (1.x/2.x)</option>
				<option value="csv">CSV</option>
			</select>
		</div>
		<label for="account" class="block text-gray-900 pr-4">Source account</label>
		<ComboBoxAccounts v-model="sourceAccount" />
		<label for="file" class="block text-gray-900 pr-4">File</label>
		<div class="flex grow">
			<!-- WebKit: file:hidden hides the filename as well so we have a dummy text input -->
			<input type="text" class="bordered-field" :value="selectedFilename" @click="openFileDialog" placeholder=" " readonly>
			<input type="file" class="hidden" id="file" accept=".ofx" ref="file" @change="fileInputChanged">
			<label for="file" class="btn-primary bg-gray-600 hover:bg-gray-700">Browse</label>
		</div>
	</div>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-secondary" @click="previewImport">Preview</button>
	</div>
	
	<div v-if="statementLines.length > 0">
		<h2 class="page-heading my-4">
			Import statement preview
		</h2>
		
		<table class="min-w-full">
			<thead>
				<tr class="border-b border-gray-300">
					<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Cr</th>
					<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Balance</th>
				</tr>
			</thead>
			<tbody>
				<tr v-for="line in statementLines">
					<td class="py-0.5 pr-1 text-gray-900">{{ dayjs(line.dt).format('YYYY-MM-DD') }}</td>
					<td class="py-0.5 px-1 text-gray-900">{{ line.description }}</td>
					<td class="py-0.5 px-1 text-gray-900 text-end">{{ line.quantity >= 0 ? ppWithCommodity(line.quantity, line.commodity) : '' }}</td>
					<td class="py-0.5 px-1 text-gray-900 text-end">{{ line.quantity < 0 ? ppWithCommodity(-line.quantity, line.commodity) : '' }}</td>
					<td class="py-0.5 pl-1 text-gray-900 text-end">{{ line.balance ?? '' }}</td>
				</tr>
			</tbody>
		</table>
		
		<div class="flex justify-end mt-4 space-x-2">
			<button class="btn-primary" @click="doImport">Import</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	
	import { ref, useTemplateRef } from 'vue';
	import { useRouter } from 'vue-router';
	
	import { StatementLine, db } from '../db.ts';
	import ComboBoxAccounts from '../components/ComboBoxAccounts.vue';
	import { ppWithCommodity } from '../display.ts';
	
	import importCsv from '../importers/csv.ts';
	import importOfxAutodetectVersion from '../importers/ofx.ts';
	
	const fileInput = useTemplateRef('file');
	
	const format = ref('ofx');
	const selectedFilename = ref('');
	const sourceAccount = ref('');
	
	const router = useRouter();
	
	const statementLines = ref([] as StatementLine[]);
	
	function openFileDialog() {
		fileInput.value?.click();
	}
	
	function fileInputChanged() {
		selectedFilename.value = fileInput.value!.files?.item(0)?.name ?? '';
	}
	
	async function previewImport() {
		const file = fileInput.value!.files?.item(0);
		if (!file) {
			return;
		}
		
		const content = await file.text();
		
		if (format.value === 'csv') {
			statementLines.value = importCsv(sourceAccount.value, content);
		} else if (format.value === 'ofx') {
			statementLines.value = importOfxAutodetectVersion(sourceAccount.value, content);
		} else {
			throw new Error('Unexpected import format');
		}
	}
	
	async function doImport() {
		// Import statement lines to database atomically
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		for (const line of statementLines.value) {
			await dbTransaction.execute(
				`INSERT INTO statement_lines (source_account, dt, description, quantity, balance, commodity)
				VALUES (?, ?, ?, ?, ?, ?)`,
				[line.source_account, line.dt, line.description, line.quantity, line.balance, line.commodity]
			);
		}
		
		dbTransaction.commit();
		
		router.push({ name: 'statement-lines' });
	}
</script>
