import { describe, expect, it } from 'vitest';
import {
	continuousViolationIntervalIds,
	daysWithBreakViolations,
	evaluateBreakCompliance
} from './break-compliance.ts';
import type { WorkInterval } from './types.ts';

function interval(id: number, start: string, end: string, open = false): WorkInterval {
	return { id, start_at: start, end_at: end, open };
}

function kinds(intervals: WorkInterval[], now?: Date): string[] {
	return evaluateBreakCompliance(intervals, now).map((item) => item.kind);
}

describe('evaluateBreakCompliance', () => {
	it('returns no violations for an empty day', () => {
		expect(evaluateBreakCompliance([])).toEqual([]);
	});

	it('accepts exactly six hours of continuous work', () => {
		expect(
			evaluateBreakCompliance([interval(1, '2026-08-21T06:00:00Z', '2026-08-21T12:00:00Z')])
		).toEqual([]);
	});

	it('flags continuous work one second over six hours', () => {
		const result = evaluateBreakCompliance([
			interval(1, '2026-08-21T06:00:00Z', '2026-08-21T12:00:01Z')
		]);
		expect(result.map((item) => item.kind)).toEqual([
			'continuous_too_long',
			'insufficient_break'
		]);
		expect(result[0]?.message).toBe('Durchgehende Arbeitszeit 6:00 ohne Pause (höchstens 6:00)');
		expect(result[0]?.intervalIds).toEqual([1]);
	});

	it('flags eight hours without a break as continuous and insufficient', () => {
		const result = evaluateBreakCompliance([
			interval(1, '2026-08-21T06:00:00Z', '2026-08-21T14:00:00Z')
		]);
		expect(result.map((item) => item.kind)).toEqual(['continuous_too_long', 'insufficient_break']);
		expect(result.find((item) => item.kind === 'insufficient_break')?.message).toBe(
			'Pausensumme 0:00, bei 8:00 Arbeitszeit sind 0:30 nötig'
		);
	});

	it('accepts eight hours with a thirty-minute qualifying break', () => {
		expect(
			kinds([
				interval(1, '2026-08-21T06:00:00Z', '2026-08-21T10:00:00Z'),
				interval(2, '2026-08-21T10:30:00Z', '2026-08-21T14:30:00Z')
			])
		).toEqual([]);
	});

	it('treats a ten-minute gap as too short and stitches continuous work', () => {
		const result = evaluateBreakCompliance([
			interval(1, '2026-08-21T06:00:00Z', '2026-08-21T10:00:00Z'),
			interval(2, '2026-08-21T10:10:00Z', '2026-08-21T14:10:00Z')
		]);
		expect(result.map((item) => item.kind)).toEqual([
			'continuous_too_long',
			'break_too_short',
			'insufficient_break'
		]);
		expect(result.find((item) => item.kind === 'break_too_short')?.message).toBe(
			'Pause 0:10 zählt nicht (mindestens 0:15)'
		);
		expect(result.find((item) => item.kind === 'continuous_too_long')?.intervalIds).toEqual([1, 2]);
	});

	it('does not flag a short pause when net work stays at most six hours', () => {
		expect(
			kinds([
				interval(1, '2026-08-21T06:00:00Z', '2026-08-21T08:00:00Z'),
				interval(2, '2026-08-21T08:10:00Z', '2026-08-21T10:00:00Z')
			])
		).toEqual([]);
	});

	it('accepts nine hours net with a thirty-minute break', () => {
		expect(
			kinds([
				interval(1, '2026-08-21T06:00:00Z', '2026-08-21T10:30:00Z'),
				interval(2, '2026-08-21T11:00:00Z', '2026-08-21T15:30:00Z')
			])
		).toEqual([]);
	});

	it('requires forty-five minutes of break after more than nine hours', () => {
		const result = evaluateBreakCompliance([
			interval(1, '2026-08-21T06:00:00Z', '2026-08-21T10:30:00Z'),
			interval(2, '2026-08-21T11:00:00Z', '2026-08-21T15:31:00Z')
		]);
		expect(result.map((item) => item.kind)).toEqual(['insufficient_break']);
		expect(result[0]?.message).toBe('Pausensumme 0:30, bei 9:01 Arbeitszeit sind 0:45 nötig');
	});

	it('counts two fifteen-minute breaks toward thirty minutes', () => {
		expect(
			kinds([
				interval(1, '2026-08-21T06:00:00Z', '2026-08-21T09:00:00Z'),
				interval(2, '2026-08-21T09:15:00Z', '2026-08-21T11:15:00Z'),
				interval(3, '2026-08-21T11:30:00Z', '2026-08-21T14:30:00Z')
			])
		).toEqual([]);
	});

	it('does not flag an open interval at exactly six hours', () => {
		expect(
			evaluateBreakCompliance(
				[interval(1, '2026-08-21T06:00:00Z', '2026-08-21T06:00:00Z', true)],
				new Date('2026-08-21T12:00:00Z')
			)
		).toEqual([]);
	});

	it('flags an open interval once now exceeds six hours', () => {
		const result = evaluateBreakCompliance(
			[interval(1, '2026-08-21T06:00:00Z', '2026-08-21T06:00:00Z', true)],
			new Date('2026-08-21T12:00:01Z')
		);
		expect(result.map((item) => item.kind)).toEqual([
			'continuous_too_long',
			'insufficient_break'
		]);
	});
});

describe('daysWithBreakViolations', () => {
	it('lists only days that have violations', () => {
		expect(
			daysWithBreakViolations([
				{
					local_date: '2026-08-21',
					elapsed_seconds: 3600,
					intervals: [interval(1, '2026-08-21T06:00:00Z', '2026-08-21T07:00:00Z')]
				},
				{
					local_date: '2026-08-22',
					elapsed_seconds: 8 * 3600,
					intervals: [interval(2, '2026-08-22T06:00:00Z', '2026-08-22T14:00:00Z')]
				}
			]).map((day) => day.local_date)
		).toEqual(['2026-08-22']);
	});
});

describe('continuousViolationIntervalIds', () => {
	it('collects interval ids from continuous violations only', () => {
		const ids = continuousViolationIntervalIds([
			{ kind: 'continuous_too_long', message: '', intervalIds: [1, 2] },
			{ kind: 'break_too_short', message: '', intervalIds: [2, 3] },
			{ kind: 'insufficient_break', message: '', intervalIds: [1, 2, 3] },
			{ kind: 'continuous_too_long', message: '', intervalIds: [4] }
		]);
		expect([...ids].sort((a, b) => a - b)).toEqual([1, 2, 4]);
	});
});

