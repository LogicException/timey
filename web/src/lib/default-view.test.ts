import { describe, expect, it } from 'vitest';
import { homePath, parseDefaultView } from './default-view.ts';

describe('parseDefaultView', () => {
	it('accepts day and week', () => {
		expect(parseDefaultView('day')).toBe('day');
		expect(parseDefaultView('week')).toBe('week');
	});

	it('falls back to day for unknown values', () => {
		expect(parseDefaultView('month')).toBe('day');
		expect(parseDefaultView(undefined)).toBe('day');
	});
});

describe('homePath', () => {
	it('maps the stored view to a route', () => {
		expect(homePath('day')).toBe('/day');
		expect(homePath('week')).toBe('/week');
	});
});
