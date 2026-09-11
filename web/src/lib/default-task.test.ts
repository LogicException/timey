import { describe, expect, it } from 'vitest';
import { suggestedTaskId } from './default-task.ts';

const coding = { id: 3 };
const meeting = { id: 1 };
const review = { id: 4 };

describe('suggestedTaskId', () => {
	it('returns null when there are no tasks', () => {
		expect(suggestedTaskId([], 3)).toBeNull();
		expect(suggestedTaskId([], null)).toBeNull();
	});

	it('prefers a configured default that is still in the list', () => {
		expect(suggestedTaskId([meeting, coding, review], 3)).toBe(3);
	});

	it('falls back to the first task when no default is set', () => {
		expect(suggestedTaskId([meeting, coding], null)).toBe(1);
		expect(suggestedTaskId([meeting, coding], undefined)).toBe(1);
	});

	it('falls back to the first task when the default is missing from the list', () => {
		expect(suggestedTaskId([meeting, coding], 99)).toBe(1);
	});

	it('does not fall back when fallback is none', () => {
		expect(suggestedTaskId([meeting, coding], null, 'none')).toBeNull();
		expect(suggestedTaskId([meeting, coding], 99, 'none')).toBeNull();
		expect(suggestedTaskId([meeting, coding], 3, 'none')).toBe(3);
	});
});
