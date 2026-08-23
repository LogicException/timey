import { describe, expect, it } from 'vitest';
import {
	collapseSlices,
	colorForIndex,
	groupEntryDurations,
	slicesToArcs,
	type ChartSlice
} from './report-chart.ts';
import type { Entry } from './types.ts';

function entry(overrides: Partial<Entry> = {}): Entry {
	return {
		id: 1,
		task_id: 3,
		project_id: 8,
		task_name: 'E-Mail',
		project_name: 'Efa',
		start_at: '2026-08-21T12:00:00Z',
		end_at: '2026-08-21T12:30:00Z',
		status: 'complete',
		...overrides
	};
}

function slice(overrides: Partial<ChartSlice> & Pick<ChartSlice, 'key' | 'seconds'>): ChartSlice {
	return {
		label: overrides.key,
		...overrides
	};
}

describe('groupEntryDurations', () => {
	it('groups completed entries by task and sorts by duration descending', () => {
		expect(
			groupEntryDurations(
				[
					entry({
						id: 1,
						task_id: 3,
						task_name: 'E-Mail',
						start_at: '2026-08-21T12:00:00Z',
						end_at: '2026-08-21T12:30:00Z'
					}),
					entry({
						id: 2,
						task_id: 4,
						task_name: 'Review',
						start_at: '2026-08-21T13:00:00Z',
						end_at: '2026-08-21T14:00:00Z'
					}),
					entry({
						id: 3,
						task_id: 3,
						task_name: 'E-Mail',
						start_at: '2026-08-21T15:00:00Z',
						end_at: '2026-08-21T15:15:00Z'
					})
				],
				'task'
			)
		).toEqual([
			{ key: 'task:4', label: 'Review', seconds: 60 * 60 },
			{ key: 'task:3', label: 'E-Mail', seconds: 45 * 60 }
		]);
	});

	it('groups by project independently of task', () => {
		expect(
			groupEntryDurations(
				[
					entry({
						id: 1,
						task_id: 3,
						task_name: 'E-Mail',
						project_id: 8,
						project_name: 'Efa',
						start_at: '2026-08-21T12:00:00Z',
						end_at: '2026-08-21T13:00:00Z'
					}),
					entry({
						id: 2,
						task_id: 4,
						task_name: 'Review',
						project_id: 8,
						project_name: 'Efa',
						start_at: '2026-08-21T13:00:00Z',
						end_at: '2026-08-21T13:30:00Z'
					}),
					entry({
						id: 3,
						task_id: 4,
						task_name: 'Review',
						project_id: 9,
						project_name: 'Intern',
						start_at: '2026-08-21T14:00:00Z',
						end_at: '2026-08-21T14:10:00Z'
					})
				],
				'project'
			)
		).toEqual([
			{ key: 'project:8', label: 'Efa', seconds: 90 * 60 },
			{ key: 'project:9', label: 'Intern', seconds: 10 * 60 }
		]);
	});

	it('uses fallback labels for missing task or project names', () => {
		expect(
			groupEntryDurations(
				[
					entry({
						id: 1,
						task_id: null,
						task_name: null,
						start_at: '2026-08-21T12:00:00Z',
						end_at: '2026-08-21T12:20:00Z'
					})
				],
				'task'
			)
		).toEqual([{ key: 'task:none', label: 'Ohne Task', seconds: 20 * 60 }]);

		expect(
			groupEntryDurations(
				[
					entry({
						id: 2,
						project_id: null,
						project_name: null,
						start_at: '2026-08-21T12:00:00Z',
						end_at: '2026-08-21T12:10:00Z'
					})
				],
				'project'
			)
		).toEqual([{ key: 'project:none', label: 'Ohne Projekt', seconds: 10 * 60 }]);
	});

	it('ignores running entries and zero-duration groups', () => {
		expect(
			groupEntryDurations(
				[
					entry({
						id: 1,
						task_id: 3,
						end_at: null,
						status: 'running'
					}),
					entry({
						id: 2,
						task_id: 4,
						task_name: 'Review',
						start_at: '2026-08-21T13:00:00Z',
						end_at: '2026-08-21T13:30:00Z'
					})
				],
				'task'
			)
		).toEqual([{ key: 'task:4', label: 'Review', seconds: 30 * 60 }]);
	});

	it('returns an empty list when nothing is completed', () => {
		expect(groupEntryDurations([], 'task')).toEqual([]);
		expect(
			groupEntryDurations(
				[entry({ id: 1, end_at: null, status: 'running' })],
				'project'
			)
		).toEqual([]);
	});
});

describe('collapseSlices', () => {
	it('keeps all slices when at or below the limit', () => {
		const slices = [
			slice({ key: 'a', seconds: 30 }),
			slice({ key: 'b', seconds: 20 })
		];
		expect(collapseSlices(slices, 8)).toEqual(slices);
	});

	it('collapses leftover slices into Sonstige', () => {
		const slices = Array.from({ length: 10 }, (_, index) =>
			slice({ key: `s${index}`, label: `Slice ${index}`, seconds: 10 - index })
		);
		expect(collapseSlices(slices, 8)).toEqual([
			...slices.slice(0, 8),
			{ key: 'other', label: 'Sonstige', seconds: 3 }
		]);
	});

	it('omits Sonstige when leftover duration is zero', () => {
		const slices = [
			...Array.from({ length: 8 }, (_, index) => slice({ key: `s${index}`, seconds: 5 })),
			slice({ key: 'zero', seconds: 0 })
		];
		expect(collapseSlices(slices, 8)).toEqual(slices.slice(0, 8));
	});
});

describe('slicesToArcs', () => {
	it('returns an empty list for empty input', () => {
		expect(slicesToArcs([])).toEqual([]);
	});

	it('assigns percent and sweep from -90 degrees for a half and quarter split', () => {
		const arcs = slicesToArcs([
			slice({ key: 'half', label: 'Half', seconds: 30 }),
			slice({ key: 'quarter-a', label: 'A', seconds: 15 }),
			slice({ key: 'quarter-b', label: 'B', seconds: 15 })
		]);
		expect(arcs.map((arc) => ({ key: arc.key, percent: arc.percent }))).toEqual([
			{ key: 'half', percent: 50 },
			{ key: 'quarter-a', percent: 25 },
			{ key: 'quarter-b', percent: 25 }
		]);
		expect(arcs[0]?.startDeg).toBe(-90);
		expect(arcs[0]?.endDeg).toBe(90);
		expect(arcs[1]?.startDeg).toBe(90);
		expect(arcs[1]?.endDeg).toBe(180);
		expect(arcs[2]?.startDeg).toBe(180);
		expect(arcs[2]?.endDeg).toBe(270);
		expect(arcs.every((arc) => arc.kind === 'arc')).toBe(true);
	});

	it('marks a single full slice as a complete circle', () => {
		expect(slicesToArcs([slice({ key: 'only', label: 'Only', seconds: 60 })])).toEqual([
			{
				key: 'only',
				label: 'Only',
				seconds: 60,
				percent: 100,
				startDeg: -90,
				endDeg: 270,
				kind: 'full'
			}
		]);
	});
});

describe('colorForIndex', () => {
	it('cycles through the palette', () => {
		expect(colorForIndex(0)).toBe(colorForIndex(0));
		expect(colorForIndex(0)).not.toBe(colorForIndex(1));
		expect(colorForIndex(8)).toBe(colorForIndex(0));
	});
});
