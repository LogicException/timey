import type { WorkSnapshot } from './types.ts';

export type WorkTransportAction = 'start' | 'pause' | 'resume' | 'stop';

export type WorkStatus = WorkSnapshot['status'];

const SESSION_PATH: Record<WorkTransportAction, string> = {
	start: '/api/work-sessions/start',
	pause: '/api/work-sessions/pause',
	resume: '/api/work-sessions/resume',
	stop: '/api/work-sessions/stop'
};

export function workTransportActions(status: WorkStatus): WorkTransportAction[] {
	if (status === 'running') return ['pause', 'stop'];
	if (status === 'paused') return ['resume', 'stop'];
	return ['start'];
}

export function workTransportBarVisible(_status: WorkStatus): boolean {
	return true;
}

export function workSessionPath(action: WorkTransportAction): string {
	return SESSION_PATH[action];
}

export function liveElapsedSeconds(
	elapsedSeconds: number,
	status: WorkStatus,
	originMs: number,
	nowMs: number
): number {
	if (status !== 'running') return elapsedSeconds;
	return elapsedSeconds + Math.max(0, Math.floor((nowMs - originMs) / 1000));
}
