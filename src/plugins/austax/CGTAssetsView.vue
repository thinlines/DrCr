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
		CGT assets
	</h1>
	
	<table class="min-w-full">
		<thead>
			<tr>
				<th></th>
				<th></th>
				<th></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start border-l border-gray-300" colspan="2">Acquisition</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start border-l border-gray-300" colspan="2">Adjustment</th>
				<th class="print:hidden"></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start border-l border-gray-300" colspan="2">Disposal</th>
				<th class="border-l border-gray-300"></th>
			</tr>
			<tr>
				<th class="py-0.5 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 text-gray-900 font-semibold text-start">Asset</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Units</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start border-l border-gray-300">Date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Value</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end border-l border-gray-300">b/f&nbsp;</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">{{ eofyDate?.format('YYYY') }}</th>
				<th class="print:hidden"></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start border-l border-gray-300">Date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Value</th>
				<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end border-l border-gray-300">Gain&nbsp;</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="asset of cgtAssets" class="border-t border-gray-300">
				<td class="py-0.5 pr-1 text-gray-900">
					<RouterLink :to="{ name: 'transactions', params: { account: asset.account } }" class="hover:text-blue-700 hover:underline">{{ asset.account }}</RouterLink>
				</td>
				<td class="py-0.5 px-1 text-gray-900">{{ cgtAssetCommodityName(asset.commodity) }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(asset.quantity) }}</td>
				<td class="py-0.5 px-1 text-gray-900 border-l border-gray-300">{{ dayjs(asset.acquisition_dt).format('YYYY-MM-DD') }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(asCost(asset.quantity, asset.commodity)) }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end border-l border-gray-300" v-html="costAdjustmentBroughtForward(asset)"></td>
				<td class="py-0.5 px-1 text-gray-900 text-end" v-html="costAdjustmentCurrentPeriod(asset)"></td>
				<td class="py-0.5 px-1 text-gray-900 text-center print:hidden">
					<!--<a href="#" class="text-gray-500 hover:text-gray-700">
						<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
							<path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
						</svg>
					</a>-->
				</td>
				<td class="py-0.5 px-1 text-gray-900 border-l border-gray-300">{{ asset.disposal_dt !== null ? dayjs(asset.disposal_dt).format('YYYY-MM-DD') : '' }}</td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ asset.disposal_value !== null ? pp(asset.disposal_value) : '' }}</td>
				<td class="py-0.5 pl-1 text-gray-900 text-end border-l border-gray-300" v-html="cgtAssetGain(asset)"></td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ref } from 'vue';
	
	import { asCost } from '../../amounts.ts';
	import { CGTAsset, cgtAssetCommodityName, getCGTAssets } from './cgt.ts';
	import { db } from '../../db.ts';
	import { pp, ppBracketed } from '../../display.ts';
	
	const cgtAssets = ref([] as CGTAsset[]);
	const eofyDate = ref(null as dayjs.Dayjs | null);
	
	async function load() {
		// Load CGT assets
		const session = await db.load();
		cgtAssets.value = await getCGTAssets(session);
		eofyDate.value = dayjs(db.metadata.eofy_date);
		
		console.log(cgtAssets.value);
	}
	
	load();
	
	function costAdjustmentBroughtForward(asset: CGTAsset): string {
		const thisEofyDate = eofyDate.value!;
		const lastEofyDate = thisEofyDate.set('year', thisEofyDate.year() - 1);
		
		let total = 0;
		for (const costAdjustment of asset.cost_adjustments) {
			if (!dayjs(costAdjustment.dt).isAfter(lastEofyDate)) {
				total += costAdjustment.cost_adjustment;
			}
		}
		
		if (total !== 0) {
			// TODO: Link to CGT adjustments page
			return ppBracketed(total);
		} else {
			return '';
		}
	}
	
	function costAdjustmentCurrentPeriod(asset: CGTAsset): string {
		const thisEofyDate = eofyDate.value!;
		const lastEofyDate = thisEofyDate.set('year', thisEofyDate.year() - 1);
		
		let total = 0;
		for (const costAdjustment of asset.cost_adjustments) {
			if (dayjs(costAdjustment.dt).isAfter(lastEofyDate) && !dayjs(costAdjustment.dt).isAfter(thisEofyDate)) {
				total += costAdjustment.cost_adjustment;
			}
		}
		
		if (total !== 0) {
			// TODO: Link to CGT adjustments page
			return ppBracketed(total);
		} else {
			return '';
		}
	}
	
	function cgtAssetGain(asset: CGTAsset): string {
		if (asset.disposal_dt === null) {
			return '';
		}
		
		// Get total cost adjustments
		let totalCostAdjustment = 0;
		for (const costAdjustment of asset.cost_adjustments) {
			totalCostAdjustment += costAdjustment.cost_adjustment;
		}
		
		const netGain = asset.disposal_value! - (asCost(asset.quantity, asset.commodity) + totalCostAdjustment);
		
		return ppBracketed(netGain);
	}
</script>
