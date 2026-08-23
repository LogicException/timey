import { berlinLocalToUtc, isoToBerlinHoursMinutes } from './dates.ts';
import type { Entry } from './types.ts';

export type DayEntryForm = {
	fromH: number;
	fromM: number;
	toH: number;
	toM: number;
	taskId: number | null;
	projectId: number | null;
};

export type DayEntryPayload = {
	task_id: number | null;
	project_id: number | null;
	start_at: string;
	end_at: string;
};

export type DayModalState = {
	open: boolean;
	editing: number | null;
};

export function entryToForm(entry: Entry): DayEntryForm {
	const start = isoToBerlinHoursMinutes(entry.start_at);
	const end = entry.end_at ? isoToBerlinHoursMinutes(entry.end_at) : start;
	return {
		fromH: start.hours,
		fromM: start.minutes,
		toH: end.hours,
		toM: end.minutes,
		taskId: entry.task_id,
		projectId: entry.project_id
	};
}

export function savePayload(day: string, form: DayEntryForm): DayEntryPayload {
	return {
		task_id: form.taskId,
		project_id: form.projectId,
		start_at: berlinLocalToUtc(day, form.fromH, form.fromM).toISOString(),
		end_at: berlinLocalToUtc(day, form.toH, form.toM).toISOString()
	};
}

export function closeModalState(): DayModalState {
	return { open: false, editing: null };
}
