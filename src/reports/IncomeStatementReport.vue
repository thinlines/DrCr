<!--
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2025  Lee Yingtong Li (RunasSudo)
	
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

<script lang="ts">
	export class IncomeStatementReport extends DynamicReport {
		constructor() {
			super('Income statement');
		}
		
		async generate(balances: Map<string, number>) {
			const report = this;
			this.entries = [
				new Section(
					'Income',
					[
						...await DynamicReport.entriesForKind(balances, 'drcr.income', true),
						new Subtotal('Total income', 'total_income', true /* visible */, true /* bordered */)
					]
				),
				new Spacer(),
				new Section(
					'Expenses',
					[
						...await DynamicReport.entriesForKind(balances, 'drcr.expense'),
						new Subtotal('Total expenses', 'total_expenses', true /* visible */, true /* bordered */)
					]
				),
				new Spacer(),
				new Computed(
					'Net surplus (deficit)',
					() => (report.byId('total_income') as Subtotal).quantity - (report.byId('total_expenses') as Subtotal).quantity,
					'net_surplus',
					true /* visible */, false /* autoHide */, null /* link */, true /* heading */, true /* bordered */
				)
			];
			
			this.calculate();
		}
	}
</script>

<!-- Report display -->

<template>
	<DynamicReportComponent :report="report">
		<div class="my-2 py-2 flex">
			<div class="grow flex gap-x-2 items-baseline">
				<input type="date" class="bordered-field" v-model="dtStart">
				<span>to</span>
				<input type="date" class="bordered-field" v-model="dt">
			</div>
		</div>
	</DynamicReportComponent>
</template>

<script setup lang="ts">
	import { ref, watch } from 'vue';
	import dayjs from 'dayjs';
	
	import { Computed, DynamicReport, Section, Spacer, Subtotal } from './base.ts';
	import { db } from '../db.ts';
	import { ExtendedDatabase } from '../dbutil.ts';
	import DynamicReportComponent from '../components/DynamicReportComponent.vue';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	
	const report = ref(null as IncomeStatementReport | null);
	
	const dt = ref(null as string | null);
	const dtStart = ref(null as string | null);
	
	async function load() {
		const session = await db.load();
		
		dt.value = db.metadata.eofy_date;
		dtStart.value = dayjs(db.metadata.eofy_date).subtract(1, 'year').add(1, 'day').format('YYYY-MM-DD');
		
		await updateReport(session);
	}
	
	async function updateReport(session: ExtendedDatabase) {
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session, dt.value!, dtStart.value!);
		
		report.value = reportingWorkflow.getReportAtStage(ReportingStage.InterimIncomeStatement, IncomeStatementReport) as IncomeStatementReport;
	}
	
	// Update report when dates etc. changed
	watch([dt, dtStart], async () => {
		const session = await db.load();
		updateReport(session);
	});
	
	load();
</script>
