<script lang="ts">
	import { api } from '$lib/api';
	import BarChart from '$lib/components/BarChart.svelte';
	import DateField from '$lib/components/DateField.svelte';
	import MultiSelect from '$lib/components/MultiSelect.svelte';
	import PieChart from '$lib/components/PieChart.svelte';
	import {
		formatBerlinDate,
		formatBerlinTime,
		PRESET_LABELS,
		rangeForPreset,
		showsManualDateFields,
		type RangePreset
	} from '$lib/dates';
	import { durationBetween, formatHm, totalDurationSeconds } from '$lib/format';
	import { collapseSlices, groupEntryDurations, type ChartGroupBy } from '$lib/report-chart';
	import type { Entry, NamedItem } from '$lib/types';

	type ReportView = 'table' | 'bar' | 'pie';

	const VIEW_LABELS: Record<ReportView, string> = {
		table: 'Tabelle',
		bar: 'Balken',
		pie: 'Torte'
	};

	const GROUP_LABELS: Record<ChartGroupBy, string> = {
		task: 'Task',
		project: 'Projekt'
	};

	const today = formatBerlinDate(new Date());
	let preset = $state<RangePreset>('today');
	let from = $state(today);
	let to = $state(today);
	let entries = $state<Entry[]>([]);
	let tasks = $state<NamedItem[]>([]);
	let projects = $state<NamedItem[]>([]);
	let selectedTasks = $state<number[]>([]);
	let selectedProjects = $state<number[]>([]);
	let error = $state('');
	let view = $state<ReportView>('table');
	let groupBy = $state<ChartGroupBy>('task');

	const slices = $derived(groupEntryDurations(entries, groupBy));
	const pieSlices = $derived(collapseSlices(slices, 8));

	function applyPreset(next: RangePreset) {
		preset = next;
		if (next !== 'custom') {
			const range = rangeForPreset(next, formatBerlinDate(new Date()));
			from = range.from;
			to = range.to;
		}
	}

	async function load() {
		const params = new URLSearchParams({ from, to });
		if (selectedTasks.length) params.set('task_ids', selectedTasks.join(','));
		if (selectedProjects.length) params.set('project_ids', selectedProjects.join(','));
		entries = await api(`/api/entries?${params}`);
	}

	$effect(() => {
		void Promise.all([
			api<NamedItem[]>('/api/tasks?include_archived=true&include_system=true'),
			api<NamedItem[]>('/api/projects?include_archived=true')
		]).then(([t, p]) => {
			tasks = t;
			projects = p;
		});
	});

	$effect(() => {
		void from;
		void to;
		void selectedTasks;
		void selectedProjects;
		void load().catch((err) => {
			error = err instanceof Error ? err.message : 'Laden fehlgeschlagen';
		});
	});

	function exportCsv() {
		const params = new URLSearchParams({ from, to });
		if (selectedTasks.length) params.set('task_ids', selectedTasks.join(','));
		if (selectedProjects.length) params.set('project_ids', selectedProjects.join(','));
		window.location.href = `/api/entries/export.csv?${params}`;
	}
</script>

<div class="space-y-4">
	<div class="flex flex-wrap gap-2">
		{#each Object.entries(PRESET_LABELS) as [key, label]}
			<button
				class="rounded-full px-3 py-1 text-sm {preset === key ? 'bg-amber text-bg' : 'panel'}"
				onclick={() => applyPreset(key as RangePreset)}>{label}</button
			>
		{/each}
	</div>
	<div class="flex flex-wrap gap-4">
		{#if showsManualDateFields(preset)}
			<div class="w-52"><DateField bind:value={from} label="Von" /></div>
			<div class="w-52"><DateField bind:value={to} label="Bis" /></div>
		{/if}
		<button class="self-end rounded-md bg-amber px-4 py-2 text-sm text-bg" onclick={exportCsv}>CSV exportieren</button>
	</div>
	<div class="grid gap-4 md:grid-cols-2">
		<MultiSelect label="Tasks" items={tasks} bind:value={selectedTasks} placeholder="Task suchen …" />
		<MultiSelect
			label="Projekte"
			items={projects}
			bind:value={selectedProjects}
			placeholder="Projekt suchen …"
		/>
	</div>
	{#if error}
		<p class="text-sm text-stop">{error}</p>
	{/if}
	<div class="flex flex-wrap items-center justify-between gap-4">
		<div class="flex flex-wrap gap-2">
			{#each Object.entries(VIEW_LABELS) as [key, label]}
				<button
					class="rounded-full px-3 py-1 text-sm {view === key ? 'bg-amber text-bg' : 'panel'}"
					onclick={() => (view = key as ReportView)}>{label}</button
				>
			{/each}
		</div>
		{#if view !== 'table'}
			<div class="flex flex-wrap items-center gap-2">
				<span class="text-xs uppercase tracking-wider text-muted">Gruppierung</span>
				{#each Object.entries(GROUP_LABELS) as [key, label]}
					<button
						class="rounded-full px-3 py-1 text-sm {groupBy === key ? 'bg-amber text-bg' : 'panel'}"
						onclick={() => (groupBy = key as ChartGroupBy)}>{label}</button
					>
				{/each}
			</div>
		{/if}
	</div>
	{#if view === 'table'}
	<div class="panel overflow-hidden rounded-xl">
		<table class="w-full text-sm">
			<thead class="bg-panel-2 text-left text-xs uppercase tracking-wider text-muted">
				<tr>
					<th class="px-4 py-2">Start</th>
					<th class="px-4 py-2">Ende</th>
					<th class="px-4 py-2">Dauer</th>
					<th class="px-4 py-2">Task</th>
					<th class="px-4 py-2">Projekt</th>
				</tr>
			</thead>
			<tbody>
				{#each entries as entry}
					<tr class="border-t border-line">
						<td class="clock-face px-4 py-2"
							>{formatBerlinDate(new Date(entry.start_at))} {formatBerlinTime(new Date(entry.start_at))}</td
						>
						<td class="clock-face px-4 py-2"
							>{entry.end_at
								? `${formatBerlinDate(new Date(entry.end_at))} ${formatBerlinTime(new Date(entry.end_at))}`
								: 'läuft'}</td
						>
						<td class="clock-face px-4 py-2">{formatHm(durationBetween(entry.start_at, entry.end_at))}</td>
						<td class="px-4 py-2">{entry.task_name ?? '—'}</td>
						<td class="px-4 py-2">{entry.project_name ?? '—'}</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t border-line bg-panel-2">
					<td class="px-4 py-2 text-xs uppercase tracking-wider text-muted" colspan="2">Summe</td>
					<td class="clock-face px-4 py-2">{formatHm(totalDurationSeconds(entries))}</td>
					<td colspan="2"></td>
				</tr>
			</tfoot>
		</table>
	</div>
	{:else if slices.length === 0}
		<p class="text-sm text-muted">Keine abgeschlossenen Einträge</p>
	{:else if view === 'bar'}
		<div class="panel overflow-hidden rounded-xl">
			<BarChart {slices} />
		</div>
	{:else}
		<div class="panel overflow-hidden rounded-xl">
			<PieChart slices={pieSlices} />
		</div>
	{/if}
</div>
