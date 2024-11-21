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
	<h1 class="page-heading">
		Chart of accounts
	</h1>
	
	<div class="my-2 py-2 flex gap-x-2 items-baseline bg-white sticky top-0">
		<DropdownBox class="w-[450px]" :values="accountKindsByModule" v-model="selectedAccountKind" />
		<button class="btn-primary" @click="addAccountType">Add type</button>
		<button class="btn-secondary text-red-600 ring-red-500" @click="removeAccountType">Remove type</button>
	</div>
	
	<table class="min-w-full">
		<thead>
			<tr>
				<th></th>
				<th class="py-0.5 px-1 text-gray-900 font-semibold text-start">Account</th>
				<th class="py-0.5 pl-1 text-gray-900 font-semibold text-start">Associated types</th>
			</tr>
		</thead>
		<tbody>
			<tr class="border-t border-gray-300" v-for="[account, thisAccountKinds] in accounts.entries()">
				<td class="py-0.5 pr-1 text-gray-900 align-baseline"><input class="checkbox-primary" type="checkbox" v-model="selectedAccounts" :value="account"></td>
				<td class="py-0.5 px-1 text-gray-900 align-baseline">{{ account }}</td>
				<td class="py-0.5 pl-1 text-gray-900 align-baseline">
					<ul class="list-disc ml-5" v-if="thisAccountKinds">
						<!-- First display known account kinds -->
						<template v-for="[accountKind, accountKindPrettyName] in accountKindsMap.entries()">
							<li v-if="thisAccountKinds.indexOf(accountKind) >= 0">{{ accountKindPrettyName }}</li>
						</template>
						<!-- Then display unknown account kinds -->
						<template v-for="accountKind in thisAccountKinds">
							<li v-if="!accountKindsMap.has(accountKind)" class="italic">{{ accountKind }}</li>
						</template>
					</ul>
				</td>
			</tr>
		</tbody>
	</table>
</template>

<script setup lang="ts">
	import { ref } from 'vue';
	
	import { accountKinds } from '../registry.ts';
	import { db } from '../db.ts';
	import DropdownBox from '../components/DropdownBox.vue';
	
	const accountKindsMap = new Map(accountKinds);
	const accountKindsByModule = [...Map.groupBy(accountKinds, (k) => k[0].split('.')[0]).entries()];
	
	const accounts = ref(new Map<string, string[]>());
	const selectedAccounts = ref([]);
	const selectedAccountKind = ref(accountKinds[0]);
	
	async function load() {
		const session = await db.load();
		
		const accountKindsRaw: {account: string, kind: string}[] = await session.select(
			`SELECT account, kind FROM account_configurations ORDER BY account, kind`
		);
		
		for (const accountKindRaw of accountKindsRaw) {
			const kinds = accounts.value.get(accountKindRaw.account) ?? [];
			kinds.push(accountKindRaw.kind);
			accounts.value.set(accountKindRaw.account, kinds);
		}
	}
	
	load();
	
	async function addAccountType() {
		// Associate selected accounts with the selected account kind
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		for (const account of selectedAccounts.value) {
			await dbTransaction.execute(
				`INSERT INTO account_configurations (account, kind)
				VALUES ($1, $2)`,
				[account, selectedAccountKind.value[0]]
			);
		}
		
		await dbTransaction.commit();
		
		selectedAccounts.value = [];
		
		// Reload data
		accounts.value.clear();
		await load();
	}
	
	async function removeAccountType() {
		// De-associate selected accounts with the selected account kind
		const session = await db.load();
		const dbTransaction = await session.begin();
		
		for (const account of selectedAccounts.value) {
			await dbTransaction.execute(
				`DELETE FROM account_configurations
				WHERE account = $1 AND kind = $2`,
				[account, selectedAccountKind.value[0]]
			);
		}
		
		await dbTransaction.commit();
		
		selectedAccounts.value = [];
		
		// Reload data
		accounts.value.clear();
		await load();
	}
</script>
