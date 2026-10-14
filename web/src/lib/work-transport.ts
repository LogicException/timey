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

export function bindLiveClock(
	elapsedSeconds: number,
	status: WorkStatus,
	hasOpenInterval: boolean,
	onTick: (displaySeconds: number, nowMs: number) => void,
	now: () => number = Date.now
): () => void {
	const origin = now();
	const tick = () => {
		const current = now();
		onTick(liveElapsedSeconds(elapsedSeconds, status, origin, current), current);
	};
	tick();
	if (status !== 'running' && !hasOpenInterval) return () => {};
	const id = setInterval(tick, 1000);
	return () => clearInterval(id);
}
