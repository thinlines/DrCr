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
	<DynamicReportComponent :report="report" />
</template>

<script setup lang="ts">
	import { ref } from 'vue';
	
	import { Computed, DynamicReport, Section, Spacer, Subtotal } from './base.ts';
	import { db } from '../db.ts';
	import DynamicReportComponent from '../components/DynamicReportComponent.vue';
	import { ReportingStage, ReportingWorkflow } from '../reporting.ts';
	
	const report = ref(null as IncomeStatementReport | null);
	
	async function load() {
		const session = await db.load();
		const reportingWorkflow = new ReportingWorkflow();
		await reportingWorkflow.generate(session);
		
		report.value = reportingWorkflow.getReportAtStage(ReportingStage.InterimIncomeStatement, IncomeStatementReport) as IncomeStatementReport;
	}
	load();
</script>
