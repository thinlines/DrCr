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

import { StatementLine, db } from '../db.ts';

export type DuplicateReason =
    | 'existing-fitid'
    | 'file-fitid'
    | 'existing-signature'
    | 'file-signature'
    | 'existing-date-amount'
    | 'file-date-amount';

export interface AnnotatedStatementLine extends StatementLine {
    duplicate: boolean;
    duplicateReason: DuplicateReason | null;
}

interface ExistingStatementLineRow {
    fitid: string | null;
    dt: string;
    description: string;
    name: string | null;
    memo: string | null;
    quantity: number;
}

export async function annotateStatementLineDuplicates(sourceAccount: string, lines: StatementLine[]): Promise<AnnotatedStatementLine[]> {
    if (!sourceAccount) {
        throw new Error('Source account must be selected before importing a statement.');
    }

    const session = await db.load();
    const existingLines = await session.select<ExistingStatementLineRow[]>(
        `SELECT fitid, dt, description, name, memo, quantity
        FROM statement_lines
        WHERE source_account = ?`,
        [sourceAccount]
    );

    const existingFitids = new Set(existingLines.filter((row) => row.fitid !== null).map((row) => row.fitid!));
    const existingSignatures = new Set(existingLines.map(signatureForRow));
    const existingDateAmounts = new Set(existingLines.map(dateAmountKeyForRow));

    const seenFitids = new Set(existingFitids);
    const seenSignatures = new Set(existingSignatures);
    const seenDateAmounts = new Set(existingDateAmounts);

    return lines.map((line) => {
        if (line.fitid) {
            const key = line.fitid.trim();
            if (!key) {
                return enrich(line, false, null);
            }
            if (seenFitids.has(key)) {
                const reason: DuplicateReason = existingFitids.has(key) ? 'existing-fitid' : 'file-fitid';
                return enrich(line, true, reason);
            }
            seenFitids.add(key);
            return enrich(line, false, null);
        }

        const signature = signatureForLine(line);
        if (seenSignatures.has(signature)) {
            const reason: DuplicateReason = existingSignatures.has(signature) ? 'existing-signature' : 'file-signature';
            return enrich(line, true, reason);
        }
        seenSignatures.add(signature);

        const dateAmountKey = dateAmountKeyForLine(line);
        if (seenDateAmounts.has(dateAmountKey)) {
            const reason: DuplicateReason = existingDateAmounts.has(dateAmountKey) ? 'existing-date-amount' : 'file-date-amount';
            return enrich(line, true, reason);
        }
        seenDateAmounts.add(dateAmountKey);
        return enrich(line, false, null);
    });
}

function enrich(line: StatementLine, duplicate: boolean, duplicateReason: DuplicateReason | null): AnnotatedStatementLine {
    return { ...line, duplicate, duplicateReason };
}

function normaliseComponent(value: string | null | undefined): string {
    return (value ?? '').trim().replace(/\s+/g, ' ');
}

function signatureForRow(row: ExistingStatementLineRow): string {
    return [normaliseComponent(row.dt), row.quantity, normaliseComponent(row.description), normaliseComponent(row.name), normaliseComponent(row.memo)].join('|');
}

function signatureForLine(line: StatementLine): string {
    return [normaliseComponent(line.dt), line.quantity, normaliseComponent(line.description), normaliseComponent(line.name), normaliseComponent(line.memo)].join('|');
}

function dateAmountKeyForRow(row: ExistingStatementLineRow): string {
    return dateAmountKey(row.dt, row.quantity);
}

function dateAmountKeyForLine(line: StatementLine): string {
    return dateAmountKey(line.dt, line.quantity);
}

function dateAmountKey(dt: string, quantity: number): string {
    return `${normaliseComponent(dt)}|${quantity}`;
}
