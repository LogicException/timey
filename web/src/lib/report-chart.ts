import { durationBetween } from './format.ts';
import type { Entry } from './types.ts';

export type ChartGroupBy = 'task' | 'project' | 'project_task';

export type ReportChartView = 'table' | 'bar' | 'pie';

export type ChartSlice = {
	key: string;
	label: string;
	seconds: number;
};

export type ChartGroup = {
	key: string;
	label: string;
	seconds: number;
	bars: ChartSlice[];
};

export type ChartArcKind = 'arc' | 'full';

export type ChartArc = ChartSlice & {
	percent: number;
	startDeg: number;
	endDeg: number;
	kind: ChartArcKind;
};

export const CHART_COLORS = [
	'#e4b04a',
	'#3f9d6c',
	'#d45b49',
	'#6b8cae',
	'#c47a3a',
	'#7a9e5c',
	'#8b6bb0',
	'#4a9ea0'
] as const;

export function colorForIndex(index: number): string {
	const size = CHART_COLORS.length;
	const normalized = ((index % size) + size) % size;
	return CHART_COLORS[normalized] ?? CHART_COLORS[0];
}

export function sanitizeChartGroupBy(view: ReportChartView, groupBy: ChartGroupBy): ChartGroupBy {
	if (view !== 'bar' && groupBy === 'project_task') {
		return 'task';
	}
	return groupBy;
}

export function groupEntryDurations(
	entries: ReadonlyArray<Entry>,
	groupBy: Exclude<ChartGroupBy, 'project_task'>
): ChartSlice[] {
	const totals = new Map<string, ChartSlice>();
	for (const item of entries) {
		const seconds = durationBetween(item.start_at, item.end_at);
		if (seconds <= 0) continue;
		const grouped = describeGroup(item, groupBy);
		const existing = totals.get(grouped.key);
		if (existing) {
			existing.seconds += seconds;
			continue;
		}
		totals.set(grouped.key, { ...grouped, seconds });
	}
	return [...totals.values()].sort((left, right) => right.seconds - left.seconds);
}

export function groupEntryDurationsByProjectAndTask(
	entries: ReadonlyArray<Entry>
): ChartGroup[] {
	type ProjectBucket = {
		key: string;
		label: string;
		seconds: number;
		bars: Map<string, ChartSlice>;
	};
	const projects = new Map<string, ProjectBucket>();
	for (const item of entries) {
		const seconds = durationBetween(item.start_at, item.end_at);
		if (seconds <= 0) continue;
		const project = describeGroup(item, 'project');
		const task = describeGroup(item, 'task');
		let bucket = projects.get(project.key);
		if (!bucket) {
			bucket = { key: project.key, label: project.label, seconds: 0, bars: new Map() };
			projects.set(project.key, bucket);
		}
		bucket.seconds += seconds;
		const existing = bucket.bars.get(task.key);
		if (existing) {
			existing.seconds += seconds;
			continue;
		}
		bucket.bars.set(task.key, { ...task, seconds });
	}
	return [...projects.values()]
		.map((bucket) => ({
			key: bucket.key,
			label: bucket.label,
			seconds: bucket.seconds,
			bars: [...bucket.bars.values()].sort((left, right) => right.seconds - left.seconds)
		}))
		.sort((left, right) => right.seconds - left.seconds);
}

function describeGroup(
	entry: Entry,
	groupBy: Exclude<ChartGroupBy, 'project_task'>
): { key: string; label: string } {
	if (groupBy === 'task') {
		return {
			key: entry.task_id === null ? 'task:none' : `task:${entry.task_id}`,
			label: entry.task_name ?? 'Ohne Task'
		};
	}
	return {
		key: entry.project_id === null ? 'project:none' : `project:${entry.project_id}`,
		label: entry.project_name ?? 'Ohne Projekt'
	};
}

export function collapseSlices(slices: ReadonlyArray<ChartSlice>, keep: number): ChartSlice[] {
	if (slices.length <= keep) {
		return [...slices];
	}
	const kept = slices.slice(0, keep);
	const leftoverSeconds = slices.slice(keep).reduce((sum, item) => sum + item.seconds, 0);
	if (leftoverSeconds <= 0) {
		return kept;
	}
	return [...kept, { key: 'other', label: 'Sonstige', seconds: leftoverSeconds }];
}

export function slicesToArcs(slices: ReadonlyArray<ChartSlice>): ChartArc[] {
	const total = slices.reduce((sum, item) => sum + item.seconds, 0);
	if (total <= 0) {
		return [];
	}
	if (slices.length === 1) {
		const only = slices[0];
		if (!only) return [];
		return [
			{
				...only,
				percent: 100,
				startDeg: -90,
				endDeg: 270,
				kind: 'full'
			}
		];
	}
	let cursor = -90;
	return slices.map((item) => {
		const ratio = item.seconds / total;
		const startDeg = cursor;
		const endDeg = cursor + ratio * 360;
		cursor = endDeg;
		return {
			...item,
			percent: ratio * 100,
			startDeg,
			endDeg,
			kind: 'arc' as const
		};
	});
}
