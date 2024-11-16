<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2024  Lee Yingtong Li (RunasSudo)
	
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
		Trial balance
	</h1>
	
	<table class="min-w-full">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Dr</th>
				<th class="py-0.5 pl-1 text-gray-900 font-semibold text-end">Cr</th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="account in accounts">
				<td class="py-0.5 pr-1 text-gray-900"><RouterLink :to="{ name: 'transactions', params: { 'account': account.account } }" class="hover:text-blue-700 hover:underline">{{ account.account }}</RouterLink></td>
				<td class="py-0.5 px-1 text-gray-900 text-end">
					<template v-if="account.quantity >= 0">{{ pp(account.quantity) }}</template>
				</td>
				<td class="py-0.5 pl-1 text-gray-900 text-end">
					<template v-if="account.quantity < 0">{{ pp(-account.quantity) }}</template>
				</td>
			</tr>
			<tr>
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Total</th>
				<th class="py-0.5 px-1 text-gray-900 text-end">{{ pp(total_dr) }}</th>
				<th class="py-0.5 pl-1 text-gray-900 text-end">{{ pp(-total_cr) }}</th>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import { computed, ref } from 'vue';
	
	import { db, totalBalances } from '../db.ts';
	import { pp } from '../display.ts';
	
	const accounts = ref([] as {account: string, quantity: number}[]);
	
	const total_dr = computed(() => accounts.value.reduce((acc, x) => x.quantity > 0 ? acc + x.quantity : acc, 0));
	const total_cr = computed(() => accounts.value.reduce((acc, x) => x.quantity < 0 ? acc + x.quantity : acc, 0));
	
	async function load() {
		const session = await db.load();
		accounts.value = await totalBalances(session);
	}
	load();
</script>
