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
	<h1 class="page-heading">
		General ledger
	</h1>
	
	<div class="my-4 flex gap-x-2 items-center">
		<!-- Use a rather than RouterLink because RouterLink adds its own event handler -->
		<a :href="$router.resolve({name: 'journal-new-transaction'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New transaction
		</a>
		<div class="flex items-baseline">
			<input id="only-unclassified" class="ml-3 mr-1 self-center checkbox-primary" type="checkbox" v-model="commodityDetail">
			<label for="only-unclassified" class="text-gray-900">Show commodity detail</label>
		</div>
	</div>
	
	<div id="transaction-list" class="max-h-[100vh] overflow-y-scroll wk-aa">
		<table class="min-w-full">
			<thead>
				<tr>
					<th class="py-0.5 pr-1 text-gray-900 font-semibold lg:w-[12ex] text-start">Date</th>
					<th class="py-0.5 px-1 text-gray-900 font-semibold text-start" colspan="3">Description</th>
					<template v-if="commodityDetail">
						<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
						<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Cr</th>
					</template>
					<template v-if="!commodityDetail">
						<th class="py-0.5 px-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Dr</th>
						<th class="py-0.5 pl-1 text-gray-900 font-semibold lg:w-[12ex] text-end">Cr</th>
					</template>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td colspan="6">Loading data…</td>
				</tr>
			</tbody>
		</table>
	</div>
</template>

<script setup lang="ts">
	import Clusterize from 'clusterize.js';
	
	import dayjs from 'dayjs';
	
	import { PencilIcon, PlusIcon } from '@heroicons/vue/24/outline';
	
	import { onUnmounted, ref, watch } from 'vue';
	
	import { Transaction, db } from '../db.ts';
	import { pp, ppWithCommodity } from '../display.ts';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	import { renderComponent } from '../webutil.ts';
	
	const commodityDetail = ref(false);
	
	const transactions = ref([] as Transaction[]);
	let clusterize: Clusterize | null = null;
	
	async function load() {
		const session = await db.load();
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session);
		
		transactions.value = reportingWorkflow.getTransactionsAtStage(ReportingStage.FINAL_STAGE);
		
		// Display transactions in reverse chronological order
		// We must sort here because they are returned by reportingWorkflow in order of ReportingStage
		transactions.value.sort((a, b) => (b.dt.localeCompare(a.dt)) || ((b.id ?? 0) - (a.id ?? 0)));
	}
	
	function renderTable() {
		const PencilIconHTML = renderComponent(PencilIcon, { 'class': 'w-4 h-4 inline align-middle -mt-0.5' });  // Pre-render the pencil icon
		const rows = [];
		
		for (const transaction of transactions.value) {
			let editLink = '';
			if (transaction.id !== null) {
				editLink = `<a href="/journal/edit/${ transaction.id }" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">${ PencilIconHTML }</a>`;
			}
			rows.push(
				`<tr class="border-t border-gray-300">
					<td class="py-0.5 pr-1 text-gray-900 lg:w-[12ex]">${ dayjs(transaction.dt).format('YYYY-MM-DD') }</td>
					<td class="py-0.5 px-1 text-gray-900" colspan="3">${ transaction.description } ${ editLink }</td>
					<td></td>
					<td></td>
				</tr>`
			);
			
			for (const posting of transaction.postings) {
				if (commodityDetail.value) {
					rows.push(
						`<tr>
							<td class=""></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]">${ posting.description ?? '' }</td>
							<td class="py-0.5 px-1 text-gray-900 text-end"><i>${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</i></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]"><a href="/transactions/${ encodeURIComponent(posting.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ posting.account }</a></td>
							<td class="py-0.5 px-1 text-gray-900 text-end">
								${ posting.quantity >= 0 ? ppWithCommodity(posting.quantity, posting.commodity) : '' }
							</td>
							<td class="py-0.5 pl-1 text-gray-900 text-end">
								${ posting.quantity < 0 ? ppWithCommodity(-posting.quantity, posting.commodity) : '' }
							</td>
						</tr>`
					);
				} else {
					rows.push(
						`<tr>
							<td class=""></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]">${ posting.description ?? '' }</td>
							<td class="py-0.5 px-1 text-gray-900 text-end"><i>${ posting.quantity >= 0 ? 'Dr' : 'Cr' }</i></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[30%]"><a href="/transactions/${ encodeURIComponent(posting.account) }" class="text-gray-900 hover:text-blue-700 hover:underline">${ posting.account }</a></td>
							<td class="py-0.5 px-1 text-gray-900 lg:w-[12ex] text-end">
								${ posting.quantity >= 0 ? pp(posting.quantity_ascost!) : '' }
							</td>
							<td class="py-0.5 pl-1 text-gray-900 lg:w-[12ex] text-end">
								${ posting.quantity < 0 ? pp(-posting.quantity_ascost!) : '' }
							</td>
						</tr>`
					);
				}
			}
		}
		
		if (clusterize === null) {
			clusterize = new Clusterize({
				'rows': rows,
				scrollElem: document.getElementById('transaction-list')!,
				contentElem: document.querySelector('#transaction-list tbody')!,
				show_no_data_row: false,
			});
		} else {
			clusterize.update(rows);
		}
	}
	
	watch(commodityDetail, renderTable);
	watch(transactions, renderTable);
	
	load();
	
	onUnmounted(() => {
		if (clusterize !== null) {
			clusterize.destroy();
		}
	});
</script>
