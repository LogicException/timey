import type { WorkDaySummary, WorkSnapshot } from './types.ts';

export function totalWorkSeconds(days: ReadonlyArray<WorkDaySummary>): number {
	return days.reduce((sum, day) => sum + Math.max(0, day.elapsed_seconds), 0);
}

export function workAllowsTimer(work: WorkSnapshot | null): boolean {
	return work?.status === 'running';
}
