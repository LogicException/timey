import { berlinLocalToUtc, isoToBerlinDate } from './dates.ts';
import type { Entry } from './types.ts';

export type WeekEntryEditState = {
	startIso: string;
	endIso: string;
	taskId: number | null;
	projectId: number | null;
};

export function entryToEditState(entry: Entry): WeekEntryEditState | null {
	if (!entry.end_at) return null;
	return {
		startIso: entry.start_at,
		endIso: entry.end_at,
		taskId: entry.task_id,
		projectId: entry.project_id
	};
}

export function savePayload(state: WeekEntryEditState): {
	task_id: number | null;
	project_id: number | null;
	start_at: string;
	end_at: string;
} {
	return {
		task_id: state.taskId,
		project_id: state.projectId,
		start_at: state.startIso,
		end_at: state.endIso
	};
}

export function applyBerlinTimes(iso: string, hours: number, minutes: number): string {
	return berlinLocalToUtc(isoToBerlinDate(iso), hours, minutes).toISOString();
}
