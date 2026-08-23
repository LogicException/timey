import { describe, expect, it } from 'vitest';
import {
	addDays,
	berlinLocalToUtc,
	endOfMonth,
	endOfWeek,
	formatBerlinDate,
	rangeForPreset,
	showsManualDateFields,
	startOfMonth,
	startOfWeek
} from './dates.ts';

describe('week helpers', () => {
	it('starts weeks on Monday', () => {
		expect(startOfWeek('2026-08-21')).toBe('2026-08-17');
		expect(endOfWeek('2026-08-21')).toBe('2026-08-23');
	});

	it('handles Sunday as end of the ISO week', () => {
		expect(startOfWeek('2026-08-23')).toBe('2026-08-17');
	});
});

describe('month helpers', () => {
	it('uses first and last calendar day', () => {
		expect(startOfMonth('2026-08-21')).toBe('2026-08-01');
		expect(endOfMonth('2026-08-21')).toBe('2026-08-31');
	});
});

describe('range presets', () => {
	it('defaults today to a single day', () => {
		expect(rangeForPreset('today', '2026-08-21')).toEqual({
			from: '2026-08-21',
			to: '2026-08-21'
		});
	});

	it('computes last week from a Friday', () => {
		expect(rangeForPreset('last_week', '2026-08-21')).toEqual({
			from: '2026-08-10',
			to: '2026-08-16'
		});
	});

	it('computes last month across year boundaries', () => {
		expect(rangeForPreset('last_month', '2026-01-10')).toEqual({
			from: '2025-12-01',
			to: '2025-12-31'
		});
	});
});

describe('showsManualDateFields', () => {
	it('is true only for the custom preset', () => {
		expect(showsManualDateFields('custom')).toBe(true);
	});

	it('is false for named presets', () => {
		expect(showsManualDateFields('today')).toBe(false);
		expect(showsManualDateFields('yesterday')).toBe(false);
		expect(showsManualDateFields('this_week')).toBe(false);
		expect(showsManualDateFields('last_week')).toBe(false);
		expect(showsManualDateFields('this_month')).toBe(false);
		expect(showsManualDateFields('last_month')).toBe(false);
	});
});

describe('Berlin conversion', () => {
	it('maps CEST morning to UTC minus two hours', () => {
		const utc = berlinLocalToUtc('2026-08-21', 9, 0);
		expect(utc.toISOString()).toBe('2026-08-21T07:00:00.000Z');
		expect(formatBerlinDate(utc)).toBe('2026-08-21');
	});

	it('maps CET morning to UTC minus one hour', () => {
		const utc = berlinLocalToUtc('2026-01-15', 9, 0);
		expect(utc.toISOString()).toBe('2026-01-15T08:00:00.000Z');
	});
});

describe('addDays', () => {
	it('crosses months', () => {
		expect(addDays('2026-08-31', 1)).toBe('2026-09-01');
	});
});
