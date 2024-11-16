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
	<div class="min-h-full">
		<HeaderBar />
		<div class="py-8">
			<main>
				<div class="mx-auto max-w-7xl px-6 lg:px-8">
					<NoFileView v-if="db.filename === null" />
					<RouterView v-if="db.filename !== null" />
				</div>
			</main>
		</div>
	</div>
</template>

<script setup lang="ts">
	import HeaderBar from './components/HeaderBar.vue';
	import NoFileView from './pages/NoFileView.vue';
	
	import { db } from './db.js';
</script>
