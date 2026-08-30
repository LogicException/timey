import {
	continuousViolationIntervalIds,
	evaluateBreakCompliance
} from './break-compliance.ts';
import type { WorkDaySummary } from './types.ts';

const WORK_INTERVAL_CLASS = 'work-interval' as const;
const WORK_INTERVAL_VIOLATION_CLASS = 'work-interval-violation' as const;

export type WeekWorkEvent = {
	id: string;
	start: string;
	end: string;
	display: 'background';
	classNames:
		| [typeof WORK_INTERVAL_CLASS]
		| [typeof WORK_INTERVAL_CLASS, typeof WORK_INTERVAL_VIOLATION_CLASS];
};

export function workIntervalEvents(
	days: ReadonlyArray<WorkDaySummary>,
	now?: Date
): WeekWorkEvent[] {
	return days.flatMap((day) => {
		const intervals = day.intervals ?? [];
		const violating = continuousViolationIntervalIds(evaluateBreakCompliance(intervals, now));
		return intervals.map((interval, index): WeekWorkEvent => ({
			id: `work-${day.local_date}-${index}`,
			start: interval.start_at,
			end: interval.end_at,
			display: 'background',
			classNames: violating.has(interval.id)
				? [WORK_INTERVAL_CLASS, WORK_INTERVAL_VIOLATION_CLASS]
				: [WORK_INTERVAL_CLASS]
		}));
	});
}
