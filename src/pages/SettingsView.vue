<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2025  Contributors
-->

<template>
    <h1 class="page-heading mb-4">Settings</h1>

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
        <div></div>
        <p class="text-xs text-gray-500 mt-1">
            Only month and day are used. Stored as the next upcoming occurrence.
        </p>
    </div>

    <div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
        <label for="thousands-sep" class="block text-gray-900 pr-4">Thousands separator</label>
        <div>
            <select id="thousands-sep" class="bordered-field" v-model="placeSeparator">
                <option :value="'\u202F'">Space (thin, non-breaking)</option>
                <option :value="','" >Comma</option>
                <option :value="''">None</option>
            </select>
            <p class="text-xs text-gray-500 mt-1">Used between every three digits in whole numbers.</p>
        </div>

        <label for="decimal-sep" class="block text-gray-900 pr-4">Decimal separator</label>
        <div>
            <select id="decimal-sep" class="bordered-field" v-model="decimalSeparator">
                <option value=".">Dot (.)</option>
                <option value=",">Comma (,)</option>
            </select>
        </div>
    </div>

    <div class="flex justify-end mt-4 space-x-2">
        <button class="btn-primary" @click="save" :disabled="saving">Save</button>
    </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs';
import { computed, ref, watch } from 'vue';

import { db } from '../db.ts';

// Initial EOFY from metadata
const initialEofy = dayjs(db.metadata.eofy_date);
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
const eofyMonth = ref(initialEofy.month() + 1);
const eofyDay = ref(Math.min(initialEofy.date(), monthLengths[eofyMonth.value]));
const daysInSelectedMonth = computed(() => Array.from({ length: monthLengths[eofyMonth.value] }, (_, i) => i + 1));
watch(eofyMonth, () => {
  if (eofyDay.value > monthLengths[eofyMonth.value]) {
    eofyDay.value = monthLengths[eofyMonth.value];
  }
});
const saving = ref(false);

// Number formatting settings
const placeSeparator = ref<string>(db.metadata.place_separator ?? '\u202F');
const decimalSeparator = ref<string>(db.metadata.decimal_separator ?? '.');

async function save() {
    try {
        saving.value = true;
        // Normalise EOFY month/day to next upcoming concrete date (ignore year entirely)
        const today = dayjs();
        let newEofyDayjs = today.set('month', eofyMonth.value - 1).set('date', eofyDay.value);
        if (newEofyDayjs.isBefore(today)) {
            newEofyDayjs = newEofyDayjs.add(1, 'year');
        }
        const newEofy = newEofyDayjs.format('YYYY-MM-DD');

        const session = await db.load();
        const tx = await session.begin();

        // EOFY date
        await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'eofy_date'`, [newEofy]);

        // Thousands/place separator (upsert)
        let res = await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'place_separator'`, [placeSeparator.value]);
        if (res.rowsAffected === 0) {
            await tx.execute(`INSERT INTO metadata (key, value) VALUES ('place_separator', ?)`, [placeSeparator.value]);
        }

        // Decimal separator (upsert)
        res = await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'decimal_separator'`, [decimalSeparator.value]);
        if (res.rowsAffected === 0) {
            await tx.execute(`INSERT INTO metadata (key, value) VALUES ('decimal_separator', ?)`, [decimalSeparator.value]);
        }

        await tx.commit();

        // Update reactive cache
        db.metadata.eofy_date = newEofy;
        db.metadata.place_separator = placeSeparator.value;
        db.metadata.decimal_separator = decimalSeparator.value;
    } finally {
        saving.value = false;
    }
}
</script>
