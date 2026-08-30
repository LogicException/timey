import { describe, expect, it } from 'vitest';
import { weekTimeGridLayout } from './week-calendar.ts';

describe('weekTimeGridLayout', () => {
	it('does not stretch events past their time range', () => {
		expect(weekTimeGridLayout()).toEqual({ eventMinHeight: 1 });
	});
});
