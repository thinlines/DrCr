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
	<h1 class="page-heading mb-4">
		Multiple CGT adjustments
	</h1>
	
	<div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
		<h2 class="col-span-2 text-xl text-gray-900 font-semibold">CGT assets</h2>
		
		<label for="account" class="block text-gray-900 pr-4">Account</label>
		<ComboBoxAccounts v-model="account" />
		<label for="commodity" class="block text-gray-900 pr-4">Commodity</label>
		<div>
			<input type="text" class="bordered-field" name="commodity" id="commodity" placeholder=" " v-model="commodity">
		</div>
		
		<div class="rounded-md bg-blue-50 p-4 col-span-2">
			<div class="flex">
				<div class="flex-shrink-0">
					<InformationCircleIcon class="w-5 h-5 text-blue-400" />
				</div>
				<div class="ml-3 flex-1">
					<p class="text-sm text-blue-700">The total cost adjustment will be distributed proportionally across all matching CGT assets.</p>
				</div>
			</div>
		</div>
		
		<h2 class="col-span-2 text-xl text-gray-900 font-semibold pt-4">CGT adjustment</h2>
		
		<label for="dt" class="block text-gray-900 pr-4">Adjustment date</label>
		<div>
			<input type="date" class="bordered-field" name="dt" id="dt" v-model="dt">
		</div>
		<label for="description" class="block text-gray-900 pr-4">Description</label>
		<div>
			<input type="text" class="bordered-field" name="description" id="description" placeholder=" " v-model="description">
		</div>
		<label for="cost_adjustment" class="block text-gray-900 pr-4">Total cost adjustment</label>
		<div class="relative shadow-sm">
			<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
				<span class="text-gray-500">{{ db.metadata.reporting_commodity }}</span>
			</div>
			<input type="number" class="bordered-field pl-7 pr-16" step="0.01" v-model="cost_adjustment_abs" placeholder="0.00">
			<div class="absolute inset-y-0 right-0 flex items-center">
				<select class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-indigo-600" v-model="sign">
					<option value="dr">Dr</option>
					<option value="cr">Cr</option>
				</select>
			</div>
		</div>
	</div>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-primary" @click="saveAdjustment">Save</button>
	</div>
	
	<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2" v-if="error !== null">
		<div class="flex">
			<div class="flex-shrink-0">
				<XCircleIcon class="h-5 w-5 text-red-400" />
			</div>
			<div class="ml-3 flex-1">
				<p class="text-sm text-red-700">{{ error }}</p>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { XCircleIcon } from '@heroicons/vue/24/solid';
	import { InformationCircleIcon } from '@heroicons/vue/20/solid';
	import { emit } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { ref } from 'vue';
	
	import { CGTAsset } from './cgt.ts';
	import ComboBoxAccounts from '../../components/ComboBoxAccounts.vue';
	import { DT_FORMAT, DeserialiseAmountError, JoinedTransactionPosting, db, deserialiseAmount } from '../../db.ts';
	import { ppWithCommodity } from '../../display.ts';
	import { CriticalError } from '../../error.ts';
	
	const account = ref('');
	const commodity = ref('');
	const dt = ref(dayjs().format('YYYY-MM-DD'));
	const description = ref('');
	const cost_adjustment_abs = ref(null! as number);
	const sign = ref('dr');
	
	const error = ref(null as string | null);
	
	async function saveAdjustment() {
		// TODO: Preview mode?
		
		error.value = null;
		
		let totalAdjustmentAbs;
		try {
			totalAdjustmentAbs = deserialiseAmount('' + cost_adjustment_abs.value);
		} catch (err) {
			if (err instanceof DeserialiseAmountError) {
				error.value = err.message;
				return;
			} else {
				throw err;
			}
		}
		
		const totalAdjustment = sign.value === 'dr' ? totalAdjustmentAbs.quantity : -totalAdjustmentAbs.quantity;
		
		// Get all postings to the CGT asset account
		const session = await db.load();
		const cgtPostings = await session.select(
			`SELECT *
			FROM joined_transactions
			WHERE account = $1`,
			[account.value]
		) as JoinedTransactionPosting[];
		
		// Process postings to determine final balances
		// Based on cgt.ts getCGTAssets
		const assets: CGTAsset[] = [];
		
		for (const posting of cgtPostings) {
			// Check for matching asset
			if (posting.commodity.indexOf(' {') < 0) {
				if (posting.commodity !== commodity.value) {
					continue;
				}
			} else {
				const postingCommodityName = posting.commodity.substring(0, posting.commodity.indexOf(' {'));
				if (postingCommodityName !== commodity.value) {
					continue;
				}
			}
			
			// This is a matching CGT asset
			
			if (posting.quantity >= 0) {
				// Debit CGT asset - create new CGTAsset
				assets.push(new CGTAsset(posting.quantity, posting.commodity, posting.account, posting.dt))
			} else {
				// Credit CGT asset
				// Currently only a full disposal of a CGT asset is implemented
				
				// Find matching CGT asset
				const asset = assets.find((a) => a.commodity === posting.commodity && a.account === posting.account);
				
				if (!asset) {
					throw new Error('Attempted credit of ' + ppWithCommodity(posting.quantity, posting.commodity) + ' without preceding debit balance');
				}
				if (asset.quantity + posting.quantity < 0) {
					throw new Error('Attempted credit of ' + ppWithCommodity(posting.quantity, posting.commodity) + ' which exceeds debit balance of ' + ppWithCommodity(asset.quantity, asset.commodity));
				}
				if (asset.quantity + posting.quantity != 0) {
					throw new Error('Partial disposal of CGT asset not implemented');
				}
				
				assets.splice(assets.indexOf(asset), 1);
			}
		}
		
		if (assets.length === 0) {
			error.value = 'No matching CGT assets';
			return;
		}
		
		// Distribute total adjustment across matching assets
		const totalQuantity = assets.reduce((acc, asset) => acc + asset.quantity, 0)
		const cgtAdjustments = [];
		
		for (let i = 0; i < assets.length; i++) {
			// This might be fractional at this stage
			cgtAdjustments[i] = totalAdjustment * assets[i].quantity / totalQuantity;
		}
		
		// Compute the difference due to rounding
		const roundingShortfall = cgtAdjustments.reduce((acc, adj) => acc - Math.floor(Math.abs(adj)), Math.abs(totalAdjustment));
		
		// Sort by largest remainder
		const largestRemainders = cgtAdjustments.map((adj, i) => [Math.abs(adj) - Math.floor(Math.abs(adj)), i]);
		largestRemainders.sort((a, b) => b[0] - a[0]);
		
		// Round up as many as required to equal the total adjustment
		let i = 0;
		for (; i < roundingShortfall; i++) {
			const assetIndex = largestRemainders[i][1];
			const cgtAdjustment: number = cgtAdjustments[assetIndex];
			
			// Round away from zero
			const cgtAdjustmentAbs = Math.floor(Math.abs(cgtAdjustment)) + 1;
			cgtAdjustments[assetIndex] = cgtAdjustmentAbs * Math.sign(cgtAdjustment);
		}
		
		// Round others down
		for (; i < largestRemainders.length; i++) {
			const assetIndex = largestRemainders[i][1];
			const cgtAdjustment: number = cgtAdjustments[assetIndex];
			
			// Round towards zero
			const cgtAdjustmentAbs = Math.floor(Math.abs(cgtAdjustment));
			cgtAdjustments[assetIndex] = cgtAdjustmentAbs * Math.sign(cgtAdjustment);
		}
		
		// Sanity check
		const totalRoundedAdjustment = cgtAdjustments.reduce((acc, adj) => acc + adj, 0);
		if (totalRoundedAdjustment !== totalAdjustment) {
			throw new CriticalError('Rounding unexpectedly changed total CGT adjustment amount');
		}
		
		// Add adjustments to database atomically
		const dbTransaction = await session.begin();
		for (let i = 0; i < assets.length; i++) {
			const asset = assets[i];
			const cgtAdjustment = cgtAdjustments[i];
			
			await dbTransaction.execute(
				`INSERT INTO austax_cgt_cost_adjustments (quantity, commodity, account, acquisition_dt, dt, description, cost_adjustment)
				VALUES ($1, $2, $3, $4, $5, $6, $7)`,
				[asset.quantity, asset.commodity, asset.account, asset.acquisition_dt, dayjs(dt.value).format(DT_FORMAT), description.value, cgtAdjustment]
			);
		}
		await dbTransaction.commit();
		
		await emit('cgt-adjustment-updated');
		await getCurrentWindow().close();
	}
</script>
