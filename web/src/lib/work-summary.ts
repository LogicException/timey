import type { Entry, WorkDaySummary, WorkSnapshot } from './types.ts';

export function totalWorkSeconds(days: ReadonlyArray<WorkDaySummary>): number {
	return days.reduce((sum, day) => sum + Math.max(0, day.elapsed_seconds), 0);
}

export function workAllowsTimer(work: WorkSnapshot | null): boolean {
	return work?.status === 'running';
}

export function taskTimerCanStart(work: WorkSnapshot | null, timer: Entry | null): boolean {
	return workAllowsTimer(work) && timer == null;
}

export function taskTimerStopError(taskId: number | null): string | null {
	return taskId == null ? 'Task ist erforderlich' : null;
}

export function taskTimerBarText(timer: Entry): string {
	if (timer.task_name == null || timer.task_name === '') return 'Task wählen';
	if (timer.project_name == null || timer.project_name === '') return timer.task_name;
	return `${timer.task_name} · ${timer.project_name}`;
}

export type TimerAssignmentBody = {
	task_id: number;
	project_id: number | null;
	start_at: string;
	end_at: null;
};

export type TimerAssignmentResult =
	| { ok: true; body: TimerAssignmentBody }
	| { ok: false; error: string };

export function timerAssignmentPayload(
	timer: Entry,
	taskId: number | null,
	projectId: number | null
): TimerAssignmentResult {
	const error = taskTimerStopError(taskId);
	if (error != null || taskId == null) {
		return { ok: false, error: error ?? 'Task ist erforderlich' };
	}
	return {
		ok: true,
		body: {
			task_id: taskId,
			project_id: projectId,
			start_at: timer.start_at,
			end_at: null
		}
	};
}
