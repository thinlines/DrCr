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
	<nav class="border-b border-gray-200 bg-white print:hidden" v-if="isMainWindow">
		<div class="mx-auto max-w-7xl px-6 lg:px-8">
			<div class="flex h-12 items-center justify-between mx-[-0.25rem] w-full"><!-- Adjust margin by -0.25rem to align navbar text with body text -->
				<div class="flex items-center gap-2">
					<!-- Back button-->
					<div class="hidden min-[1408px]:block" v-if="route.name !== 'index'">
						<button type="button" @click="router.back" class="p-1 text-gray-400 hover:text-gray-500">
							<ArrowLeftCircleIcon class="w-6 h-6" />
						</button>
					</div>
					<span class="text-sm font-medium text-gray-900">DrCr</span>
				</div>
				<div v-if="db.filename !== null" class="flex items-center gap-3">
					<button type="button" @click="openSettings" class="p-1 text-gray-400 hover:text-gray-500" aria-label="Open settings">
						<Cog6ToothIcon class="w-6 h-6" />
					</button>
				</div>
			</div>
		</div>
	</nav>
</template>

<script setup lang="ts">
	import { ArrowLeftCircleIcon, Cog6ToothIcon } from '@heroicons/vue/24/outline';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { useRoute, useRouter } from 'vue-router';
	
	import { db } from '../db.js';
	
	// Only display header bar in main window
	const isMainWindow = getCurrentWindow().label === 'main';
	
	const route = useRoute();
	const router = useRouter();

	function openSettings() {
		if (route.name !== 'settings') {
			router.push({ name: 'settings' });
		}
	}
</script>
