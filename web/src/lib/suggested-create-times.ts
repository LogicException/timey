import { isoToBerlinHoursMinutes } from './dates.ts';
import { effectiveSlotMinutes } from './slot-minutes.ts';

export type SuggestedCreateClock = {
	fromH: number;
	fromM: number;
	toH: number;
	toM: number;
};

const END_OF_DAY_MINUTES = 23 * 60 + 59;

export function latestStopIso(
	endAts: ReadonlyArray<string | null | undefined>
): string | null {
	let latest: string | null = null;
	let latestMs = Number.NEGATIVE_INFINITY;
	for (const endAt of endAts) {
		if (!endAt) continue;
		const ms = Date.parse(endAt);
		if (Number.isNaN(ms)) continue;
		if (ms >= latestMs) {
			latestMs = ms;
			latest = endAt;
		}
	}
	return latest;
}

export function suggestedCreateClock(
	startIso: string,
	slotMinutes: number | null | undefined
): SuggestedCreateClock {
	const start = isoToBerlinHoursMinutes(startIso);
	const startTotal = start.hours * 60 + start.minutes;
	const duration = effectiveSlotMinutes(slotMinutes);
	const endTotal = Math.min(startTotal + duration, END_OF_DAY_MINUTES);
	return {
		fromH: start.hours,
		fromM: start.minutes,
		toH: Math.floor(endTotal / 60),
		toM: endTotal % 60
	};
}
