import { formatHm } from './format.ts';
import type { WorkDaySummary, WorkInterval } from './types.ts';

export type BreakViolationKind = 'continuous_too_long' | 'break_too_short' | 'insufficient_break';

export type BreakViolation = {
	kind: BreakViolationKind;
	message: string;
	intervalIds: number[];
};

const MIN_QUALIFYING_BREAK_SECONDS = 15 * 60;
const MAX_CONTINUOUS_SECONDS = 6 * 60 * 60;
const SIX_HOURS_SECONDS = 6 * 60 * 60;
const NINE_HOURS_SECONDS = 9 * 60 * 60;
const BREAK_AFTER_SIX_HOURS_SECONDS = 30 * 60;
const BREAK_AFTER_NINE_HOURS_SECONDS = 45 * 60;

type ResolvedInterval = {
	id: number;
	startMs: number;
	endMs: number;
};

type Gap = {
	seconds: number;
	leftId: number;
	rightId: number;
};

type ContinuousBlock = {
	intervalIds: number[];
	workSeconds: number;
};

function resolveIntervals(
	intervals: ReadonlyArray<WorkInterval>,
	now?: Date
): ResolvedInterval[] {
	const nowMs = now?.getTime() ?? Date.now();
	return intervals
		.map((interval) => {
			const startMs = Date.parse(interval.start_at);
			const endMs = interval.open ? nowMs : Date.parse(interval.end_at);
			return { id: interval.id, startMs, endMs };
		})
		.filter(
			(interval) =>
				Number.isFinite(interval.startMs) &&
				Number.isFinite(interval.endMs) &&
				interval.endMs > interval.startMs
		)
		.sort((left, right) => left.startMs - right.startMs);
}

function durationSeconds(interval: ResolvedInterval): number {
	return Math.max(0, (interval.endMs - interval.startMs) / 1000);
}

function requiredBreakSeconds(netWorkSeconds: number): number {
	if (netWorkSeconds > NINE_HOURS_SECONDS) {
		return BREAK_AFTER_NINE_HOURS_SECONDS;
	}
	if (netWorkSeconds > SIX_HOURS_SECONDS) {
		return BREAK_AFTER_SIX_HOURS_SECONDS;
	}
	return 0;
}

function gapsBetween(resolved: ReadonlyArray<ResolvedInterval>): Gap[] {
	const gaps: Gap[] = [];
	for (let index = 0; index < resolved.length - 1; index += 1) {
		const left = resolved[index];
		const right = resolved[index + 1];
		if (!left || !right) continue;
		gaps.push({
			seconds: Math.max(0, (right.startMs - left.endMs) / 1000),
			leftId: left.id,
			rightId: right.id
		});
	}
	return gaps;
}

function continuousBlocks(
	resolved: ReadonlyArray<ResolvedInterval>,
	gaps: ReadonlyArray<Gap>
): ContinuousBlock[] {
	const first = resolved[0];
	if (!first) return [];
	const blocks: ContinuousBlock[] = [];
	let current: ContinuousBlock = {
		intervalIds: [first.id],
		workSeconds: durationSeconds(first)
	};
	for (let index = 0; index < gaps.length; index += 1) {
		const gap = gaps[index];
		const next = resolved[index + 1];
		if (!gap || !next) continue;
		if (gap.seconds < MIN_QUALIFYING_BREAK_SECONDS) {
			current.intervalIds.push(next.id);
			current.workSeconds += durationSeconds(next);
		} else {
			blocks.push(current);
			current = {
				intervalIds: [next.id],
				workSeconds: durationSeconds(next)
			};
		}
	}
	blocks.push(current);
	return blocks;
}

export function evaluateBreakCompliance(
	intervals: ReadonlyArray<WorkInterval>,
	now?: Date
): BreakViolation[] {
	const resolved = resolveIntervals(intervals, now);
	if (resolved.length === 0) {
		return [];
	}

	const netWorkSeconds = resolved.reduce((sum, item) => sum + durationSeconds(item), 0);
	const gaps = gapsBetween(resolved);
	const qualifyingSeconds = gaps
		.filter((gap) => gap.seconds >= MIN_QUALIFYING_BREAK_SECONDS)
		.reduce((sum, gap) => sum + gap.seconds, 0);
	const shortGaps = gaps.filter(
		(gap) => gap.seconds > 0 && gap.seconds < MIN_QUALIFYING_BREAK_SECONDS
	);

	const violations: BreakViolation[] = [];
	for (const block of continuousBlocks(resolved, gaps)) {
		if (block.workSeconds > MAX_CONTINUOUS_SECONDS) {
			violations.push({
				kind: 'continuous_too_long',
				message: `Durchgehende Arbeitszeit ${formatHm(block.workSeconds)} ohne Pause (höchstens 6:00)`,
				intervalIds: block.intervalIds
			});
		}
	}

	const shortRelevant =
		netWorkSeconds > SIX_HOURS_SECONDS ||
		violations.some((item) => item.kind === 'continuous_too_long');
	if (shortRelevant) {
		for (const gap of shortGaps) {
			violations.push({
				kind: 'break_too_short',
				message: `Pause ${formatHm(gap.seconds)} zählt nicht (mindestens 0:15)`,
				intervalIds: [gap.leftId, gap.rightId]
			});
		}
	}

	const required = requiredBreakSeconds(netWorkSeconds);
	if (required > 0 && qualifyingSeconds < required) {
		violations.push({
			kind: 'insufficient_break',
			message: `Pausensumme ${formatHm(qualifyingSeconds)}, bei ${formatHm(netWorkSeconds)} Arbeitszeit sind ${formatHm(required)} nötig`,
			intervalIds: resolved.map((item) => item.id)
		});
	}

	return violations;
}

export function continuousViolationIntervalIds(
	violations: ReadonlyArray<BreakViolation>
): Set<number> {
	const ids = new Set<number>();
	for (const violation of violations) {
		if (violation.kind !== 'continuous_too_long') continue;
		for (const id of violation.intervalIds) {
			ids.add(id);
		}
	}
	return ids;
}

export type DayBreakWarnings = {
	local_date: string;
	violations: BreakViolation[];
};

export function daysWithBreakViolations(
	days: ReadonlyArray<WorkDaySummary>,
	now?: Date
): DayBreakWarnings[] {
	const result: DayBreakWarnings[] = [];
	for (const day of days) {
		const violations = evaluateBreakCompliance(day.intervals ?? [], now);
		if (violations.length === 0) continue;
		result.push({ local_date: day.local_date, violations });
	}
	return result;
}
