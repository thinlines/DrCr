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
	<h1 class="page-heading">
		CGT adjustments
	</h1>
	
	<div class="my-4 flex gap-x-2">
		<a :href="$router.resolve({name: 'cgt-adjustments-new'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New CGT adjustment
		</a>
		<a :href="$router.resolve({name: 'cgt-adjustments-multinew'}).fullPath" class="btn-secondary pl-2 text-emerald-700 ring-emerald-600" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			Multiple CGT adjustments
		</a>
	</div>
	
	<table class="min-w-full">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Asset</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Units</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Acquisition date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Acquisition value</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Adjustment date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Cost adjustment&nbsp;</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="cgt_adjustment of cgtAdjustments">
				<td class="py-0.5 pr-1 text-gray-900">{{ cgt_adjustment.account }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ cgtAssetCommodityName(cgt_adjustment.commodity) }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(cgt_adjustment.quantity) }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ dayjs(cgt_adjustment.acquisition_dt).format('YYYY-MM-DD') }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(asCost(cgt_adjustment.quantity, cgt_adjustment.commodity)) }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ dayjs(cgt_adjustment.dt).format('YYYY-MM-DD') }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ cgt_adjustment.description }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end" v-html="ppBracketed(cgt_adjustment.cost_adjustment)"></td>
				<td class="py-0.5 pl-1 text-end">
					<a :href="$router.resolve({name: 'cgt-adjustments-edit', params: {id: cgt_adjustment.id}}).fullPath" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">
						<PencilIcon class="w-4 h-4" />
					</a>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { PencilIcon } from '@heroicons/vue/24/outline';
	import { PlusIcon } from '@heroicons/vue/16/solid';
	import { UnlistenFn, listen } from '@tauri-apps/api/event';
	import { onUnmounted, ref } from 'vue';
	
	import { CGTAdjustment, cgtAssetCommodityName } from './cgt.ts';
	import { asCost } from '../../amounts.ts';
	import { db } from '../../db.ts';
	import { pp, ppBracketed } from '../../display.ts';
	
	const cgtAdjustments = ref([] as CGTAdjustment[]);
	
	async function load() {
		// Load CGT adjustments from database
		const session = await db.load();
		cgtAdjustments.value = await session.select(
			`SELECT id, quantity, commodity, account, acquisition_dt, dt, description, cost_adjustment
			FROM austax_cgt_cost_adjustments
			ORDER BY dt DESC, account, substr(commodity, 1, instr(commodity, ' {')), acquisition_dt DESC, id DESC`
		);
	}
	
	load();
	
	// Refresh CGT adjustments list when CGT adjustment updated
	let unlistenAdjustmentUpdated: UnlistenFn | null = null;
	(async () => {
		// Cannot await at top level without <Suspense> therefore do this in an async function
		unlistenAdjustmentUpdated = await listen('cgt-adjustment-updated', async (_event) => { await load(); });
	})();
	
	onUnmounted(() => {
		if (unlistenAdjustmentUpdated !== null) {
			unlistenAdjustmentUpdated();
		}
	});
</script>
