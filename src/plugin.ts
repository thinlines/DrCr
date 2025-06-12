/*
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
*/

import { Component, ref } from 'vue';
import { Router, RouteRecordRaw } from 'vue-router';

import { db } from './db.ts';
import austax from './plugins/austax/plugin.ts';

export const loadedPlugins = ref([] as string[]);

export interface Plugin {
	getAccountKinds: () => Promise<[string, string][]>,
	getAdvancedReportsLinks: () => Component,
	getDataSourcesLinks: () => Component,
	getGeneralReportsLinks: () => Component,
	getRoutes: () => RouteRecordRaw[],
}

export function initPlugins(router: Router) {
	// Add plugin routes to router
	if (db.filename !== null) {
		if (db.metadata.plugins.indexOf('austax') >= 0) {
			for (const route of austax.getRoutes()) {
				router.addRoute(route);
			}
		}
		
		loadedPlugins.value = db.metadata.plugins;
	}
}
