import { describe, expect, it } from 'vitest';
import { closeModalState, entryToForm, savePayload } from './day-entry.ts';
import type { Entry } from './types.ts';

function completeEntry(overrides: Partial<Entry> = {}): Entry {
	return {
		id: 12,
		task_id: 3,
		project_id: 8,
		task_name: 'E-Mail',
		project_name: 'Efa',
		start_at: '2026-08-21T06:45:00.000Z',
		end_at: '2026-08-21T08:00:00.000Z',
		status: 'complete',
		...overrides
	};
}

describe('entryToForm', () => {
	it('maps a completed entry to Berlin clock times and ids', () => {
		expect(entryToForm(completeEntry())).toEqual({
			fromH: 8,
			fromM: 45,
			toH: 10,
			toM: 0,
			taskId: 3,
			projectId: 8
		});
	});

	it('uses start time for both clocks when end_at is missing', () => {
		expect(
			entryToForm(
				completeEntry({ end_at: null, status: 'running', task_id: null, task_name: null })
			)
		).toEqual({
			fromH: 8,
			fromM: 45,
			toH: 8,
			toM: 45,
			taskId: null,
			projectId: 8
		});
	});
});

describe('savePayload', () => {
	it('builds the POST/PATCH body from the day and form fields', () => {
		expect(
			savePayload('2026-08-21', {
				fromH: 8,
				fromM: 45,
				toH: 10,
				toM: 0,
				taskId: 3,
				projectId: 8
			})
		).toEqual({
			task_id: 3,
			project_id: 8,
			start_at: '2026-08-21T06:45:00.000Z',
			end_at: '2026-08-21T08:00:00.000Z'
		});
	});
});

describe('closeModalState', () => {
	it('clears open and editing after cancel or successful save', () => {
		expect(closeModalState()).toEqual({ open: false, editing: null });
	});
});
