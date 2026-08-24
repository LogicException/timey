import { describe, expect, it } from 'vitest';
import { closeWorkModalState, intervalToForm, saveWorkPayload } from './day-work.ts';
import type { WorkInterval } from './types.ts';

function interval(overrides: Partial<WorkInterval> = {}): WorkInterval {
	return {
		id: 4,
		start_at: '2026-08-21T06:45:00.000Z',
		end_at: '2026-08-21T08:00:00.000Z',
		open: false,
		...overrides
	};
}

describe('intervalToForm', () => {
	it('maps a closed interval to Berlin clock times', () => {
		expect(intervalToForm(interval())).toEqual({
			fromH: 8,
			fromM: 45,
			toH: 10,
			toM: 0
		});
	});

	it('uses start for both clocks when the interval is open', () => {
		expect(intervalToForm(interval({ open: true, end_at: '2026-08-21T07:00:00.000Z' }))).toEqual({
			fromH: 8,
			fromM: 45,
			toH: 8,
			toM: 45
		});
	});
});

describe('saveWorkPayload', () => {
	it('builds start and end for a closed interval', () => {
		expect(
			saveWorkPayload('2026-08-21', { fromH: 8, fromM: 45, toH: 10, toM: 0 }, false)
		).toEqual({
			start_at: '2026-08-21T06:45:00.000Z',
			end_at: '2026-08-21T08:00:00.000Z'
		});
	});

	it('omits end_at for an open interval', () => {
		expect(
			saveWorkPayload('2026-08-21', { fromH: 7, fromM: 0, toH: 10, toM: 0 }, true)
		).toEqual({
			start_at: '2026-08-21T05:00:00.000Z'
		});
	});
});

describe('closeWorkModalState', () => {
	it('clears open and editing after cancel or successful save', () => {
		expect(closeWorkModalState()).toEqual({ open: false, editing: null });
	});
});
