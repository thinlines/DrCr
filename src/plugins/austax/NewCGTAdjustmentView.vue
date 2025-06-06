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
		New CGT adjustment
	</h1>
	
	<CGTAdjustmentEditor :adjustment="adjustment" />
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ref } from 'vue';
	
	import CGTAdjustmentEditor, { EditingCGTAdjustment } from './CGTAdjustmentEditor.vue';
	
	const adjustment = ref({
		id: null,
		asset: '',
		account: '',
		acquisition_dt: dayjs().format('YYYY-MM-DD'),
		dt: dayjs().format('YYYY-MM-DD'),
		description: '',
		sign: 'dr',
		cost_adjustment_abs: '',
	} as EditingCGTAdjustment);
</script>
