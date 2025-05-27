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
		Balance assertions
	</h1>
	
	<div class="my-4 flex gap-x-2">
		<a :href="$router.resolve({name: 'balance-assertions-new'}).fullPath" class="btn-primary pl-2" onclick="return openLinkInNewWindow(this);">
			<PlusIcon class="w-4 h-4" />
			New assertion
		</a>
	</div>
	
	<table class="min-w-full">
		<thead>
			<tr class="border-b border-gray-300">
				<th class="py-0.5 pr-1 text-gray-900 font-semibold text-start">Date</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Description</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-end">Balance</th>
				<th></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Status</th>
				<th></th>
			</tr>
		</thead>
		<tbody>
			<tr v-for="assertion of balanceAssertions">
				<td class="py-0.5 pr-1 text-gray-900">{{ dayjs(assertion.dt).format('YYYY-MM-DD') }}</td>
				<td class="py-0.5 px-1 text-gray-900">{{ assertion.description }}</td>
				<td class="py-0.5 px-1 text-gray-900"><RouterLink :to="{ name: 'transactions', params: { account: assertion.account } }" class="text-gray-900 hover:text-blue-700 hover:underline">{{ assertion.account }}</RouterLink></td>
				<td class="py-0.5 px-1 text-gray-900 text-end">{{ pp(Math.abs(assertion.quantity)) }}</td>
				<td class="py-0.5 pr-1 text-gray-900">{{ assertion.quantity >= 0 ? 'Dr' : 'Cr' }}</td>
				<td class="py-0.5 px-1 text-gray-900">
					<CheckIcon class="w-4 h-4" v-if="assertion.is_valid" />
					<XMarkIcon class="w-4 h-4 text-red-500" v-if="!assertion.is_valid" />
				</td>
				<td class="py-0.5 pl-1 text-gray-900 text-end">
					<a :href="'/balance-assertions/edit/' + assertion.id" class="text-gray-500 hover:text-gray-700" onclick="return openLinkInNewWindow(this);">
						<PencilIcon class="w-4 h-4" />
					</a>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { CheckIcon, PencilIcon, XMarkIcon } from '@heroicons/vue/24/outline';
	import { PlusIcon } from '@heroicons/vue/16/solid';
	import { invoke } from '@tauri-apps/api/core';
	import { ref } from 'vue';
	
	import { pp } from '../display.ts';
	
	const balanceAssertions = ref([] as ValidatedBalanceAssertion[]);
	
	interface ValidatedBalanceAssertion {
		id: number,
		dt: string,
		description: string,
		account: string,
		quantity: number,
		commodity: string,
		is_valid: boolean,
	}
	
	async function load() {
		balanceAssertions.value = JSON.parse(await invoke('get_validated_balance_assertions'));
	}
	
	load();
</script>
