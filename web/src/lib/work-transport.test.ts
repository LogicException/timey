import { afterEach, describe, expect, it, vi } from 'vitest';
import {
	bindLiveClock,
	liveElapsedSeconds,
	workSessionPath,
	workTransportActions,
	workTransportBarVisible
} from './work-transport.ts';

describe('workTransportActions', () => {
	it('shows start when idle', () => {
		expect(workTransportActions(null)).toEqual(['start']);
		expect(workTransportActions('stopped')).toEqual(['start']);
	});

	it('shows pause and stop while running', () => {
		expect(workTransportActions('running')).toEqual(['pause', 'stop']);
	});

	it('shows resume and stop while paused', () => {
		expect(workTransportActions('paused')).toEqual(['resume', 'stop']);
	});
});

describe('workTransportBarVisible', () => {
	it('is visible for every work status including idle', () => {
		expect(workTransportBarVisible('running')).toBe(true);
		expect(workTransportBarVisible('paused')).toBe(true);
		expect(workTransportBarVisible('stopped')).toBe(true);
		expect(workTransportBarVisible(null)).toBe(true);
	});
});

describe('workSessionPath', () => {
	it('maps each action to its session endpoint', () => {
		expect(workSessionPath('start')).toBe('/api/work-sessions/start');
		expect(workSessionPath('pause')).toBe('/api/work-sessions/pause');
		expect(workSessionPath('resume')).toBe('/api/work-sessions/resume');
		expect(workSessionPath('stop')).toBe('/api/work-sessions/stop');
	});
});

describe('liveElapsedSeconds', () => {
	it('stays at the base elapsed time when not running', () => {
		expect(liveElapsedSeconds(90, 'paused', 1_000, 6_000)).toBe(90);
		expect(liveElapsedSeconds(90, 'stopped', 1_000, 6_000)).toBe(90);
		expect(liveElapsedSeconds(90, null, 1_000, 6_000)).toBe(90);
	});

	it('adds elapsed wall time while running', () => {
		expect(liveElapsedSeconds(90, 'running', 1_000, 6_500)).toBe(95);
	});
});

describe('bindLiveClock', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('advances a running clock from a stable origin', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date('2026-10-14T10:00:00.000Z'));
		const ticks: number[] = [];
		const stop = bindLiveClock(100, 'running', false, (seconds) => {
			ticks.push(seconds);
		});

		expect(ticks).toEqual([100]);
		vi.advanceTimersByTime(3000);
		expect(ticks).toEqual([100, 101, 102, 103]);
		stop();
		vi.advanceTimersByTime(2000);
		expect(ticks).toEqual([100, 101, 102, 103]);
	});

	it('restarts from the supplied base instead of stacking previous ticks', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date('2026-10-14T10:00:00.000Z'));
		let seconds = 0;
		const stop = bindLiveClock(100, 'running', false, (value) => {
			seconds = value;
		});
		vi.advanceTimersByTime(5000);
		expect(seconds).toBe(105);
		stop();

		const restarted = bindLiveClock(105, 'running', false, (value) => {
			seconds = value;
		});
		expect(seconds).toBe(105);
		vi.advanceTimersByTime(2000);
		expect(seconds).toBe(107);
		restarted();
	});

	it('does not schedule ticks when work is idle', () => {
		vi.useFakeTimers();
		const ticks: number[] = [];
		const stop = bindLiveClock(40, null, false, (seconds) => {
			ticks.push(seconds);
		});
		vi.advanceTimersByTime(5000);
		expect(ticks).toEqual([40]);
		stop();
	});

	it('keeps publishing the current time for an open interval while paused', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date('2026-10-14T10:00:00.000Z'));
		const samples: Array<{ seconds: number; nowMs: number }> = [];
		const stop = bindLiveClock(10, 'paused', true, (seconds, nowMs) => {
			samples.push({ seconds, nowMs });
		});
		vi.advanceTimersByTime(2000);
		expect(samples.map((sample) => sample.seconds)).toEqual([10, 10, 10]);
		expect(samples[2]?.nowMs).toBe(samples[0]!.nowMs + 2000);
		stop();
	});
});
