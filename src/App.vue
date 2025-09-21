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
    <div class="h-full flex flex-col">
        <HeaderBar />
        <div class="flex-1 min-h-0">
            <main class="h-full overflow-hidden">
                <div class="mx-auto max-w-7xl px-6 lg:px-8 py-8 h-full">
                    <template v-if="error === null">
                        <RouterView />
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
	
	import FooterBar from './components/FooterBar.vue';
	import HeaderBar from './components/HeaderBar.vue';
    import { error, handleCriticalError } from './error.js';
    import CriticalErrorView from './pages/CriticalErrorView.vue';
    
    onErrorCaptured((err) => handleCriticalError(err));
</script>
