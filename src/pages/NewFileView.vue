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
		New file
	</h1>
	
	<div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
		<label for="eofy_date" class="block text-gray-900 pr-4">End of financial year</label>
		<div>
			<input type="date" class="bordered-field" id="eofy_date" v-model="eofy_date">
		</div>
		<label for="reporting_commodity" class="block text-gray-900 pr-4">Reporting currency</label>
		<div>
			<input type="text" class="bordered-field text-gray-500" id="reporting_commodity" v-model="reporting_commodity" disabled>
		</div>
		<label for="amount_dps" class="block text-gray-900 pr-4">Decimal places</label>
		<div>
			<input type="number" class="bordered-field text-gray-500" id="amount_dps" v-model="amount_dps" disabled>
		</div>
	</div>
	
	<div class="flex justify-end mt-4 space-x-2">
		<button class="btn-primary" @click="createNewFile">OK</button>
	</div>
</template>

<script setup type="ts">
	import dayjs from 'dayjs';
	import { save } from '@tauri-apps/plugin-dialog';
	import { ref } from 'vue';
	import { useRouter } from 'vue-router';
	
	import { createNewDatabase, db } from '../db.ts';
	
	// Get initial EOFY date
	let eofy_dayjs = dayjs().set('month', 5).set('date', 30);
	if (eofy_dayjs.isBefore(dayjs())) {
		eofy_dayjs.set('year', eofy_dayjs.year() + 1);
	}
	
	const eofy_date = ref(eofy_dayjs.format('YYYY-MM-DD'));
	const reporting_commodity = ref('$');
	const amount_dps = ref(2);
	
	const router = useRouter();
	
	async function createNewFile() {
		const file = await save({
			filters: [
				{ name: 'DrCr database (SQLite)', extensions: ['db', 'sqlite', 'sqlite3', 'sql'] }
			],
		});
		
		if (file !== null) {
			// Create new database
			await createNewDatabase(file, dayjs(eofy_date.value).format('YYYY-MM-DD'), reporting_commodity.value, amount_dps.value);
			
			// Load the database
			await db.init(file);
			router.push({ name: 'index' });
		}
	}
</script>
