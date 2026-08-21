import { berlinLocalToUtc, isoToBerlinDate } from './dates.ts';
import type { Entry } from './types.ts';

export type WeekEntryEditState = {
	startIso: string;
	endIso: string;
	taskId: number | null;
	projectId: number | null;
	aufgabeId: number | null;
};

export function entryToEditState(entry: Entry): WeekEntryEditState | null {
	if (!entry.end_at) return null;
	return {
		startIso: entry.start_at,
		endIso: entry.end_at,
		taskId: entry.task_id,
		projectId: entry.project_id,
		aufgabeId: entry.aufgabe_id
	};
}

export function savePayload(state: WeekEntryEditState): {
	task_id: number | null;
	project_id: number | null;
	aufgabe_id: number | null;
	start_at: string;
	end_at: string;
} {
	return {
		task_id: state.taskId,
		project_id: state.projectId,
		aufgabe_id: state.aufgabeId,
		start_at: state.startIso,
		end_at: state.endIso
	};
}

export function applyBerlinTimes(iso: string, hours: number, minutes: number): string {
	return berlinLocalToUtc(isoToBerlinDate(iso), hours, minutes).toISOString();
}
