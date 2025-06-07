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
	<div class="rounded-md bg-red-50 mt-4 p-4 col-span-2">
		<div class="flex">
			<div class="flex-shrink-0">
				<XCircleIcon class="h-5 w-5 text-red-400" />
			</div>
			<div class="ml-3 flex-1">
				<p class="text-sm text-red-700">{{ error.message }}</p>
				<p class="text-sm text-red-700 mt-1" v-if="error.error">{{ error.error }}</p>
			</div>
		</div>
	</div>
</template>

<script setup type="ts">
	import { XCircleIcon } from '@heroicons/vue/24/solid';
	
	import { error } from '../error.ts';
	
	console.error(error.value);
</script>
