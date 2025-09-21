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

export type DuplicateMatch = ExistingDuplicateMatch | FileDuplicateMatch;

export interface AnnotatedStatementLine extends StatementLine {
    duplicate: boolean;
    duplicateReason: DuplicateReason | null;
    duplicateMatch: DuplicateMatch | null;
}

interface ExistingStatementLineRow {
    id: number;
    fitid: string | null;
    dt: string;
    description: string;
    name: string | null;
    memo: string | null;
    quantity: number;
    commodity: string;
    dedup_ignore: number;
}

interface ExistingDuplicateMatch {
    kind: 'existing';
    statementLine: ExistingStatementLineRow;
}

interface FileDuplicateMatch {
    kind: 'file';
    previousLine: StatementLine;
}

export async function annotateStatementLineDuplicates(sourceAccount: string, lines: StatementLine[]): Promise<AnnotatedStatementLine[]> {
    if (!sourceAccount) {
        throw new Error('Source account must be selected before importing a statement.');
    }

    const session = await db.load();
    const existingLines = await session.select<ExistingStatementLineRow[]>(
        `SELECT id, fitid, dt, description, name, memo, quantity, commodity, dedup_ignore
        FROM statement_lines
        WHERE source_account = ?`,
        [sourceAccount]
    );

    const existingFitidMap = new Map<string, ExistingStatementLineRow>();
    const existingSignatureMap = new Map<string, ExistingStatementLineRow>();
    const existingDateAmountMap = new Map<string, ExistingStatementLineRow>();

    for (const row of existingLines) {
        if (row.dedup_ignore) {
            continue;
        }
        if (row.fitid) {
            existingFitidMap.set(row.fitid, row);
        }
        existingSignatureMap.set(signatureForRow(row), row);
        existingDateAmountMap.set(dateAmountKeyForRow(row), row);
    }

    const seenFitids = new Set(existingFitidMap.keys());
    const seenSignatures = new Set(existingSignatureMap.keys());
    const seenDateAmounts = new Set(existingDateAmountMap.keys());

    const firstSeenFitids = new Map<string, StatementLine>();
    const firstSeenSignatures = new Map<string, StatementLine>();
    const firstSeenDateAmounts = new Map<string, StatementLine>();

    return lines.map((line) => {
        let duplicate = false;
        let reason: DuplicateReason | null = null;
        let match: DuplicateMatch | null = null;

        const fitid = line.fitid?.trim() ?? '';
        if (fitid) {
            if (seenFitids.has(fitid)) {
                duplicate = true;
                reason = existingFitidMap.has(fitid) ? 'existing-fitid' : 'file-fitid';
                match = existingFitidMap.has(fitid)
                    ? existingDuplicate(existingFitidMap.get(fitid)!)
                    : firstSeenFitids.has(fitid)
                        ? fileDuplicate(firstSeenFitids.get(fitid)!)
                        : null;
            } else {
                seenFitids.add(fitid);
                firstSeenFitids.set(fitid, line);
            }
        }

        if (!duplicate) {
            const signature = signatureForLine(line);
            if (seenSignatures.has(signature)) {
                duplicate = true;
                reason = existingSignatureMap.has(signature) ? 'existing-signature' : 'file-signature';
                match = existingSignatureMap.has(signature)
                    ? existingDuplicate(existingSignatureMap.get(signature)!)
                    : firstSeenSignatures.has(signature)
                        ? fileDuplicate(firstSeenSignatures.get(signature)!)
                        : null;
            } else {
                seenSignatures.add(signature);
                firstSeenSignatures.set(signature, line);
            }
        }

        if (!duplicate) {
            const dateAmountKey = dateAmountKeyForLine(line);
            if (seenDateAmounts.has(dateAmountKey)) {
                duplicate = true;
                reason = existingDateAmountMap.has(dateAmountKey) ? 'existing-date-amount' : 'file-date-amount';
                match = existingDateAmountMap.has(dateAmountKey)
                    ? existingDuplicate(existingDateAmountMap.get(dateAmountKey)!)
                    : firstSeenDateAmounts.has(dateAmountKey)
                        ? fileDuplicate(firstSeenDateAmounts.get(dateAmountKey)!)
                        : null;
            } else {
                seenDateAmounts.add(dateAmountKey);
                firstSeenDateAmounts.set(dateAmountKey, line);
            }
        }

        return enrich(line, duplicate, reason, match);
    });
}

function enrich(
    line: StatementLine,
    duplicate: boolean,
    duplicateReason: DuplicateReason | null,
    duplicateMatch: DuplicateMatch | null
): AnnotatedStatementLine {
    return { ...line, duplicate, duplicateReason, duplicateMatch };
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

function existingDuplicate(row: ExistingStatementLineRow): ExistingDuplicateMatch {
    return { kind: 'existing', statementLine: row };
}

function fileDuplicate(line: StatementLine): FileDuplicateMatch {
    return { kind: 'file', previousLine: line };
}
