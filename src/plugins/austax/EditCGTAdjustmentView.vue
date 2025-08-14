<!--
	DrCr: Double-entry bookkeeping framework
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
		Edit CGT adjustment
	</h1>
	
	<CGTAdjustmentEditor :adjustment="adjustment" />
</template>

<script setup lang="ts">
	import dayjs from 'dayjs';
	import { ref } from 'vue';
	import { useRoute } from 'vue-router';
	
	import CGTAdjustmentEditor, { EditingCGTAdjustment } from './CGTAdjustmentEditor.vue';
	import { db, serialiseAmount } from '../../db.ts';
	
	const route = useRoute();
	
	const adjustment = ref({
		id: null,
		asset: null!,
		account: null!,
		acquisition_dt: null!,
		dt: null!,
		description: null!,
		cost_adjustment: null!,
	} as EditingCGTAdjustment);
	
	async function load() {
		const session = await db.load();
		
		const rawAdjustments: any[] = await session.select(
			`SELECT id, quantity, commodity, account, acquisition_dt, dt, description, cost_adjustment
			FROM austax_cgt_cost_adjustments
			WHERE id = $1`,
			[route.params.id]
		);
		const rawAdjustment = rawAdjustments[0];
		
		// Format parameters for display
		rawAdjustment.asset = serialiseAmount(rawAdjustment.quantity, rawAdjustment.commodity);
		rawAdjustment.acquisition_dt = dayjs(rawAdjustment.acquisition_dt).format('YYYY-MM-DD');
		rawAdjustment.dt = dayjs(rawAdjustment.dt).format('YYYY-MM-DD');
		rawAdjustment.cost_adjustment = serialiseAmount(rawAdjustment.cost_adjustment, db.metadata.reporting_commodity);
		
		adjustment.value = rawAdjustment as EditingCGTAdjustment;
	}
	
	load();
</script>
