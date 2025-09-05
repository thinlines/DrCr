<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2025  Contributors
-->

<template>
    <h1 class="page-heading mb-4">Settings</h1>

    <div class="grid grid-cols-[max-content_1fr] space-y-2 mb-4 items-baseline">
        <label for="sofy_month" class="block text-gray-900 pr-4">Start of financial year</label>
        <div class="flex items-center gap-2">
            <select id="sofy_month" class="bordered-field" v-model.number="sofyMonth">
                <option v-for="m in months" :key="m.value" :value="m.value">{{ m.label }}</option>
            </select>
            <select id="sofy_day" class="bordered-field" v-model.number="sofyDay">
                <option v-for="d in daysInSelectedMonth" :key="d" :value="d">{{ d }}</option>
            </select>
        </div>
        <div></div>
        <p class="text-xs text-gray-500 mt-1">
            Only month and day are used. We set the end of financial year to the day before this date,
            normalised to the next upcoming year.
        </p>
    </div>

    <div class="flex justify-end mt-4 space-x-2">
        <button class="btn-primary" @click="save" :disabled="saving">Save</button>
    </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs';
import { computed, ref, watch } from 'vue';

import { db } from '../db.ts';

// Initial SOFY = (EOFY + 1 day)
const initialSofy = dayjs(db.metadata.eofy_date).add(1, 'day');
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
const sofyMonth = ref(initialSofy.month() + 1);
const sofyDay = ref(Math.min(initialSofy.date(), monthLengths[sofyMonth.value]));
const daysInSelectedMonth = computed(() => Array.from({ length: monthLengths[sofyMonth.value] }, (_, i) => i + 1));
watch(sofyMonth, () => {
  if (sofyDay.value > monthLengths[sofyMonth.value]) {
    sofyDay.value = monthLengths[sofyMonth.value];
  }
});
const saving = ref(false);

async function save() {
    try {
        saving.value = true;
        // Build a safe anchor date to compute EOFY month/day from SOFY (ignore year entirely)
        const mm = String(sofyMonth.value).padStart(2, '0');
        const dd = String(sofyDay.value).padStart(2, '0');
        const sofyAnchor = dayjs(`2001-${mm}-${dd}`); // 2001 is a non-leap year; we avoid Feb 29 in UI
        // Derive EOFY (SOFY - 1 day) month/day
        const eofyMd = sofyAnchor.subtract(1, 'day');
        const today = dayjs();
        let newEofyDayjs = today.set('month', eofyMd.month()).set('date', eofyMd.date());
        if (newEofyDayjs.isBefore(today)) {
            newEofyDayjs = newEofyDayjs.add(1, 'year');
        }
        const newEofy = newEofyDayjs.format('YYYY-MM-DD');

        const session = await db.load();
        const tx = await session.begin();
        await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'eofy_date'`, [newEofy]);
        await tx.commit();

        // Update reactive cache
        db.metadata.eofy_date = newEofy;
    } finally {
        saving.value = false;
    }
}
</script>
