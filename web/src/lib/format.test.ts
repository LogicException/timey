import { describe, expect, it } from 'vitest';
import { durationBetween, formatHm, formatWorkDuration, totalDurationSeconds } from './format.ts';

describe('formatWorkDuration', () => {
	it('uses German plural labels', () => {
		expect(formatWorkDuration(7 * 3600 + 56 * 60)).toBe('7 Stunden 56 Minuten');
	});

	it('uses singular hour and minute', () => {
		expect(formatWorkDuration(3660)).toBe('1 Stunde 1 Minute');
	});
});

describe('formatHm', () => {
	it('pads minutes', () => {
		expect(formatHm(90 * 60)).toBe('1:30');
	});
});

describe('durationBetween', () => {
	it('returns zero without end', () => {
		expect(durationBetween('2026-08-21T07:00:00Z', null)).toBe(0);
	});
});

describe('totalDurationSeconds', () => {
	it('returns zero for an empty list', () => {
		expect(totalDurationSeconds([])).toBe(0);
	});

	it('sums completed entries', () => {
		expect(
			totalDurationSeconds([
				{ start_at: '2026-08-21T12:00:00Z', end_at: '2026-08-21T12:30:00Z' },
				{ start_at: '2026-08-21T09:45:00Z', end_at: '2026-08-21T11:00:00Z' }
			])
		).toBe(105 * 60);
	});

	it('treats running entries as zero', () => {
		expect(
			totalDurationSeconds([
				{ start_at: '2026-08-21T12:00:00Z', end_at: '2026-08-21T12:30:00Z' },
				{ start_at: '2026-08-21T13:00:00Z', end_at: null }
			])
		).toBe(30 * 60);
	});
});
