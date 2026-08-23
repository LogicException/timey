import { describe, expect, it } from 'vitest';
import { totalWorkSeconds, workAllowsTimer } from './work-summary.ts';
import type { WorkSnapshot } from './types.ts';

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
