import { describe, expect, it } from 'vitest';
import { calendarEventColors, weekTimeGridLayout } from './week-calendar.ts';

describe('weekTimeGridLayout', () => {
	it('does not stretch events past their time range', () => {
		expect(weekTimeGridLayout()).toEqual({ eventMinHeight: 1 });
	});
});

describe('calendarEventColors', () => {
	it('uses the task color for background and border', () => {
		expect(calendarEventColors('#3f9d6c')).toEqual({
			backgroundColor: '#3f9d6c',
			borderColor: '#3f9d6c',
			textColor: '#ece7dc'
		});
	});

	it('uses dark text on a light background', () => {
		expect(calendarEventColors('#f5f5f5')).toEqual({
			backgroundColor: '#f5f5f5',
			borderColor: '#f5f5f5',
			textColor: '#121512'
		});
	});

	it('falls back when the task has no color', () => {
		expect(calendarEventColors(null)).toEqual({
			backgroundColor: '#2f5d45',
			borderColor: '#3f9d6c',
			textColor: '#ece7dc'
		});
		expect(calendarEventColors(undefined)).toEqual({
			backgroundColor: '#2f5d45',
			borderColor: '#3f9d6c',
			textColor: '#ece7dc'
		});
	});
});
