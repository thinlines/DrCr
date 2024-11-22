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

<template>
	<nav class="border-b border-gray-200 bg-white print:hidden" v-if="isMainWindow">
		<div class="mx-auto max-w-7xl px-6 lg:px-8">
			<div class="flex h-12 justify-between ml-[-0.25rem] w-full"><!-- Adjust margin by -0.25rem to align navbar text with body text -->
				<div class="flex w-full">
					<div class="flex flex-shrink-0">
						<RouterLink to="/" class="border-transparent text-gray-900 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm font-medium">
							DrCr
						</RouterLink>
					</div>
					
					<!-- Menu items -->
					<div v-if="db.filename !== null" class="hidden sm:-my-px sm:ml-6 sm:flex sm:gap-4 w-full">
						<RouterLink :to="{ name: 'journal' }" class="border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Journal
						</RouterLink>
						<RouterLink :to="{ name: 'statement-lines' }" class="border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Statement lines
						</RouterLink>
						<RouterLink :to="{ name: 'trial-balance'}" class="border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Trial balance
						</RouterLink>
						<RouterLink :to="{ name: 'balance-sheet'}" class="border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Balance sheet
						</RouterLink>
						<RouterLink :to="{ name: 'income-statement'}" class="border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Income statement
						</RouterLink>
						
						<a href="#" @click="closeFile" class="ml-auto border-transparent text-gray-700 hover:border-emerald-500 hover:text-emerald-700 inline-flex items-center border-b-2 px-1 pt-1 text-sm">
							Close file
						</a>
					</div>
				</div>
			</div>
		</div>
	</nav>
</template>

<script setup lang="ts">
	import { getCurrentWindow } from '@tauri-apps/api/window';
	
	import { useRouter } from 'vue-router';
	
	import { db } from '../db.js';
	
	// Only display header bar in main window
	const isMainWindow = getCurrentWindow().label === 'main';
	
	const router = useRouter();
	
	async function closeFile() {
		await db.init(null);
		router.push({ name: 'index' });
	}
</script>
