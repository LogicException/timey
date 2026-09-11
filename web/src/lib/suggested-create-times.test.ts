import { describe, expect, it } from 'vitest';
import { latestStopIso, suggestedCreateClock } from './suggested-create-times.ts';

describe('latestStopIso', () => {
	it('returns the latest non-empty stop', () => {
		expect(
			latestStopIso(['2026-08-21T06:45:00.000Z', '2026-08-21T08:00:00.000Z', null])
		).toBe('2026-08-21T08:00:00.000Z');
	});

	it('returns null when every stop is missing', () => {
		expect(latestStopIso([null, undefined, ''])).toBeNull();
		expect(latestStopIso([])).toBeNull();
	});
});

describe('suggestedCreateClock', () => {
	it('sets Bis to Von plus the slot duration', () => {
		expect(suggestedCreateClock('2026-08-21T08:00:00.000Z', 30)).toEqual({
			fromH: 10,
			fromM: 0,
			toH: 10,
			toM: 30
		});
	});

	it('uses 60 minutes when slot duration is empty', () => {
		expect(suggestedCreateClock('2026-08-21T08:00:00.000Z', null)).toEqual({
			fromH: 10,
			fromM: 0,
			toH: 11,
			toM: 0
		});
	});

	it('clamps Bis to 23:59 on the same day', () => {
		expect(suggestedCreateClock('2026-08-21T21:30:00.000Z', 60)).toEqual({
			fromH: 23,
			fromM: 30,
			toH: 23,
			toM: 59
		});
	});
});
