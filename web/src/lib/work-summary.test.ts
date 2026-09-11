import { describe, expect, it } from 'vitest';
import {
	taskTimerBarText,
	taskTimerCanStart,
	taskTimerStopError,
	timerAssignmentPayload,
	totalWorkSeconds,
	workAllowsTimer
} from './work-summary.ts';
import type { Entry, WorkSnapshot } from './types.ts';

describe('totalWorkSeconds', () => {
	it('returns zero for an empty list', () => {
		expect(totalWorkSeconds([])).toBe(0);
	});

	it('sums elapsed seconds across days', () => {
		expect(
			totalWorkSeconds([
				{ local_date: '2026-08-21', elapsed_seconds: 3600 },
				{ local_date: '2026-08-22', elapsed_seconds: 90 }
			])
		).toBe(3690);
	});
});

describe('workAllowsTimer', () => {
	it('allows a timer only while work is running', () => {
		const running: WorkSnapshot = {
			session_id: 1,
			status: 'running',
			local_date: '2026-08-23',
			elapsed_seconds: 10
		};
		expect(workAllowsTimer(running)).toBe(true);
		expect(workAllowsTimer({ ...running, status: 'paused' })).toBe(false);
		expect(workAllowsTimer(null)).toBe(false);
	});
});

describe('taskTimerCanStart', () => {
	const running: WorkSnapshot = {
		session_id: 1,
		status: 'running',
		local_date: '2026-08-23',
		elapsed_seconds: 10
	};

	it('allows a task timer only while work is running and no timer exists', () => {
		expect(taskTimerCanStart(running, null)).toBe(true);
		expect(taskTimerCanStart({ ...running, status: 'paused' }, null)).toBe(false);
		expect(taskTimerCanStart(null, null)).toBe(false);
	});

	it('blocks a second task timer while one is already running', () => {
		const timer: Entry = {
			id: 9,
			task_id: null,
			project_id: null,
			task_name: null,
			project_name: null,
			start_at: '2026-08-23T07:00:00.000Z',
			end_at: null,
			status: 'needs_task'
		};
		expect(taskTimerCanStart(running, timer)).toBe(false);
	});
});

describe('taskTimerStopError', () => {
	it('requires a task before stopping', () => {
		expect(taskTimerStopError(null)).toBe('Task ist erforderlich');
		expect(taskTimerStopError(4)).toBeNull();
	});
});

describe('taskTimerBarText', () => {
	it('asks to choose a task when none is assigned', () => {
		expect(
			taskTimerBarText({
				id: 9,
				task_id: null,
				project_id: null,
				task_name: null,
				project_name: null,
				start_at: '2026-08-23T07:00:00.000Z',
				end_at: null,
				status: 'running'
			})
		).toBe('Task wählen');
	});

	it('shows task and optional project', () => {
		expect(
			taskTimerBarText({
				id: 9,
				task_id: 3,
				project_id: null,
				task_name: 'Coding',
				project_name: null,
				start_at: '2026-08-23T07:00:00.000Z',
				end_at: null,
				status: 'running'
			})
		).toBe('Coding');
		expect(
			taskTimerBarText({
				id: 9,
				task_id: 3,
				project_id: 2,
				task_name: 'Coding',
				project_name: 'Elba',
				start_at: '2026-08-23T07:00:00.000Z',
				end_at: null,
				status: 'running'
			})
		).toBe('Coding · Elba');
	});
});

describe('timerAssignmentPayload', () => {
	const timer: Entry = {
		id: 9,
		task_id: null,
		project_id: null,
		task_name: null,
		project_name: null,
		start_at: '2026-08-23T07:00:00.000Z',
		end_at: null,
		status: 'running'
	};

	it('rejects a missing task', () => {
		expect(timerAssignmentPayload(timer, null, 2)).toEqual({
			ok: false,
			error: 'Task ist erforderlich'
		});
	});

	it('keeps the running start time and optional project', () => {
		expect(timerAssignmentPayload(timer, 3, 2)).toEqual({
			ok: true,
			body: {
				task_id: 3,
				project_id: 2,
				start_at: '2026-08-23T07:00:00.000Z',
				end_at: null
			}
		});
		expect(timerAssignmentPayload(timer, 3, null)).toEqual({
			ok: true,
			body: {
				task_id: 3,
				project_id: null,
				start_at: '2026-08-23T07:00:00.000Z',
				end_at: null
			}
		});
	});
});
