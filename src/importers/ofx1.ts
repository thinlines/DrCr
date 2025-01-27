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

import dayjs from 'dayjs';

import { DT_FORMAT, StatementLine, db } from '../db.ts';

export default function importOfx1(sourceAccount: string, content: string): StatementLine[] {
	// Import an OFX1 SGML file
	
	// Strip OFX header and parse
	const raw_payload = content.substring(content.indexOf('<OFX')).replaceAll('&', '&amp;');
	const tree = new DOMParser().parseFromString(raw_payload, 'text/html');  // HTML was originally based on SGML so use this parser
	
	// Read transactions
	const statementLines: StatementLine[] = [];
	
	for (const transaction of tree.querySelectorAll('banktranlist stmttrn')) {
		let dateRaw = getNodeText(transaction.querySelector('dtposted'));
		if (dateRaw && dateRaw.indexOf('[') >= 0) {
			// Ignore time zone
			dateRaw = dateRaw?.substring(0, dateRaw.indexOf('['));
		}
		const date = dayjs(dateRaw, 'YYYYMMDDHHmmss.SSS').hour(0).minute(0).second(0).millisecond(0).format(DT_FORMAT);
		
		const description = getNodeText(transaction.querySelector('memo'));
		const amount = getNodeText(transaction.querySelector('trnamt'));
		
		const quantity = Math.round(parseFloat(amount!) * Math.pow(10, db.metadata.dps));
		if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
		
		if (description.indexOf('PENDING') >= 0) {
			// FIXME: This needs to be configurable
			continue;
		}
		
		statementLines.push({
			id: null,
			source_account: sourceAccount,
			dt: date,
			description: description ?? '',
			quantity: quantity,
			balance: null,
			commodity: db.metadata.reporting_commodity
		});
	}
	
	return statementLines;
}

function getNodeText(node: Node | null): string {
	// Get text of the first text node
	// HTML parser does not understand SGML/OFX nesting rules, so siblings will be incorrectly considered as children
	// Therefore we use only the first text node
	
	if (node === null) {
		throw new Error('Node not found');
	}
	
	for (const child of node.childNodes) {
		if (child.nodeType === Node.TEXT_NODE && child.nodeValue !== null && child.nodeValue.trim().length > 0) {
			return child.nodeValue.trim();
		}
		if (child.nodeType === Node.ELEMENT_NODE) {
			break;
		}
	}
	
	throw new Error('No text in node');
}
