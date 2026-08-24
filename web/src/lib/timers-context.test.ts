import { describe, expect, it } from 'vitest';
import { REFRESH_TIMERS_KEY } from './timers-context.ts';

describe('REFRESH_TIMERS_KEY', () => {
	it('is a unique symbol for layout context', () => {
		expect(typeof REFRESH_TIMERS_KEY).toBe('symbol');
		expect(REFRESH_TIMERS_KEY).not.toBe(Symbol('refreshTimers'));
	});
});
