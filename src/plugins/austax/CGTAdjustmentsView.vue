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
		<!--<a :href="$router.resolve({name: 'cgt-adjustments-new'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New CGT adjustment
		</a>-->
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
					<!--<a href="#" class="text-gray-500 hover:text-gray-700">
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4 inline align-middle -mt-0.5">
							<path stroke-linecap="round" stroke-linejoin="round" d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L6.832 19.82a4.5 4.5 0 0 1-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 0 1 1.13-1.897L16.863 4.487Zm0 0L19.5 7.125" />
						</svg>
					</a>-->
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ref } from 'vue';
	
	import { CGTAdjustment } from './model.ts';
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
			ORDER BY dt DESC, account, substr(commodity, 1, instr(commodity, ' {')), acquisition_dt, id DESC`
		);
	}
	
	load();
	
	function cgtAssetCommodityName(commodity: string): string {
		return commodity.substring(0, commodity.indexOf(' {'));
	}
</script>
