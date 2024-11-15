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
	<p class="text-gray-900 mb-4">Welcome to DrCr. No file is currently open.</p>
	<ul class="list-disc ml-6">
		<!--<li><a href="#" class="text-gray-900 hover:text-blue-700 hover:underline">New file</a></li>-->
		<li><a href="#" @click="openFile" class="text-gray-900 hover:text-blue-700 hover:underline">Open file</a></li>
	</ul>
</template>

<script setup lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	
	import { db } from './db.js';
	
	async function openFile() {
		const file = await open({
			multiple: false,
			directory: false,
		});
		
		if (file !== null) {
			db.init(file);
		}
	}
</script>
