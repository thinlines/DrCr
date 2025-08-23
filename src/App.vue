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
	<div class="min-h-full">
		<HeaderBar />
		<div class="py-8">
			<main>
				<div class="mx-auto max-w-7xl px-6 lg:px-8">
					<template v-if="error === null">
						<NoFileView v-if="!(db.filename !== null || route.name === 'new-file')" />
						<RouterView v-if="db.filename !== null || route.name === 'new-file'" />
					</template>
					<template v-if="error !== null">
						<CriticalErrorView />
					</template>
				</div>
			</main>
		</div>
		<FooterBar />
	</div>
</template>

<script setup lang="ts">
	import { onErrorCaptured } from 'vue';
	import { useRoute } from 'vue-router';
	
	import FooterBar from './components/FooterBar.vue';
	import HeaderBar from './components/HeaderBar.vue';
	import { db } from './db.js';
	import { error, handleCriticalError } from './error.js';
	import CriticalErrorView from './pages/CriticalErrorView.vue';
	import NoFileView from './pages/NoFileView.vue';
	
	const route = useRoute();
	
	onErrorCaptured((err) => handleCriticalError(err));
</script>
