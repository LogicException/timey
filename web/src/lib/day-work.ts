import { berlinLocalToUtc, isoToBerlinHoursMinutes } from './dates.ts';
import type { WorkInterval } from './types.ts';

export type DayWorkForm = {
	fromH: number;
	fromM: number;
	toH: number;
	toM: number;
};

export type DayWorkPayload = {
	start_at: string;
	end_at?: string;
};

export type DayWorkModalState = {
	open: boolean;
	editing: number | null;
};

export function intervalToForm(interval: WorkInterval): DayWorkForm {
	const start = isoToBerlinHoursMinutes(interval.start_at);
	const end = interval.open ? start : isoToBerlinHoursMinutes(interval.end_at);
	return {
		fromH: start.hours,
		fromM: start.minutes,
		toH: end.hours,
		toM: end.minutes
	};
}

export function saveWorkPayload(day: string, form: DayWorkForm, open: boolean): DayWorkPayload {
	const payload: DayWorkPayload = {
		start_at: berlinLocalToUtc(day, form.fromH, form.fromM).toISOString()
	};
	if (!open) {
		payload.end_at = berlinLocalToUtc(day, form.toH, form.toM).toISOString();
	}
	return payload;
}

export function closeWorkModalState(): DayWorkModalState {
	return { open: false, editing: null };
}
