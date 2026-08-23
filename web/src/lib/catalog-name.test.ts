import { describe, expect, it } from 'vitest';
import { isReservedTaskName, renameIfChanged } from './catalog-name.ts';

describe('renameIfChanged', () => {
	it('returns the trimmed name when it changed', () => {
		expect(renameIfChanged('Coding', '  Coding reviewen  ')).toBe('Coding reviewen');
	});

	it('returns null when the name is unchanged', () => {
		expect(renameIfChanged('Coding', 'Coding')).toBeNull();
		expect(renameIfChanged('Coding', '  Coding  ')).toBeNull();
	});

	it('returns null for an empty name', () => {
		expect(renameIfChanged('Coding', '   ')).toBeNull();
	});
});

describe('isReservedTaskName', () => {
	it('detects unbestimmt ignoring case and whitespace', () => {
		expect(isReservedTaskName('unbestimmt')).toBe(true);
		expect(isReservedTaskName('  Unbestimmt  ')).toBe(true);
		expect(isReservedTaskName('UNBESTIMMT')).toBe(true);
		expect(isReservedTaskName('Meeting')).toBe(false);
		expect(isReservedTaskName('')).toBe(false);
	});
});
