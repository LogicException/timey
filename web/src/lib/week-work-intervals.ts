import type { WorkDaySummary } from './types.ts';

const WORK_INTERVAL_CLASS = 'work-interval' as const;

export type WeekWorkEvent = {
	id: string;
	start: string;
	end: string;
	display: 'background';
	classNames: [typeof WORK_INTERVAL_CLASS];
};

export function workIntervalEvents(days: ReadonlyArray<WorkDaySummary>): WeekWorkEvent[] {
	return days.flatMap((day) =>
		(day.intervals ?? []).map(
			(interval, index): WeekWorkEvent => ({
				id: `work-${day.local_date}-${index}`,
				start: interval.start_at,
				end: interval.end_at,
				display: 'background',
				classNames: [WORK_INTERVAL_CLASS]
			})
		)
	);
}
