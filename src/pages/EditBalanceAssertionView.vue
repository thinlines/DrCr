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
		Edit balance assertion
	</h1>
	
	<BalanceAssertionEditor :assertion="assertion" />
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	
	import { ref } from 'vue';
	import { useRoute } from 'vue-router';
	
	import { db, serialiseAmount } from '../db.ts';
	import BalanceAssertionEditor, { EditingAssertion } from '../components/BalanceAssertionEditor.vue';
	
	const route = useRoute();
	
	const assertion = ref({
		id: null,
		dt: null!,
		description: null!,
		account: null!,
		sign: null!,
		amount_abs: null!,
	} as EditingAssertion);
	
	async function load() {
		const session = await db.load();
		
		const rawAssertions: any[] = await session.select(
			`SELECT *
			FROM balance_assertions
			WHERE id = $1`,
			[route.params.id]
		);
		const rawAssertion = rawAssertions[0];
		
		// Format parameters for display
		rawAssertion.dt = dayjs(rawAssertion.dt).format('YYYY-MM-DD');
		rawAssertion.sign = rawAssertion.quantity >= 0 ? 'dr' : 'cr';
		rawAssertion.amount_abs = serialiseAmount(Math.abs(rawAssertion.quantity), rawAssertion.commodity);
		
		assertion.value = rawAssertion as EditingAssertion;
	}
	
	load();
</script>
