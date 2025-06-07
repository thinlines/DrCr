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
		<label for="dt" class="block text-gray-900 pr-4">Date</label>
		<div>
			<input type="date" class="bordered-field" id="dt" v-model="assertion.dt">
		</div>
		<label for="description" class="block text-gray-900 pr-4">Description</label>
		<div>
			<input type="text" class="bordered-field" id="description" v-model="assertion.description" placeholder=" ">
		</div>
		<label for="account" class="block text-gray-900 pr-4">Account</label>
		<ComboBoxAccounts v-model="assertion.account" />
		<label for="amount" class="block text-gray-900 pr-4">Balance</label>
		<div class="relative shadow-sm">
			<div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
				<span class="text-gray-500">{{ db.metadata.reporting_commodity }}</span>
			</div>
			<input type="number" class="bordered-field pl-7 pr-16" step="0.01" v-model="assertion.amount_abs" placeholder="0.00">
			<div class="absolute inset-y-0 right-0 flex items-center">
				<select class="h-full border-0 bg-transparent py-0 pl-2 pr-8 text-gray-900 focus:ring-2 focus:ring-inset focus:ring-indigo-600" v-model="assertion.sign">
					<option value="dr">Dr</option>
					<option value="cr">Cr</option>
				</select>
			</div>
		</div>
	</div>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-secondary text-red-600 ring-red-500" @click="deleteAssertion" v-if="assertion.id !== null">Delete</button>
		<button class="btn-primary" @click="saveAssertion">Save</button>
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
	
	import { DeserialiseAmountError, DT_FORMAT, db, deserialiseAmount } from '../db.ts';
	import ComboBoxAccounts from './ComboBoxAccounts.vue';
	
	export interface EditingAssertion {
		id: number | null,
		dt: string,
		description: string,
		account: string,
		sign: string,
		amount_abs: string,
	}
	
	const { assertion } = defineProps<{ assertion: EditingAssertion }>();
	
	const error = ref(null as string | null);
	
	async function saveAssertion() {
		// Save changes to the assertion
		error.value = null;
		
		let amount_abs;
		try {
			amount_abs = deserialiseAmount('' + assertion.amount_abs);
		} catch (err) {
			if (err instanceof DeserialiseAmountError) {
				error.value = err.message;
				return;
			} else {
				throw err;
			}
		}
		const quantity = assertion.sign === 'dr' ? amount_abs.quantity : -amount_abs.quantity;
		
		const session = await db.load();
		
		if (assertion.id === null) {
			const result = await session.execute(
				`INSERT INTO balance_assertions (dt, description, account, quantity, commodity)
				VALUES ($1, $2, $3, $4, $5)`,
				[dayjs(assertion.dt).format(DT_FORMAT), assertion.description, assertion.account, quantity, amount_abs.commodity]
			);
			assertion.id = result.lastInsertId!;
		} else {
			await session.execute(
				`UPDATE balance_assertions
				SET dt = $1, description = $2, account = $3, quantity = $4, commodity = $5
				WHERE id = $6`,
				[dayjs(assertion.dt).format(DT_FORMAT), assertion.description, assertion.account, quantity, amount_abs.commodity, assertion.id]
			);
		}
		
		await emit('balance-assertion-updated', {id: assertion.id});
		await getCurrentWindow().close();
	}
	
	async function deleteAssertion() {
		// Delete the current assertion
		if (!await confirm('Are you sure you want to delete this balance assertion? This operation is irreversible.')) {
			return;
		}
		
		const session = await db.load();
		
		await session.execute(
			`DELETE FROM balance_assertions
			WHERE id = $1`,
			[assertion.id]
		);
		
		await emit('balance-assertion-updated', {id: assertion.id});
		await getCurrentWindow().close();
	}
</script>
