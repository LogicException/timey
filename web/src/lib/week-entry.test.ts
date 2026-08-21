import { describe, expect, it } from 'vitest';
import type { Entry } from './types.ts';
import { applyBerlinTimes, entryToEditState, savePayload } from './week-entry.ts';

function completeEntry(overrides: Partial<Entry> = {}): Entry {
	return {
		id: 12,
		task_id: 3,
		project_id: 8,
		aufgabe_id: null,
		task_name: 'E-Mail',
		project_name: 'Efa',
		aufgabe_name: null,
		start_at: '2026-08-21T06:45:00.000Z',
		end_at: '2026-08-21T08:00:00.000Z',
		status: 'complete',
		...overrides
	};
}

describe('entryToEditState', () => {
	it('maps a completed entry to form fields', () => {
		expect(entryToEditState(completeEntry())).toEqual({
			startIso: '2026-08-21T06:45:00.000Z',
			endIso: '2026-08-21T08:00:00.000Z',
			taskId: 3,
			projectId: 8,
			aufgabeId: null
		});
	});

	it('returns null for a running entry without end_at', () => {
		expect(
			entryToEditState(
				completeEntry({ end_at: null, status: 'running', task_id: null, task_name: null })
			)
		).toBeNull();
	});
});

describe('savePayload', () => {
	it('builds the POST/PATCH body from form fields', () => {
		expect(
			savePayload({
				taskId: 3,
				projectId: 8,
				aufgabeId: 1,
				startIso: '2026-08-21T06:45:00.000Z',
				endIso: '2026-08-21T08:00:00.000Z'
			})
		).toEqual({
			task_id: 3,
			project_id: 8,
			aufgabe_id: 1,
			start_at: '2026-08-21T06:45:00.000Z',
			end_at: '2026-08-21T08:00:00.000Z'
		});
	});
});

describe('applyBerlinTimes', () => {
	it('keeps the calendar day and replaces the clock time', () => {
		const next = applyBerlinTimes('2026-08-21T06:45:00.000Z', 10, 15);
		expect(next).toBe('2026-08-21T08:15:00.000Z');
	});
});
