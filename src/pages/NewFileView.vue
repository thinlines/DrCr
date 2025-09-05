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
        <label for="eofy_month" class="block text-gray-900 pr-4">End of financial year</label>
        <div class="flex items-center gap-2">
            <select id="eofy_month" class="bordered-field" v-model.number="eofyMonth">
                <option v-for="m in months" :key="m.value" :value="m.value">{{ m.label }}</option>
            </select>
            <select id="eofy_day" class="bordered-field" v-model.number="eofyDay">
                <option v-for="d in daysInSelectedMonth" :key="d" :value="d">{{ d }}</option>
            </select>
        </div>
        <div>
            <label for="reporting_commodity" class="block text-gray-900 pr-4">Reporting currency</label>
        </div>
        <div>
            <input type="text" class="bordered-field text-gray-500" id="reporting_commodity"
                v-model="reporting_commodity" disabled>
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

<script setup lang="ts">
import dayjs from 'dayjs';
import { save } from '@tauri-apps/plugin-dialog';
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { createNewDatabase, db } from '../db.ts';

// Month/day inputs for EOFY (default June 30)
const months = [
    { value: 1, label: 'January' },
    { value: 2, label: 'February' },
    { value: 3, label: 'March' },
    { value: 4, label: 'April' },
    { value: 5, label: 'May' },
    { value: 6, label: 'June' },
    { value: 7, label: 'July' },
    { value: 8, label: 'August' },
    { value: 9, label: 'September' },
    { value: 10, label: 'October' },
    { value: 11, label: 'November' },
    { value: 12, label: 'December' },
];
const monthLengths = { 1: 31, 2: 28, 3: 31, 4: 30, 5: 31, 6: 30, 7: 31, 8: 31, 9: 30, 10: 31, 11: 30, 12: 31 } as Record<number, number>;
const eofyMonth = ref(6);
const eofyDay = ref(30);
const daysInSelectedMonth = computed(() => Array.from({ length: monthLengths[eofyMonth.value] }, (_, i) => i + 1));
watch(eofyMonth, () => {
    if (eofyDay.value > monthLengths[eofyMonth.value]) {
        eofyDay.value = monthLengths[eofyMonth.value];
    }
});

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
        // Normalise EOFY to next upcoming occurrence
        const today = dayjs();
        let newEofy = today.set('month', eofyMonth.value - 1).set('date', eofyDay.value);
        if (newEofy.isBefore(today)) {
            newEofy = newEofy.add(1, 'year');
        }
        // Create new database
        await createNewDatabase(file, newEofy.format('YYYY-MM-DD'), reporting_commodity.value, amount_dps.value);

        // Load the database
        await db.init(file);
        router.push({ name: 'index' });
    }
}
</script>
