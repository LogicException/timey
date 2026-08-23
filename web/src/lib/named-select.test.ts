import { describe, expect, it } from 'vitest';
import { namedSelectOptions } from './named-select.ts';
import type { NamedItem } from './types.ts';

const meeting: NamedItem = { id: 1, name: 'Meeting', archived: false };
const archived: NamedItem = { id: 2, name: 'Alt', archived: true };
const system: NamedItem = { id: 3, name: 'unbestimmt', archived: false, system: true };

describe('namedSelectOptions', () => {
	it('hides archived and system tasks', () => {
		expect(namedSelectOptions([meeting, archived, system], null, null)).toEqual([
			{ id: 1, name: 'Meeting' }
		]);
	});

	it('keeps the current system value as an option', () => {
		expect(namedSelectOptions([meeting, system], 3, null)).toEqual([
			{ id: 3, name: 'unbestimmt' },
			{ id: 1, name: 'Meeting' }
		]);
	});

	it('uses currentLabel when the current value is not in the list', () => {
		expect(namedSelectOptions([meeting], 9, 'unbestimmt')).toEqual([
			{ id: 9, name: 'unbestimmt' },
			{ id: 1, name: 'Meeting' }
		]);
	});
});
