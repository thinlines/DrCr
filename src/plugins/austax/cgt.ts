/*
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
*/

import { asCost } from '../../amounts.ts';
import { db, joinedToTransactions, JoinedTransactionPosting } from '../../db.ts';
import { ExtendedDatabase } from '../../dbutil.ts';
import { ppWithCommodity } from '../../display.ts';

export interface CGTAdjustment {
	id: number | null,
	quantity: number,
	commodity: string,
	account: string,
	acquisition_dt: string,
	dt: string,
	description: string,
	cost_adjustment: number,
}

export class CGTAsset {
	cost_adjustments: CGTAdjustment[];
	disposal_dt: string | null;
	disposal_value: number | null;
	
	constructor(
		public quantity: number,
		public commodity: string,
		public account: string,
		public acquisition_dt: string
	) {
		this.cost_adjustments = [];
		this.disposal_dt = null;
		this.disposal_value = null;
	}
}

// Load CGT assets and cost adjustments from database
export async function getCGTAssets(session: ExtendedDatabase) {
	// Find all CGT asset accounts
	const cgtAccounts = (await session.select(
		`SELECT account FROM account_configurations
		WHERE kind = 'austax.cgtasset'`
	) as {account: string}[]).map((a) => a.account);
	
	// Find all asset accounts (used to calculate disposal values)
	const assetAccounts = (await session.select(
		`SELECT account FROM account_configurations
		WHERE kind = 'drcr.asset'`
	) as {account: string}[]).map((a) => a.account);
	
	// Get all transactions involving CGT asset accounts
	const cgtJoinedTransactions = await session.select(
		`SELECT joined_transactions.*
		FROM postings
		JOIN joined_transactions ON postings.transaction_id = joined_transactions.transaction_id
		JOIN account_configurations ON postings.account = account_configurations.account
		WHERE account_configurations.kind = 'austax.cgtasset'`
	) as JoinedTransactionPosting[];
	const cgtTransactions = joinedToTransactions(cgtJoinedTransactions);
	
	// Process postings to determine final balances
	const assets: CGTAsset[] = [];
	
	for (const transaction of cgtTransactions) {
		for (const posting of transaction.postings) {
			if (cgtAccounts.indexOf(posting.account) < 0) {
				continue;
			}
			if (posting.commodity === db.metadata.reporting_commodity) {
				continue;
			}
			
			// This posting is to a CGT asset account
			
			if (posting.quantity >= 0) {
				// Debit CGT asset - create new CGTAsset
				assets.push(new CGTAsset(posting.quantity, posting.commodity, posting.account, transaction.dt))
			} else {
				// Credit CGT asset
				// Currently only a full disposal of a CGT asset is implemented
				
				// Find matching CGT asset
				const asset = assets.find((a) => a.commodity === posting.commodity && a.account === posting.account);
				
				if (!asset) {
					throw new Error('Attempted credit of ' + ppWithCommodity(posting.quantity, posting.commodity) + ' without preceding debit balance');
				}
				if (asset.quantity + posting.quantity < 0) {
					throw new Error('Attempted credit of ' + ppWithCommodity(posting.quantity, posting.commodity) + ' which exceeds debit balance of ' + ppWithCommodity(asset.quantity, asset.commodity));
				}
				if (asset.quantity + posting.quantity != 0) {
					throw new Error('Partial disposal of CGT asset not implemented');
				}
				
				asset.disposal_dt = transaction.dt;
				
				// Calculate disposal value for searching for matching asset postings
				asset.disposal_value = 0;
				for (const otherPosting of transaction.postings) {
					if (otherPosting !== posting && assetAccounts.indexOf(otherPosting.account) >= 0) {
						asset.disposal_value += asCost(otherPosting.quantity, otherPosting.commodity);
					}
				}
			}
		}
	}
	
	// Get all CGT cost adjustments
	const cgtAdjustments = await session.select(
		`SELECT id, quantity, commodity, account, acquisition_dt, dt, description, cost_adjustment
		FROM austax_cgt_cost_adjustments
		ORDER BY dt DESC, account, substr(commodity, 1, instr(commodity, ' {')), acquisition_dt, id DESC`
	) as CGTAdjustment[];
	
	// Process CGT adjustments
	for (const cgtAdjustment of cgtAdjustments) {
		// Get corresponding asset
		const asset = assets.find((a) => a.quantity === cgtAdjustment.quantity && a.commodity === cgtAdjustment.commodity && a.account === cgtAdjustment.account && a.acquisition_dt === cgtAdjustment.acquisition_dt);
		
		if (!asset) {
			throw new Error('No matching CGT asset for cost adjustment ' + ppWithCommodity(cgtAdjustment.quantity, cgtAdjustment.commodity));
		}
		
		asset.cost_adjustments.push(cgtAdjustment);
	}
	
	// Sort CGT assets
	assets.sort((a, b) => b.acquisition_dt.localeCompare(a.acquisition_dt));
	assets.sort((a, b) => cgtAssetCommodityName(a.commodity).localeCompare(cgtAssetCommodityName(b.commodity)));
	assets.sort((a, b) => a.account.localeCompare(b.account));
	
	return assets;
}

export function cgtAssetCommodityName(commodity: string): string {
	return commodity.substring(0, commodity.indexOf(' {'));
}
