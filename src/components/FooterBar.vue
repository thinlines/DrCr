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
	<footer class="print:hidden">
		<div class="mx-auto max-w-7xl px-6 lg:px-8">
			<div class="border-t border-gray-200 py-4 text-center text-sm text-gray-500 sm:text-left">
				DrCr (build {{ COMMIT_HASH }})
			</div>
		</div>
	</footer>
</template>

<script setup lang="ts">
	import { COMMIT_HASH } from '../version.ts';
</script>
