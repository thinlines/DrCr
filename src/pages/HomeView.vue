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
	<div :class="{'grid divide-x divide-gray-200': true, 'grid-cols-2': db.metadata.plugins.indexOf('austax') < 0, 'grid-cols-3': db.metadata.plugins.indexOf('austax') >= 0}">
		<div class="pr-4">
			<h2 class="font-medium text-gray-700 mb-2">Data sources</h2>
			<ul class="list-disc ml-6">
				<li><RouterLink :to="{ name: 'journal' }" class="text-gray-900 hover:text-blue-700 hover:underline">Journal</RouterLink></li>
				<li><RouterLink :to="{ name: 'statement-lines' }" class="text-gray-900 hover:text-blue-700 hover:underline">Statement lines</RouterLink></li>
				<li><RouterLink :to="{ name: 'balance-assertions' }" class="text-gray-900 hover:text-blue-700 hover:underline">Balance assertions</RouterLink></li>
				<li><RouterLink :to="{ name: 'chart-of-accounts' }" class="text-gray-900 hover:text-blue-700 hover:underline">Chart of accounts</RouterLink></li>
				<!-- Plugin reports -->
				<component :is="austax.getDataSourcesLinks()" v-if="db.metadata.plugins.indexOf('austax') >= 0"></component>
			</ul>
		</div>
		<div class="px-4">
			<h2 class="font-medium text-gray-700 mb-2">General reports</h2>
			<ul class="list-disc ml-6">
				<li><RouterLink :to="{ name: 'general-ledger' }" class="text-gray-900 hover:text-blue-700 hover:underline">General ledger</RouterLink></li>
				<li><RouterLink :to="{ name: 'trial-balance' }" class="text-gray-900 hover:text-blue-700 hover:underline">Trial balance</RouterLink></li>
				<li><RouterLink :to="{ name: 'balance-sheet' }" class="text-gray-900 hover:text-blue-700 hover:underline">Balance sheet</RouterLink></li>
				<li><RouterLink :to="{ name: 'income-statement' }" class="text-gray-900 hover:text-blue-700 hover:underline">Income statement</RouterLink></li>
				<!-- Plugin reports -->
				<component :is="austax.getGeneralReportsLinks()" v-if="db.metadata.plugins.indexOf('austax') >= 0"></component>
			</ul>
		</div>
		<div class="pl-4" v-if="db.metadata.plugins.indexOf('austax') >= 0">
			<h2 class="font-medium text-gray-700 mb-2">Advanced reports</h2>
			<ul class="list-disc ml-6">
				<!-- Plugin reports -->
				<component :is="austax.getAdvancedReportsLinks()" v-if="db.metadata.plugins.indexOf('austax') >= 0"></component>
			</ul>
		</div>
	</div>
</template>

<script setup lang="ts">
	import { db } from '../db.ts';
	import austax from '../plugins/austax/plugin.ts';
</script>
