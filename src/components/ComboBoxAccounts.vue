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
	<ComboBox :values="accounts" :inputClass="inputClass" />
</template>

<script setup lang="ts">
	import { defineProps, ref } from 'vue';
	
	import { db } from '../db.ts';
	import ComboBox from './ComboBox.vue';
	
	const { inputClass } = defineProps<{ inputClass?: string }>();
	
	const accounts = ref([] as string[]);
	
	async function load() {
		// Load account names
		const session = await db.load();
		
		const rawAccounts: {account: string}[] = await session.select(
			`SELECT DISTINCT account
			FROM postings
			ORDER BY account`
		);
		
		accounts.value = rawAccounts.map((a) => a.account);
	}
	load();
</script>
