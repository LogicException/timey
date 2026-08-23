import { describe, expect, it } from 'vitest';
import type { WorkDaySummary } from './types.ts';
import { workIntervalEvents } from './week-work-intervals.ts';

function day(overrides: Partial<WorkDaySummary> & Pick<WorkDaySummary, 'local_date'>): WorkDaySummary {
	return {
		elapsed_seconds: 0,
		...overrides
	};
}

describe('workIntervalEvents', () => {
	it('returns no events for an empty list', () => {
		expect(workIntervalEvents([])).toEqual([]);
	});

	it('maps intervals from multiple days to background events', () => {
		expect(
			workIntervalEvents([
				day({
					local_date: '2026-08-21',
					intervals: [
						{
							start_at: '2026-08-21T06:00:00Z',
							end_at: '2026-08-21T07:00:00Z'
						}
					]
				}),
				day({
					local_date: '2026-08-22',
					intervals: [
						{
							start_at: '2026-08-22T07:30:00Z',
							end_at: '2026-08-22T08:30:00Z'
						}
					]
				})
			])
		).toEqual([
			{
				id: 'work-2026-08-21-0',
				start: '2026-08-21T06:00:00Z',
				end: '2026-08-21T07:00:00Z',
				display: 'background',
				classNames: ['work-interval']
			},
			{
				id: 'work-2026-08-22-0',
				start: '2026-08-22T07:30:00Z',
				end: '2026-08-22T08:30:00Z',
				display: 'background',
				classNames: ['work-interval']
			}
		]);
	});

	it('keeps a pause gap as separate events', () => {
		expect(
			workIntervalEvents([
				day({
					local_date: '2026-08-21',
					intervals: [
						{
							start_at: '2026-08-21T06:00:00Z',
							end_at: '2026-08-21T07:00:00Z'
						},
						{
							start_at: '2026-08-21T07:30:00Z',
							end_at: '2026-08-21T08:30:00Z'
						}
					]
				})
			])
		).toEqual([
			{
				id: 'work-2026-08-21-0',
				start: '2026-08-21T06:00:00Z',
				end: '2026-08-21T07:00:00Z',
				display: 'background',
				classNames: ['work-interval']
			},
			{
				id: 'work-2026-08-21-1',
				start: '2026-08-21T07:30:00Z',
				end: '2026-08-21T08:30:00Z',
				display: 'background',
				classNames: ['work-interval']
			}
		]);
	});

	it('treats missing intervals as empty', () => {
		expect(workIntervalEvents([day({ local_date: '2026-08-21' })])).toEqual([]);
	});
});
