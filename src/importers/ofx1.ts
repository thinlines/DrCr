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

function getNodeText(node: Element | null): string {
	if (!node) {
		throw new Error('Missing required element');
	}
	return (node.textContent ?? '').trim();
}

export default function importOfx1(sourceAccount: string, content: string): StatementLine[] {
	// Import an OFX1/QFX SGML file by converting it to XML and parsing

	// 1) Strip OFX header (everything before the first <OFX), case-insensitive
	const start = content.search(/<OFX/i);
	if (start < 0) {
		throw new Error('OFX payload not found');
	}
	const rawPayload = content.substring(start);

	// 2) Convert OFX SGML to well-formed XML with a small streaming converter
	const xmlPayload = ofxSgmlToXml(rawPayload);

	// 3) Parse as XML
	const tree = new DOMParser().parseFromString(xmlPayload, 'application/xml');

	// Read transactions
	const statementLines: StatementLine[] = [];

	for (const transaction of tree.querySelectorAll('BANKTRANLIST STMTTRN, banktranlist stmttrn')) {
		// Date may include fractional seconds and/or a timezone in brackets
		let dateRaw = (transaction.querySelector('DTPOSTED') || transaction.querySelector('dtposted'))?.textContent || '';
		if (dateRaw && dateRaw.indexOf('[') >= 0) {
			// Ignore time zone bracket
			dateRaw = dateRaw.substring(0, dateRaw.indexOf('['));
		}
		const date = dayjs(dateRaw, 'YYYYMMDDHHmmss.SSS').hour(0).minute(0).second(0).millisecond(0).format(DT_FORMAT);
		
		// Capture NAME (payee) and MEMO (details)
		let name = '';
		let memo = '';
		try { name = getNodeText(transaction.querySelector('name') ?? transaction.querySelector('NAME')); } catch (e) { /* optional */ }
		try { memo = getNodeText(transaction.querySelector('memo') ?? transaction.querySelector('MEMO')); } catch (e) { /* optional */ }
		const description = (name + ' ' + memo).trim();
		const amount = getNodeText(transaction.querySelector('trnamt') ?? transaction.querySelector('TRNAMT'));
		const fitidNode = transaction.querySelector('fitid') ?? transaction.querySelector('FITID');
		const fitid = fitidNode ? getNodeText(fitidNode) : null;
		
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
				name: name,
				memo: memo,
				description: description ?? '',
				quantity: quantity,
				balance: null,
				commodity: db.metadata.reporting_commodity,
				fitid: fitid
			});
		}

	return statementLines;
}

function ofxSgmlToXml(sgml: string): string {
	// Convert OFX 1.x SGML to well-formed XML by walking tags
	// - Treat <TAG>text as leaf => <TAG>text</TAG>
	// - Treat <TAG> followed by another tag/newline as container => <TAG> ... </TAG>
	// - Honor explicit closing tags </TAG>
	// - Escape bare ampersands in text
	const out: string[] = [];
	out.push('<?xml version="1.0" encoding="UTF-8" standalone="no"?>');

	const stack: string[] = [];
	let i = 0;
	const n = sgml.length;
	while (i < n) {
		const lt = sgml.indexOf('<', i);
		if (lt < 0) { break; }
		const gt = sgml.indexOf('>', lt + 1);
		if (gt < 0) { break; }
		const rawTag = sgml.slice(lt + 1, gt).trim();
		i = gt + 1;

		if (rawTag.startsWith('!')) {
			// Skip comments/decls if any (rare in OFX1)
			continue;
		}

		if (rawTag.startsWith('/')) {
			const closeName = rawTag.slice(1).trim();
			// Close up to the matching tag (case-insensitive)
			let found = false;
			for (let j = stack.length - 1; j >= 0; j--) {
				if (stack[j].toUpperCase() === closeName.toUpperCase()) {
					while (stack.length - 1 >= j) {
						const name = stack.pop()!;
						out.push(`</${name}>`);
					}
					found = true;
					break;
				}
			}
			if (!found) {
				// If unmatched, emit a closing anyway to keep balance
				out.push(`</${closeName}>`);
			}
			continue;
		}

		const openName = rawTag.split(/\s+/)[0];
		// Look ahead to next '<' for inline text
		let nextLt = sgml.indexOf('<', i);
		if (nextLt < 0) nextLt = n;
		const between = sgml.slice(i, nextLt);
		if (between.trim().length > 0) {
			// Leaf with inline text
			out.push(`<${openName}>${escapeText(between)}</${openName}>`);
			i = nextLt;
		} else {
			// Container with children
			out.push(`<${openName}>`);
			stack.push(openName);
			i = nextLt; // move to next tag (whitespace/newlines skipped)
		}
	}

	// Close any remaining open tags
	while (stack.length > 0) {
		const name = stack.pop()!;
		out.push(`</${name}>`);
	}

	return out.join('');
}

function escapeText(text: string): string {
	// Escape bare ampersands; '<' shouldn't occur in text by construction
	return text.replace(/&(?![a-zA-Z]+;|#[0-9]+;)/g, '&amp;');
}
