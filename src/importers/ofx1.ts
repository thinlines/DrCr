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
	// Import an OFX1/QFX SGML file by converting it to XML and parsing

	// 1) Strip OFX header (everything before the first <OFX), case-insensitive
	const start = content.search(/<OFX/i);
	if (start < 0) {
		throw new Error('OFX payload not found');
	}
	let rawPayload = content.substring(start);

	// 2) Escape bare ampersands (avoid double-escaping entities)
	// Replace & not followed by an entity pattern with &amp;
	rawPayload = rawPayload.replace(/&(?![a-zA-Z]+;|#[0-9]+;)/g, '&amp;');

	// 3) Convert SGML-style tags <TAG>value into well-formed XML <TAG>value</TAG>
	// This handles values up to the next '<' (i.e., the next tag)
	let xmlPayload = rawPayload.replace(/<([A-Za-z0-9_.]+)>([^<]+)/g, '<$1>$2</$1>');
	// Also close empty leaf tags that appear as <TAG> immediately followed by another tag
	xmlPayload = xmlPayload.replace(/<([A-Za-z0-9_.]+)>(?=\s*<)/g, '<$1></$1>');

	// 4) Parse as XML
	const xmlHeader = '<?xml version="1.0" encoding="UTF-8" standalone="no"?>';
	const tree = new DOMParser().parseFromString(xmlHeader + xmlPayload, 'application/xml');

	// Read transactions
	const statementLines: StatementLine[] = [];

	for (const transaction of tree.querySelectorAll('BANKTRANLIST STMTTRN, banktranlist stmttrn')) {
		// Date may include fractional seconds and/or a timezone in brackets
		let dateRaw = (transaction.querySelector('DTPOSTED') || transaction.querySelector('dtposted'))?.textContent || '';
		if (dateRaw && dateRaw.indexOf('[') >= 0) {
			// Ignore time zone bracket
			dateRaw = dateRaw.substring(0, dateRaw.indexOf('['));
		}
		// Keep only digits; take first 14 as YYYYMMDDHHmmss
		const dateDigits = (dateRaw.match(/\d+/)?.[0] || '').slice(0, 14);
		if (dateDigits.length < 8) { continue; }
		const date = dayjs(dateDigits.padEnd(14, '0'), 'YYYYMMDDHHmmss')
			.hour(0).minute(0).second(0).millisecond(0)
			.format(DT_FORMAT);

		const description =
			(transaction.querySelector('MEMO') || transaction.querySelector('memo'))?.textContent?.trim() ||
			(transaction.querySelector('NAME') || transaction.querySelector('name'))?.textContent?.trim() ||
			'';
		const amountStr = (transaction.querySelector('TRNAMT') || transaction.querySelector('trnamt'))?.textContent;
		if (!amountStr) { continue; }

		const quantity = Math.round(parseFloat(amountStr) * Math.pow(10, db.metadata.dps));
		if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }

		if (description.indexOf('PENDING') >= 0) {
			// FIXME: This needs to be configurable
			continue;
		}

		statementLines.push({
			id: null,
			source_account: sourceAccount,
			dt: date,
			description: description,
			quantity: quantity,
			balance: null,
			commodity: db.metadata.reporting_commodity
		});
	}

	return statementLines;
}
