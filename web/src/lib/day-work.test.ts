import { describe, expect, it } from 'vitest';
import { closeWorkModalState, intervalToForm, saveWorkPayload, workIntervalsNewestFirst } from './day-work.ts';
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

describe('workIntervalsNewestFirst', () => {
	it('orders intervals by start time descending', () => {
		const morning = interval({ id: 1, start_at: '2026-09-11T05:44:00.000Z' });
		const midday = interval({ id: 2, start_at: '2026-09-11T11:50:00.000Z' });
		const evening = interval({
			id: 3,
			start_at: '2026-09-11T17:56:00.000Z',
			end_at: '2026-09-11T18:00:00.000Z',
			open: true
		});
		expect(workIntervalsNewestFirst([morning, midday, evening]).map((item) => item.id)).toEqual([
			3, 2, 1
		]);
	});

	it('does not mutate the input list', () => {
		const morning = interval({ id: 1, start_at: '2026-09-11T05:44:00.000Z' });
		const evening = interval({ id: 2, start_at: '2026-09-11T17:56:00.000Z' });
		const input = [morning, evening];
		workIntervalsNewestFirst(input);
		expect(input.map((item) => item.id)).toEqual([1, 2]);
	});
});
