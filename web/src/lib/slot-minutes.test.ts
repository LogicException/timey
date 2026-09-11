import { describe, expect, it } from 'vitest';
import {
	DEFAULT_SLOT_MINUTES,
	effectiveSlotMinutes,
	parseSlotMinutesInput
} from './slot-minutes.ts';

describe('effectiveSlotMinutes', () => {
	it('uses 60 when the setting is empty', () => {
		expect(DEFAULT_SLOT_MINUTES).toBe(60);
		expect(effectiveSlotMinutes(null)).toBe(60);
		expect(effectiveSlotMinutes(undefined)).toBe(60);
	});

	it('returns a stored positive duration', () => {
		expect(effectiveSlotMinutes(30)).toBe(30);
	});
});

describe('parseSlotMinutesInput', () => {
	it('treats empty input as the default setting', () => {
		expect(parseSlotMinutesInput('')).toEqual({ ok: true, value: null });
		expect(parseSlotMinutesInput('  ')).toEqual({ ok: true, value: null });
	});

	it('accepts positive integers', () => {
		expect(parseSlotMinutesInput('1')).toEqual({ ok: true, value: 1 });
		expect(parseSlotMinutesInput('90')).toEqual({ ok: true, value: 90 });
	});

	it('rejects zero, negative, decimals and text', () => {
		expect(parseSlotMinutesInput('0')).toEqual({ ok: false });
		expect(parseSlotMinutesInput('-1')).toEqual({ ok: false });
		expect(parseSlotMinutesInput('1.5')).toEqual({ ok: false });
		expect(parseSlotMinutesInput('abc')).toEqual({ ok: false });
	});
});
