import { describe, expect, it } from 'vitest';
import { durationBetween, formatHm, formatWorkDuration } from './format.ts';

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
