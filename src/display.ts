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

import { db, validateCommodity } from './db.ts';

export function pp(quantity: number): string {
	// Pretty print the quantity
	if (quantity < 0) {
		return '−' + pp(-quantity);
	}
	
	const factor = Math.pow(10, db.metadata.dps);
	const wholePart = Math.floor(quantity / factor);
	const fracPart = quantity % factor;

	const thousandsSep = db.metadata.place_separator ?? '\u202F';
	const decimalSep = db.metadata.decimal_separator ?? '.';

	return (
		wholePart
			.toString()
			.replace(/\B(?=(\d{3})+(?!\d))/g, thousandsSep) +
		decimalSep +
		fracPart.toString().padStart(db.metadata.dps, '0')
	);
}

export function ppWithCommodity(quantity: number, commodity: string): string {
	// Pretty print the amount including commodity
	validateCommodity(commodity);
	
	const commodityParts = commodity.split(' ');
	
	if (commodityParts[0].length === 1) {
		if (commodityParts.length === 1) {
			return commodityParts[0] + pp(quantity);
		} else {
			return commodityParts[0] + pp(quantity) + ' ' + commodityParts[1];
		}
	} else {
		return pp(quantity) + ' ' + commodity;
	}
}

export function ppBracketed(quantity: number, link?: string): string {
	// Pretty print the quantity with brackets for negative numbers
	let text, space;
	if (quantity >= 0) {
		text = pp(quantity);
		space = '&nbsp;';
	} else {
		text = '(' + pp(-quantity) + ')';
		space = '';
	}
	
	if (link) {
		// Put the space outside of the hyperlink so it is not underlined
		return '<a href="' + encodeURI(link) + '" class="hover:text-blue-700 hover:underline">' + text + '</a>' + space;
	} else {
		return text + space;
	}
}
