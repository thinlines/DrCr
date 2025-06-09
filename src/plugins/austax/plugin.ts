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

import { getAccountKinds } from './account_kinds.ts';
import DataSourcesLinks from './DataSourcesLinks.vue';
import GeneralReportsLinks from './GeneralReportsLinks.vue';
import AdvancedReportsLinks from './AdvancedReportsLinks.vue';
import { Plugin } from '../../plugin.ts';

export default {
	'getAccountKinds': getAccountKinds,
	
	getDataSourcesLinks: () => DataSourcesLinks,
	getGeneralReportsLinks: () => GeneralReportsLinks,
	getAdvancedReportsLinks: () => AdvancedReportsLinks,
	
	getRoutes: () => [
		{ path: '/austax/cgt-adjustments', name: 'cgt-adjustments', component: () => import('./CGTAdjustmentsView.vue') },
		{ path: '/austax/cgt-adjustments/edit/:id', name: 'cgt-adjustments-edit', component: () => import('./EditCGTAdjustmentView.vue') },
		{ path: '/austax/cgt-adjustments/new', name: 'cgt-adjustments-new', component: () => import('./NewCGTAdjustmentView.vue') },
		{ path: '/austax/cgt-adjustments/multinew', name: 'cgt-adjustments-multinew', component: () => import('./MultiNewCGTAdjustmentView.vue') },
		{ path: '/austax/cgt-assets', name: 'cgt-assets', component: () => import('./CGTAssetsView.vue') },
		{ path: '/austax/tax-summary', name: 'tax-summary', component: () => import('./TaxSummaryReport.vue') },
	],
} as Plugin;
