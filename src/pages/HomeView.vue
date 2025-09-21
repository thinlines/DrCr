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
	<div class="flex flex-col h-full min-h-0">
		<NoFileView v-if="!hasOpenFile" />
		<div v-else class="flex flex-col min-h-0 flex-1">
			<div class="flex items-center border-b border-gray-200">
				<div class="flex items-center gap-4">
					<button
						type="button"
						class="px-3 py-2 text-sm font-medium border-b-2"
						:class="activeTab === 'statements' ? 'border-blue-600 text-blue-600' : 'border-transparent text-gray-600 hover:text-gray-800'"
						@click="setActiveTab('statements')"
					>
						Statements
					</button>
					<button
						type="button"
						class="px-3 py-2 text-sm font-medium border-b-2"
						:class="activeTab === 'transactions' ? 'border-blue-600 text-blue-600' : 'border-transparent text-gray-600 hover:text-gray-800'"
						@click="setActiveTab('transactions')"
					>
						Transactions
					</button>
				</div>
				<button
					type="button"
					class="px-3 py-2 text-sm font-medium border-b-2 ml-auto"
					:class="activeTab === 'setup' ? 'border-blue-600 text-blue-600' : 'border-transparent text-gray-600 hover:text-gray-800'"
					@click="setActiveTab('setup')"
				>
					Setup
				</button>
				</div>

			<div class="mt-6 flex flex-col flex-1 min-h-0">
				<div class="flex flex-wrap gap-2" v-if="activePills.length > 0">
					<button
						type="button"
						v-for="pill in activePills"
						:key="pill.id"
						class="px-4 py-1.5 text-sm font-medium rounded-full border"
						:class="pill.id === activePillId ? 'bg-blue-600 text-white border-blue-600' : 'bg-white text-gray-700 border-gray-300 hover:border-gray-400 hover:text-gray-900'"
						@click="selectPill(pill.id)"
					>
						{{ pill.label }}
					</button>
				</div>

				<div class="mt-6 flex-1 min-h-0 overflow-hidden">
					<KeepAlive>
						<component v-if="activeComponent" :is="activeComponent" :key="activePillKey" />
					</KeepAlive>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
	import type { Component } from 'vue';
	import { computed, reactive, ref, watch, markRaw } from 'vue';
	import { useRoute, useRouter } from 'vue-router';

	import NoFileView from './NoFileView.vue';
	import JournalView from './JournalView.vue';
	import StatementLinesView from './StatementLinesView.vue';
	import BalanceSheetReport from '../reports/BalanceSheetReport.vue';
	import IncomeStatementReport from '../reports/IncomeStatementReport.vue';
	import TrialBalanceReport from '../reports/TrialBalanceReport.vue';
	import ChartOfAccountsView from './ChartOfAccountsView.vue';
	import BalanceAssertionsView from './BalanceAssertionsView.vue';
	import { db } from '../db.ts';

	type TabId = 'statements' | 'transactions' | 'setup';

	interface PillConfig {
		id: string;
		label: string;
		component: Component;
	}

	interface TabConfig {
		id: TabId;
		pills: PillConfig[];
	}

	const statementPills: PillConfig[] = [
		{ id: 'trial-balance', label: 'Trial balance', component: markRaw(TrialBalanceReport) },
		{ id: 'income-statement', label: 'Income statement', component: markRaw(IncomeStatementReport) },
		{ id: 'balance-sheet', label: 'Balance sheet', component: markRaw(BalanceSheetReport) },
	];

	const transactionPills: PillConfig[] = [
		{ id: 'general-ledger', label: 'General ledger', component: markRaw(JournalView) },
		{ id: 'imported-transactions', label: 'Imported transactions', component: markRaw(StatementLinesView) },
	];

	const setupPills: PillConfig[] = [
		{ id: 'chart-of-accounts', label: 'Chart of accounts', component: markRaw(ChartOfAccountsView) },
		{ id: 'balance-assertions', label: 'Balance assertions', component: markRaw(BalanceAssertionsView) },
	];

	const tabs: Record<TabId, TabConfig> = {
		statements: { id: 'statements', pills: statementPills },
		transactions: { id: 'transactions', pills: transactionPills },
		setup: { id: 'setup', pills: setupPills },
	};

	const activeTab = ref<TabId>('statements');
	const selectedPillByTab = reactive<Record<TabId, string>>({
		statements: statementPills[0].id,
		transactions: transactionPills[0].id,
		setup: setupPills[0].id,
	});

	const hasOpenFile = computed(() => db.filename !== null);
	const activePills = computed(() => tabs[activeTab.value].pills);
	const activePillId = computed(() => selectedPillByTab[activeTab.value]);
	const activeComponent = computed<Component | null>(() => {
		if (!hasOpenFile.value) {
			return null;
		}
		const tab = tabs[activeTab.value];
		const pills = tab.pills;
		const pill = pills.find((candidate) => candidate.id === activePillId.value) ?? pills[0];
		return pill?.component ?? null;
	});

	const activePillKey = computed(() => `${activeTab.value}:${activePillId.value}`);

	const route = useRoute();
	const router = useRouter();

	function isTabId(value: unknown): value is TabId {
		return value === 'statements' || value === 'transactions' || value === 'setup';
	}

	function getPillTab(pillId: string): TabId | null {
		for (const tabId of Object.keys(tabs) as TabId[]) {
			const tab = tabs[tabId];
			if (tab.pills.some((pill) => pill.id === pillId)) {
				return tabId;
			}
		}
		return null;
	}

	function applyRouteState() {
		const rawTab = Array.isArray(route.query.tab) ? route.query.tab[0] : route.query.tab;
		const rawPill = Array.isArray(route.query.pill) ? route.query.pill[0] : route.query.pill;
		let targetTab: TabId | null = null;
		if (typeof rawPill === 'string') {
			const pillTab = getPillTab(rawPill);
			if (pillTab !== null) {
				selectedPillByTab[pillTab] = rawPill;
				targetTab = pillTab;
			}
		}
		if (typeof rawTab === 'string' && isTabId(rawTab)) {
			targetTab = rawTab;
		}
		if (targetTab !== null) {
			activeTab.value = targetTab;
		}
	}

	function updateRouteQuery() {
		if (route.name !== 'index') {
			return;
		}
		const currentTab = activeTab.value;
		const currentPill = selectedPillByTab[currentTab];
		const rawTab = Array.isArray(route.query.tab) ? route.query.tab[0] : route.query.tab;
		const rawPill = Array.isArray(route.query.pill) ? route.query.pill[0] : route.query.pill;
		if (rawTab === currentTab && rawPill === currentPill) {
			return;
		}
		router.replace({ name: 'index', query: { tab: currentTab, pill: currentPill } });
	}

	applyRouteState();

	watch(() => [route.query.tab, route.query.pill], applyRouteState);
	watch(() => [activeTab.value, selectedPillByTab.statements, selectedPillByTab.transactions, selectedPillByTab.setup], updateRouteQuery);

	function setActiveTab(tab: TabId) {
		activeTab.value = tab;
	}

	function selectPill(pillId: string) {
		selectedPillByTab[activeTab.value] = pillId;
	}
</script>
