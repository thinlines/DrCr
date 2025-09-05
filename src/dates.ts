/*
	DrCr: Web-based double-entry bookkeeping framework
*/

import dayjs, { Dayjs } from 'dayjs';
import advancedFormat from 'dayjs/plugin/advancedFormat';

import { db } from './db.ts';

dayjs.extend(advancedFormat);

// Format a YYYY-MM-DD string according to user preference
export function fmtDate(dateStr: string | null | undefined): string {
    if (!dateStr) return '';
    const style = db.metadata.date_style ?? 'YYYY-MM-DD';
    // Use dayjs to format; assume input is YYYY-MM-DD
    const d = dayjs(dateStr);
    // Guard invalid input
    if (!d.isValid()) return dateStr;
    return d.format(style);
}

// Convenience for a range like "start to end"
export function fmtDateRange(start: string | null | undefined, end: string | null | undefined): string {
    return `${fmtDate(start)} to ${fmtDate(end)}`.trim();
}

// Column label helpers for reports
export function labelForReportMonth(d: Dayjs, isCalendarMonth: boolean): string {
    const style = db.metadata.date_style ?? 'YYYY-MM-DD';
    if (isCalendarMonth) {
        // Prefer month+year without day
        if (style === 'YYYY-MM-DD') return d.format('YYYY-MM');
        if (style.includes('MMMM')) return d.format('MMMM YYYY');
        if (style.includes('MMM')) return d.format('MMM YYYY');
        if (style.includes('/')) return d.format('MM/YYYY');
        if (style.includes('-')) return d.format('YYYY-MM');
        return d.format('YYYY-MM');
    }
    // If not a full calendar month span, include the exact end date per preference
    return fmtDate(d.format('YYYY-MM-DD'));
}

// Subtitle helpers used across reports

// Returns a description for multiple monthly comparison periods, e.g.
// "For monthly periods ending on the 31st" or a calendar-month message.
export function monthlyPeriodsSubtitle(dt: string | null | undefined, compareUnit: string, comparePeriods: number): string | undefined {
    if (!dt) return undefined;
    if (compareUnit !== 'months' || comparePeriods <= 1) return undefined;
    const dayjsDt = dayjs(dt);
    const isEom = dayjsDt.add(1, 'day').isSame(dayjsDt.set('date', 1).add(1, 'month'));
    if (isEom) {
        return 'For calendar months ending on the last day of the month';
    }
    return `For monthly periods ending on the ${dayjsDt.format('Do')}`;
}

// Income statement subtitle: show explicit date range when single period,
// otherwise fall back to monthly periods description when comparing months.
export function incomeStatementSubtitle(
    dtStart: string | null | undefined,
    dt: string | null | undefined,
    compareUnit: string,
    comparePeriods: number,
): string | undefined {
    if (comparePeriods === 1 && dtStart && dt) {
        return fmtDateRange(dtStart, dt);
    }
    return monthlyPeriodsSubtitle(dt, compareUnit, comparePeriods);
}

// Point-in-time reports (e.g., Balance Sheet / Trial Balance):
// Prefer monthly periods description when comparing months; otherwise "As at <date>".
export function asAtSubtitle(
    dt: string | null | undefined,
    compareUnit?: string,
    comparePeriods?: number,
): string | undefined {
    const monthly = monthlyPeriodsSubtitle(dt, compareUnit ?? 'years', comparePeriods ?? 1);
    if (monthly) return monthly;
    return dt ? 'As at ' + fmtDate(dt) : undefined;
}
