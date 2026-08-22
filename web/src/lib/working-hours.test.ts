import { describe, expect, it } from 'vitest';
import {
	DEFAULT_WORK_END,
	DEFAULT_WORK_START,
	WEEK_VIEW_BUFFER_MINUTES,
	joinHm,
	splitHm,
	weekSlotTimes
} from './working-hours.ts';

describe('splitHm and joinHm', () => {
	it('splits and pads clock times', () => {
		expect(splitHm('07:30')).toEqual({ hours: 7, minutes: 30 });
		expect(joinHm(7, 30)).toBe('07:30');
		expect(joinHm(16, 15)).toBe('16:15');
	});
});

describe('weekSlotTimes', () => {
	it('applies a 30 minute buffer around usual hours', () => {
		expect(WEEK_VIEW_BUFFER_MINUTES).toBe(30);
		expect(DEFAULT_WORK_START).toBe('07:30');
		expect(DEFAULT_WORK_END).toBe('16:15');
		expect(weekSlotTimes(DEFAULT_WORK_START, DEFAULT_WORK_END)).toEqual({
			min: '07:00:00',
			max: '16:45:00'
		});
	});

	it('clamps the start of the day', () => {
		expect(weekSlotTimes('00:10', '08:00')).toEqual({
			min: '00:00:00',
			max: '08:30:00'
		});
	});

	it('clamps the end of the day to 24:00', () => {
		expect(weekSlotTimes('16:00', '23:45')).toEqual({
			min: '15:30:00',
			max: '24:00:00'
		});
	});

	it('accepts a custom buffer', () => {
		expect(weekSlotTimes('08:00', '17:00', 15)).toEqual({
			min: '07:45:00',
			max: '17:15:00'
		});
	});
});
