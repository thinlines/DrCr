<!--
	DrCr: Double-entry bookkeeping framework
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
		
		<div class="flex flex-wrap items-center justify-between gap-y-2 text-sm text-gray-700 mb-2">
			<div>
				{{ importableCount }} new · {{ duplicateCount }} duplicates
			</div>
			<div class="flex flex-wrap items-center gap-3">
				<div v-for="total in formattedImportTotals" :key="total.commodity" class="whitespace-nowrap">
					Dr {{ total.debitText }} · Cr {{ total.creditText }}
				</div>
				<label class="inline-flex items-center gap-2">
					<input type="checkbox" class="checkbox-primary" v-model="hideDuplicates">
					<span>Hide duplicates</span>
				</label>
			</div>
		</div>
		
		<div class="max-h-[60vh] overflow-y-auto wk-aa">
			<table class="min-w-full table-auto sticky-table">
				<thead class="sticky-header">
				<tr class="border-b border-gray-300">
					<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Status</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Cr</th>
					<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Balance</th>
				</tr>
				</thead>
				<tbody>
					<tr v-for="line in filteredStatementLines" :class="line.duplicate ? 'bg-amber-50' : ''">
						<td :class="['py-0.5 pr-1', tableTextClass(line)]">{{ dayjs(line.dt).format('YYYY-MM-DD') }}</td>
						<td :class="['py-0.5 px-1', tableTextClass(line)]">{{ line.description }}</td>
						<td :class="['py-0.5 px-1', tableTextClass(line)]" :title="duplicateMatchTooltip(line)">{{ formatDuplicateStatus(line) }}</td>
						<td :class="['py-0.5 px-1 text-end', tableTextClass(line)]">{{ line.quantity >= 0 ? ppWithCommodity(line.quantity, line.commodity) : '' }}</td>
						<td :class="['py-0.5 px-1 text-end', tableTextClass(line)]">{{ line.quantity < 0 ? ppWithCommodity(-line.quantity, line.commodity) : '' }}</td>
						<td :class="['py-0.5 pl-1 text-end', tableTextClass(line)]">{{ line.balance ?? '' }}</td>
					</tr>
				</tbody>
			</table>
		</div>
		
		<div class="rounded-md bg-amber-50 mt-4 p-4 col-span-2" v-if="duplicateCount > 0">
			<div class="flex">
				<div class="flex-shrink-0">
					<ExclamationCircleIcon class="h-5 w-5 text-amber-400" />
				</div>
				<div class="ml-3 flex-1">
					<p class="text-sm text-amber-900">{{ duplicateCount }} duplicate {{ duplicateCount === 1 ? 'line has' : 'lines have' }} been detected. They will be skipped when importing.</p>
					<p v-if="importableCount === 0" class="text-sm text-amber-900 mt-2">Nothing new will be imported.</p>
				</div>
			</div>
		</div>
		
		<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2" v-if="hasZeroAmounts">
			<div class="flex">
				<div class="flex-shrink-0">
					<ExclamationCircleIcon class="h-5 w-5 text-red-400" />
				</div>
				<div class="ml-3 flex-1">
					<p class="text-sm text-red-700">The imported statement will contain lines with zero amounts.</p>
				</div>
			</div>
		</div>
		
		<div class="flex justify-end mt-4 space-x-2">
			<button class="btn-primary disabled:opacity-60 disabled:cursor-not-allowed" @click="doImport" :disabled="importableCount === 0">Import</button>
		</div>
	</div>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ExclamationCircleIcon } from '@heroicons/vue/20/solid';
	import { computed, ref, useTemplateRef } from 'vue';
	import { useRouter } from 'vue-router';
	
	import { StatementLine, db } from '../db.ts';
	import ComboBoxAccounts from '../components/ComboBoxAccounts.vue';
	import { ppWithCommodity } from '../display.ts';

	import importCsv from '../importers/csv.ts';
	import importOfxAutodetectVersion from '../importers/ofx.ts';
	import { AnnotatedStatementLine, annotateStatementLineDuplicates } from '../importers/deduplicate.ts';
	
	const fileInput = useTemplateRef('file');
	
	const format = ref('ofx');
	const selectedFilename = ref('');
	const sourceAccount = ref('');
	
	const router = useRouter();
	
	const statementLines = ref([] as AnnotatedStatementLine[]);
	const hideDuplicates = ref(false);

	const duplicateCount = computed(function() {
		return statementLines.value.filter((line) => line.duplicate).length;
	});

	const importableCount = computed(function() {
		return statementLines.value.filter((line) => !line.duplicate).length;
	});

	const filteredStatementLines = computed(function() {
		if (!hideDuplicates.value) {
			return statementLines.value;
		}
		return statementLines.value.filter((line) => !line.duplicate);
	});

	const importTotalsByCommodity = computed(function() {
		const totals = new Map<string, { debit: number; credit: number }>();
		for (const line of statementLines.value) {
			if (line.duplicate) {
				continue;
			}
			const entry = totals.get(line.commodity) ?? { debit: 0, credit: 0 };
			if (line.quantity >= 0) {
				entry.debit += line.quantity;
			} else {
				entry.credit += -line.quantity;
			}
			totals.set(line.commodity, entry);
		}
		return Array.from(totals.entries()).map(function([commodity, amount]) {
			return {
				commodity,
				debit: amount.debit,
				credit: amount.credit
			};
		});
	});

	const formattedImportTotals = computed(function() {
		return importTotalsByCommodity.value.map(function(entry) {
			return {
				commodity: entry.commodity,
				debitText: ppWithCommodity(entry.debit, entry.commodity),
				creditText: ppWithCommodity(entry.credit, entry.commodity)
			};
		});
	});
	
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
		
		if (!sourceAccount.value) {
			throw new Error('Select the source account before importing a statement.');
		}
		let importedLines: StatementLine[];
		if (format.value === 'csv') {
			importedLines = importCsv(sourceAccount.value, content);
		} else if (format.value === 'ofx') {
			importedLines = importOfxAutodetectVersion(sourceAccount.value, content);
		} else {
			throw new Error('Unexpected import format');
		}
		
		statementLines.value = await annotateStatementLineDuplicates(sourceAccount.value, importedLines);
	}

	function formatDuplicateStatus(line: AnnotatedStatementLine): string {
		if (!line.duplicate) {
			return 'Will import';
		}
		switch (line.duplicateReason) {
			case 'existing-fitid':
				return 'Duplicate (already imported – FITID)';
			case 'file-fitid':
				return 'Duplicate (within file – FITID)';
			case 'existing-signature':
				return 'Duplicate (already imported – date/amount/description)';
			case 'file-signature':
				return 'Duplicate (within file – date/amount/description)';
			case 'existing-date-amount':
				return 'Duplicate (already imported – date/amount)';
			case 'file-date-amount':
				return 'Duplicate (within file – date/amount)';
			default:
				return 'Duplicate';
		}
	}

	function duplicateMatchTooltip(line: AnnotatedStatementLine): string {
		if (!line.duplicate || !line.duplicateMatch) {
			return '';
		}
		if (line.duplicateMatch.kind === 'existing') {
			const match = line.duplicateMatch.statementLine;
			const amount = formatAmountForTooltip(match.quantity, match.commodity);
			const description = summariseDescription(match.description, match.name, match.memo);
			const parts = [`Existing line #${match.id}`, dayjs(match.dt).format('YYYY-MM-DD'), amount];
			if (description) {
				parts.push(description);
			}
			return parts.join(' · ');
		}
		const previous = line.duplicateMatch.previousLine;
		const amount = formatAmountForTooltip(previous.quantity, previous.commodity);
		const description = summariseDescription(previous.description, previous.name, previous.memo);
		const parts = ['Matches earlier line in this file', dayjs(previous.dt).format('YYYY-MM-DD'), amount];
		if (description) {
			parts.push(description);
		}
		return parts.join(' · ');
	}

	function summariseDescription(description: string, name?: string | null, memo?: string | null): string {
		const values = [description, name ?? '', memo ?? ''].map((value) => value.trim()).filter((value) => value.length > 0);
		return Array.from(new Set(values)).join(' · ');
	}

	function formatAmountForTooltip(quantity: number, commodity: string): string {
		if (quantity > 0) {
			return 'Dr ' + ppWithCommodity(quantity, commodity);
		}
		if (quantity < 0) {
			return 'Cr ' + ppWithCommodity(-quantity, commodity);
		}
		return ppWithCommodity(0, commodity);
	}

	function tableTextClass(line: AnnotatedStatementLine): string {
		return line.duplicate ? 'text-gray-600' : 'text-gray-900';
	}
	
	async function doImport() {
		// Import statement lines to database atomically
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		for (const line of statementLines.value) {
			if (line.duplicate) {
				continue;
			}
			await dbTransaction.execute(
				`INSERT INTO statement_lines (source_account, dt, name, memo, description, quantity, balance, commodity, fitid)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`,
				[line.source_account, line.dt, line.name, line.memo, line.description, line.quantity, line.balance, line.commodity, line.fitid]
			);
		}
		
		dbTransaction.commit();
		
		router.push({ name: 'statement-lines' });
	}
	
	const hasZeroAmounts = computed(function() {
		for (const line of statementLines.value) {
			if (line.quantity === 0) {
				return true;
			}
		}
		return false;
	});
</script>

<style scoped>
.sticky-table {
	@apply border-separate table-auto;
	border-spacing: 0;
}
.sticky-header th {
	@apply sticky top-0 bg-white z-10 border-b border-gray-300;
}
</style>
