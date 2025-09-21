<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2025  Contributors
-->

<template>
    <div class="fixed inset-0 z-40 flex items-center justify-center bg-slate-900/50 px-4 py-10">
        <div class="relative w-full max-w-3xl overflow-hidden rounded-xl bg-white shadow-2xl">
            <button
                type="button"
                class="absolute right-4 top-4 rounded-md p-1 text-gray-400 transition hover:bg-gray-100 hover:text-gray-600"
                aria-label="Close settings"
                @click="closeSettings"
            >
                <XMarkIcon class="h-5 w-5" />
            </button>

            <div class="max-h-[calc(100vh-8rem)] overflow-y-auto p-6 sm:p-8">
                <h1 class="page-heading mb-6">Settings</h1>

                <div class="space-y-8">
                    <section>
                        <h2 class="text-sm font-semibold text-gray-900 uppercase tracking-wide mb-3">Financial year</h2>
                        <div class="grid grid-cols-[max-content_1fr] items-baseline gap-y-2">
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
                            <p class="text-xs text-gray-500">
                                Only month and day are used. Stored as the next upcoming occurrence.
                            </p>
                        </div>
                    </section>

                    <section>
                        <h2 class="text-sm font-semibold text-gray-900 uppercase tracking-wide mb-3">Number &amp; date formatting</h2>
                        <div class="grid grid-cols-[max-content_1fr] items-baseline gap-y-4">
                            <label for="thousands-sep" class="block text-gray-900 pr-4">Thousands separator</label>
                            <div>
                                <select id="thousands-sep" class="bordered-field" v-model="placeSeparator">
                                    <option :value="'\u202F'">Space (thin, non-breaking)</option>
                                    <option :value="','">Comma</option>
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

                            <label for="date-style" class="block text-gray-900 pr-4">Date style</label>
                            <div>
                                <select id="date-style" class="bordered-field" v-model="dateStyle">
                                    <option v-for="fmt in dateFormats" :key="fmt" :value="fmt">{{ formatSample(fmt) }}</option>
                                </select>
                                <p class="text-xs text-gray-500 mt-1">Affects dates shown on financial statements.</p>
                            </div>
                        </div>
                    </section>
                </div>

                <div class="mt-10 flex flex-col gap-3 sm:flex-row sm:items-center">
                    <button class="btn-secondary text-red-600 ring-red-500" @click="closeFile">Close file</button>
                    <div class="flex flex-col gap-3 sm:ml-auto sm:flex-row">
                        <button class="btn-secondary" @click="closeSettings">Cancel</button>
                        <button class="btn-primary" @click="save" :disabled="saving">Save</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { XMarkIcon } from '@heroicons/vue/24/outline';
import dayjs from 'dayjs';
import advancedFormat from 'dayjs/plugin/advancedFormat';

import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';

import { db } from '../db.ts';

dayjs.extend(advancedFormat);

const router = useRouter();

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
// Date formatting settings
const dateStyle = ref<string>(db.metadata.date_style ?? 'YYYY-MM-DD');
const dateFormats = [
    'YYYY-MM-DD',
    'D MMM YYYY',
    'D MMMM YYYY',
    "MMM D, YYYY",
    'MMM Do, YYYY',
    "MMMM Do, YYYY",
    'DD/MM/YYYY',
    'MM/DD/YYYY',
] as const;

function formatSample(fmt: string): string {
    return dayjs().format(fmt);
}

async function closeSettings() {
    if (window.history.length > 1) {
        router.back();
        return;
    }
    await router.push({ name: 'index' });
}

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

        // Date style (upsert)
        res = await tx.execute(`UPDATE metadata SET value = ? WHERE key = 'date_style'`, [dateStyle.value]);
        if (res.rowsAffected === 0) {
            await tx.execute(`INSERT INTO metadata (key, value) VALUES ('date_style', ?)`, [dateStyle.value]);
        }

        await tx.commit();

        // Update reactive cache
        db.metadata.eofy_date = newEofy;
        db.metadata.place_separator = placeSeparator.value;
        db.metadata.decimal_separator = decimalSeparator.value;
        db.metadata.date_style = dateStyle.value;

        await closeSettings();
    } finally {
        saving.value = false;
    }
}

async function closeFile() {
	await db.init(null);
	await router.push({ name: 'index' });
}
</script>
