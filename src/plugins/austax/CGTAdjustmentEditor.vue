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
	<div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
		<h2 class="col-span-2 text-xl text-gray-900 font-semibold">CGT asset</h2>
		
		<label for="acquisition_date" class="block text-gray-900 pr-4">Acquisition date</label>
		<div>
			<input type="date" class="bordered-field" name="acquisition_date" id="acquisition_date" v-model="adjustment.acquisition_dt">
		</div>
		<label for="account" class="block text-gray-900 pr-4">Account</label>
		<ComboBoxAccounts v-model="adjustment.account" />
		<label for="asset" class="block text-gray-900 pr-4">Asset</label>
		<div>
			<input type="text" class="bordered-field" name="asset" id="asset" placeholder=" " v-model="adjustment.asset">
		</div>
		
		<h2 class="col-span-2 text-xl text-gray-900 font-semibold pt-4">CGT adjustment</h2>
		
		<label for="dt" class="block text-gray-900 pr-4">Adjustment date</label>
		<div>
			<input type="date" class="bordered-field" name="dt" id="dt" v-model="adjustment.dt">
		</div>
		<label for="description" class="block text-gray-900 pr-4">Description</label>
		<div>
			<input type="text" class="bordered-field" name="description" id="description" placeholder=" " v-model="adjustment.description">
		</div>
		<label for="cost_adjustment" class="block text-gray-900 pr-4">Cost adjustment</label>
		<div class="relative shadow-sm">
			<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
				<span class="text-gray-500">{{ db.metadata.reporting_commodity }}</span>
			</div>
			<input type="number" class="bordered-field pl-7 pr-16" step="0.01" v-model="adjustment.cost_adjustment_abs" placeholder="0.00">
			<div class="absolute inset-y-0 right-0 flex items-center">
				<select class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-indigo-600" v-model="adjustment.sign">
					<option value="dr">Dr</option>
					<option value="cr">Cr</option>
				</select>
			</div>
		</div>
	</div>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-secondary text-red-600 ring-red-500" @click="deleteAdjustment" v-if="adjustment.id !== null">Delete</button>
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
	import { emit } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { ref } from 'vue';
	
	import ComboBoxAccounts from '../../components/ComboBoxAccounts.vue';
	import { DT_FORMAT, DeserialiseAmountError, db, deserialiseAmount } from '../../db.ts';
	
	export interface EditingCGTAdjustment {
		id: number | null,
		asset: string,
		account: string,
		acquisition_dt: string,
		dt: string,
		description: string,
		sign: string,
		cost_adjustment_abs: string,
	}
	
	const { adjustment } = defineProps<{ adjustment: EditingCGTAdjustment }>();
	
	const error = ref(null as string | null);
	
	async function saveAdjustment() {
		// Save changes to the CGT adjustment
		error.value = null;
		
		let asset;
		try {
			asset = deserialiseAmount('' + adjustment.asset);
		} catch (err) {
			if (err instanceof DeserialiseAmountError) {
				error.value = err.message;
				return;
			} else {
				throw err;
			}
		}
		
		let cost_adjustment_abs;
		try {
			cost_adjustment_abs = deserialiseAmount('' + adjustment.cost_adjustment_abs);
		} catch (err) {
			if (err instanceof DeserialiseAmountError) {
				error.value = err.message;
				return;
			} else {
				throw err;
			}
		}
		
		const cost_adjustment = adjustment.sign === 'dr' ? cost_adjustment_abs.quantity : -cost_adjustment_abs.quantity;
		
		const session = await db.load();
		
		if (adjustment.id === null) {
			await session.execute(
				`INSERT INTO austax_cgt_cost_adjustments (quantity, commodity, account, acquisition_dt, dt, description, cost_adjustment)
				VALUES ($1, $2, $3, $4, $5, $6, $7)`,
				[asset.quantity, asset.commodity, adjustment.account, dayjs(adjustment.acquisition_dt).format(DT_FORMAT), dayjs(adjustment.dt).format(DT_FORMAT), adjustment.description, cost_adjustment]
			);
		} else {
			await session.execute(
				`UPDATE austax_cgt_cost_adjustments
				SET quantity = $1, commodity = $2, account = $3, acquisition_dt = $4, dt = $5, description = $6, cost_adjustment = $7
				WHERE id = $8`,
				[asset.quantity, asset.commodity, adjustment.account, dayjs(adjustment.acquisition_dt).format(DT_FORMAT), dayjs(adjustment.dt).format(DT_FORMAT), adjustment.description, cost_adjustment, adjustment.id]
			);
		}
		
		await emit('cgt-adjustment-updated');
		await getCurrentWindow().close();
	}
	
	async function deleteAdjustment() {
		// Delete the current CGT adjustment
		if (!await confirm('Are you sure you want to delete this CGT adjustment? This operation is irreversible.')) {
			return;
		}
		
		const session = await db.load();
		
		await session.execute(
			`DELETE FROM austax_cgt_cost_adjustments
			WHERE id = $1`,
			[adjustment.id]
		);
		
		await emit('cgt-adjustment-updated');
		await getCurrentWindow().close();
	}
</script>
