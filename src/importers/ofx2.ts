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

export default function importOfx2(sourceAccount: string, content: string): StatementLine[] {
	// Import an OFX2 XML file
	
	// Convert OFX header to XML and parse
	const xml_header = '<?xml version="1.0" encoding="UTF-8" standalone="no"?>';
	const raw_payload = content.substring(content.indexOf('?>') + 2).replaceAll('&', '&amp;');
	const tree = new DOMParser().parseFromString(xml_header + raw_payload, 'application/xml');
	
	// Read transactions
	const statementLines: StatementLine[] = [];
	
	for (const transaction of tree.querySelectorAll('BANKMSGSRSV1 STMTTRNRS STMTRS BANKTRANLIST STMTTRN')) {
		let dateRaw = transaction.querySelector('DTPOSTED')!.textContent;
		if (dateRaw && dateRaw.indexOf('[') >= 0) {
			// Ignore time zone
			dateRaw = dateRaw?.substring(0, dateRaw.indexOf('['));
		}
		const date = dayjs(dateRaw, 'YYYYMMDDHHmmss').hour(0).minute(0).second(0).millisecond(0).format(DT_FORMAT);
		const name = transaction.querySelector('NAME')?.textContent ?? '';
		const memoTag = transaction.querySelector('MEMO')?.textContent ?? '';
		const amount = transaction.querySelector('TRNAMT')!.textContent;
		const fitid = transaction.querySelector('FITID')?.textContent ?? null;
		
		if (amount === '0') {
			// Continuation line: append extra details to memo
			const last = statementLines.at(-1)!;
			last.memo = (last.memo ? last.memo + '\n' : '') + (memoTag || name);
			last.description = (last.name + ' ' + last.memo).trim();
		} else {
			const quantity = Math.round(parseFloat(amount!) * Math.pow(10, db.metadata.dps));
			if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
			
			statementLines.push({
				id: null,
				source_account: sourceAccount,
				dt: date,
				name: name ?? '',
				memo: memoTag ?? '',
				description: ((name ?? '') + ' ' + (memoTag ?? '')).trim(),
				quantity: quantity,
				balance: null,
				commodity: db.metadata.reporting_commodity,
				fitid: fitid
			});
		}
	}
	
	return statementLines;
}
