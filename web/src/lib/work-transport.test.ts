import { describe, expect, it } from 'vitest';
import { liveElapsedSeconds, workSessionPath, workTransportActions, workTransportBarVisible } from './work-transport.ts';

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
