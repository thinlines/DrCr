/*
	DrCr: Web-based double-entry bookkeeping framework
	Copyright (C) 2022–2025  Lee Yingtong Li (RunasSudo)
	
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

import { db } from './db.ts';

export interface Amount {
	quantity: number;
	commodity: string;
}

export class Balance {
	// A collection of Amount's
	amounts: Amount[] = [];
	
	add(quantity: number, commodity: string) {
		const existingAmount = this.amounts.find((a) => a.commodity === commodity);
		if (existingAmount) {
			existingAmount.quantity += quantity;
		} else {
			this.amounts.push({ quantity: quantity, commodity: commodity });
		}
	}
	
	clone(): Balance {
		const newBalance = new Balance();
		for (const amount of this.amounts) {
			newBalance.amounts.push({ quantity: amount.quantity, commodity: amount.commodity });
		}
		return newBalance;
	}
	
	clean() {
		this.amounts = this.amounts.filter((a) => a.quantity !== 0);
	}
}

export function asCost(quantity: number, commodity: string): number {
	// Convert the amount to cost price in the reporting commodity
	
	if (commodity === db.metadata.reporting_commodity) {
		return quantity;
	}
	if (commodity.indexOf('{{') >= 0) {
		// Total price
		const price = parseFloat(commodity.substring(commodity.indexOf('{{') + 2, commodity.indexOf('}}', commodity.indexOf('{{'))));
		
		// Multiply by Math.sign(quantity) in case the quantity is negative
		// FIXME: This yields unexpected results when trying to deduct a partial amount from a commodity specified in total price terms
		return Math.round(Math.sign(quantity) * price * Math.pow(10, db.metadata.dps));
	}
	if (commodity.indexOf('{') >= 0) {
		// Unit price
		const price = parseFloat(commodity.substring(commodity.indexOf('{') + 1, commodity.indexOf('}', commodity.indexOf('{'))));
		return Math.round(quantity * price);
	}
	throw new Error('No cost base specified: ' + quantity + ' ' + commodity);
}
