export const WEEK_VIEW_BUFFER_MINUTES = 30;
export const DEFAULT_WORK_START = '07:30';
export const DEFAULT_WORK_END = '16:15';
const MINUTES_PER_DAY = 24 * 60;

export function splitHm(value: string): { hours: number; minutes: number } {
	const [hoursPart, minutesPart] = value.split(':');
	return { hours: Number(hoursPart), minutes: Number(minutesPart) };
}

export function joinHm(hours: number, minutes: number): string {
	return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}

export function weekSlotTimes(
	workStart: string,
	workEnd: string,
	bufferMinutes = WEEK_VIEW_BUFFER_MINUTES
): { min: string; max: string } {
	const start = parseHm(workStart);
	const end = parseHm(workEnd);
	return {
		min: formatSlotTime(Math.max(0, start - bufferMinutes)),
		max: formatSlotTime(Math.min(MINUTES_PER_DAY, end + bufferMinutes))
	};
}

function parseHm(value: string): number {
	const { hours, minutes } = splitHm(value);
	return hours * 60 + minutes;
}

function formatSlotTime(totalMinutes: number): string {
	const hours = Math.floor(totalMinutes / 60);
	const minutes = totalMinutes % 60;
	return `${joinHm(hours, minutes)}:00`;
}

