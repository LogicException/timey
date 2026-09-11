import { describe, expect, it } from 'vitest';
import { createWorkChangedBus, REFRESH_TIMERS_KEY } from './timers-context.ts';

describe('REFRESH_TIMERS_KEY', () => {
	it('is a unique symbol for layout context', () => {
		expect(typeof REFRESH_TIMERS_KEY).toBe('symbol');
		expect(REFRESH_TIMERS_KEY).not.toBe(Symbol('refreshTimers'));
	});
});

describe('createWorkChangedBus', () => {
	it('notifies subscribers after a work change', async () => {
		const bus = createWorkChangedBus();
		let calls = 0;
		bus.subscribe(() => {
			calls += 1;
		});
		await bus.notify();
		expect(calls).toBe(1);
	});

	it('does not notify after unsubscribe', async () => {
		const bus = createWorkChangedBus();
		let calls = 0;
		const unsubscribe = bus.subscribe(() => {
			calls += 1;
		});
		unsubscribe();
		await bus.notify();
		expect(calls).toBe(0);
	});

	it('still notifies remaining listeners if one fails', async () => {
		const bus = createWorkChangedBus();
		let calls = 0;
		bus.subscribe(() => {
			throw new Error('listener failed');
		});
		bus.subscribe(() => {
			calls += 1;
		});
		await bus.notify();
		expect(calls).toBe(1);
	});
});
