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

import { parse } from 'csv-parse/browser/esm/sync';
import dayjs from 'dayjs';

import { DT_FORMAT, StatementLine, db } from '../db.ts';

export default function importCsv(sourceAccount: string, content: string): StatementLine[] {
	const records = parse(content, {
		skip_empty_lines: true,
	});
	
	// Validate column layout
	if (records.length === 0) {
		throw new Error('Empty CSV file');
	}
	if (records[0][0] !== 'Date') {
		throw new Error('Unexpected column 1, expected "Date"');
	}
	if (records[0][1] !== 'Description') {
		throw new Error('Unexpected column 1, expected "Description"');
	}
	if (records[0][2] !== 'Amount') {
		throw new Error('Unexpected column 1, expected "Amount"');
	}
	
	const statementLines: StatementLine[] = [];
	
	// Parse records
	for (let i = 1; i < records.length; i++) {
		const record = records[i];
		
		const date = dayjs(record[0], 'YYYY-MM-DD').format(DT_FORMAT);
		const name = record[1];
		const memo = '';
		const description = (name + ' ' + memo).trim();
		const amount = record[2];
		
		const quantity = Math.round(parseFloat(amount) * Math.pow(10, db.metadata.dps));
		if (!Number.isSafeInteger(quantity)) { throw new Error('Quantity not representable by safe integer'); }
		
		statementLines.push({
			id: null,
			source_account: sourceAccount,
			dt: date,
			name: name,
			memo: memo,
			description: description,
			quantity: quantity,
			balance: null,
			commodity: db.metadata.reporting_commodity,
			fitid: null,
		});
	}
	
	return statementLines;
}
