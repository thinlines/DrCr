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
	<div class="relative print:hidden">
		<button class="text-gray-400 align-middle hover:text-gray-500" @click="isMenuOpen = !isMenuOpen"><EllipsisHorizontalCircleIcon class="size-6" /></button>
		<ul class="absolute top-8 right-0 bg-white w-[11rem] shadow-lg ring-1 ring-black/5 focus:outline-hidden" :class="isMenuOpen ? 'block' : 'hidden'">
			<li class="group cursor-pointer select-none py-1 px-3 text-gray-900 hover:text-white hover:bg-emerald-600" @click="menuPrint">
				<PrinterIcon class="inline size-5 text-gray-500 group-hover:text-white" />
				Print/Save as PDF
			</li>
		</ul>
	</div>
</template>

<script setup type="ts">
	import { EllipsisHorizontalCircleIcon, PrinterIcon } from '@heroicons/vue/24/outline';
	import { ref } from 'vue';
	
	const isMenuOpen = ref(false);
	
	function menuPrint() {
		window.print();
		isMenuOpen.value = false;
	}
</script>
