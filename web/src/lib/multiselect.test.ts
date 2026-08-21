import { describe, expect, it } from 'vitest';
import { filterNamedItems, toggleId } from './multiselect.ts';

const items = [
	{ id: 1, name: 'Meeting', archived: false },
	{ id: 2, name: 'E-Mail', archived: false },
	{ id: 3, name: 'Coding', archived: true }
];

describe('filterNamedItems', () => {
	it('returns all items for an empty query', () => {
		expect(filterNamedItems(items, '  ')).toEqual(items);
	});

	it('matches case-insensitively by substring', () => {
		expect(filterNamedItems(items, 'mail').map((item) => item.id)).toEqual([2]);
	});
});

describe('toggleId', () => {
	it('adds missing ids and removes existing ones', () => {
		expect(toggleId([1], 2)).toEqual([1, 2]);
		expect(toggleId([1, 2], 1)).toEqual([2]);
	});
});
