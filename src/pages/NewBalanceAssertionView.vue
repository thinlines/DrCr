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
		New balance assertion
	</h1>
	
	<BalanceAssertionEditor :assertion="assertion" />
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ref } from 'vue';
	
	import BalanceAssertionEditor, { EditingAssertion } from '../components/BalanceAssertionEditor.vue';
	
	const assertion = ref({
		id: null,
		dt: dayjs().format('YYYY-MM-DD'),
		description: '',
		account: '',
		sign: 'dr',
		amount_abs: '',
	} as EditingAssertion);
</script>
