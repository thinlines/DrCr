/*
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
*/

import dayjs from 'dayjs';

import { DT_FORMAT, StatementLine, db } from '../db.ts';

export default function import_ofx2(sourceAccount: string, content: string): StatementLine[] {
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
		const description = transaction.querySelector('NAME')!.textContent;
		const amount = transaction.querySelector('TRNAMT')!.textContent;
		
		if (amount === '0') {
			// Continuation line
			statementLines.at(-1)!.description += '\n' + description;
		} else {
			const quantity = Math.round(parseFloat(amount!) * Math.pow(10, db.metadata.dps));
			if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
			
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
	}
	
	return statementLines;
}
