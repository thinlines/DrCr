/*
	DrCr: Web-based double-entry bookkeeping framework
*/

import dayjs, { Dayjs } from 'dayjs';
import { db } from './db.ts';

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

