<script lang="ts">
	import { api } from '$lib/api';
	import DateField from '$lib/components/DateField.svelte';
	import NamedSelect from '$lib/components/NamedSelect.svelte';
	import TimeField from '$lib/components/TimeField.svelte';
	import {
		addDays,
		berlinLocalToUtc,
		formatBerlinDate,
		formatBerlinTime,
		isoToBerlinHoursMinutes
	} from '$lib/dates';
	import { durationBetween, formatHm } from '$lib/format';
	import type { Entry, NamedItem } from '$lib/types';

	let day = $state(formatBerlinDate(new Date()));
	let entries = $state<Entry[]>([]);
	let tasks = $state<NamedItem[]>([]);
	let projects = $state<NamedItem[]>([]);
	let aufgaben = $state<NamedItem[]>([]);
	let error = $state('');

	let fromH = $state(8);
	let fromM = $state(0);
	let toH = $state(9);
	let toM = $state(0);
	let taskId = $state<number | null>(null);
	let projectId = $state<number | null>(null);
	let aufgabeId = $state<number | null>(null);
	let editing = $state<number | null>(null);

	async function load() {
		const [entryRes, taskRes, projectRes, aufgabeRes] = await Promise.all([
			api<Entry[]>(`/api/entries?from=${day}&to=${day}`),
			api<NamedItem[]>('/api/tasks'),
			api<NamedItem[]>('/api/projects'),
			api<NamedItem[]>('/api/aufgaben')
		]);
		entries = entryRes;
		tasks = taskRes;
		projects = projectRes;
		aufgaben = aufgabeRes;
		if (taskId == null && tasks[0]) taskId = tasks[0].id;
	}

	$effect(() => {
		void day;
		void load().catch((err) => {
			error = err instanceof Error ? err.message : 'Laden fehlgeschlagen';
		});
	});

	async function save() {
		error = '';
		const start_at = berlinLocalToUtc(day, fromH, fromM).toISOString();
		const end_at = berlinLocalToUtc(day, toH, toM).toISOString();
		try {
			const body = {
				task_id: taskId,
				project_id: projectId,
				aufgabe_id: aufgabeId,
				start_at,
				end_at
			};
			if (editing == null) {
				await api('/api/entries', { method: 'POST', body: JSON.stringify(body) });
			} else {
				await api(`/api/entries/${editing}`, { method: 'PATCH', body: JSON.stringify(body) });
			}
			editing = null;
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Speichern fehlgeschlagen';
		}
	}

	function edit(entry: Entry) {
		editing = entry.id;
		const start = isoToBerlinHoursMinutes(entry.start_at);
		fromH = start.hours;
		fromM = start.minutes;
		if (entry.end_at) {
			const end = isoToBerlinHoursMinutes(entry.end_at);
			toH = end.hours;
			toM = end.minutes;
		}
		taskId = entry.task_id;
		projectId = entry.project_id;
		aufgabeId = entry.aufgabe_id;
	}

	async function remove(id: number) {
		await api(`/api/entries/${id}`, { method: 'DELETE' });
		if (editing === id) editing = null;
		await load();
	}

	async function assignTask(entry: Entry) {
		if (taskId == null) return;
		await api(`/api/entries/${entry.id}`, {
			method: 'PATCH',
			body: JSON.stringify({
				task_id: taskId,
				project_id: projectId,
				aufgabe_id: aufgabeId,
				start_at: entry.start_at,
				end_at: entry.end_at
			})
		});
		await load();
	}
</script>

<div class="space-y-4">
	<div class="flex items-end gap-3">
		<button class="panel rounded-md px-3 py-2" onclick={() => (day = addDays(day, -1))}>←</button>
		<div class="w-56"><DateField bind:value={day} label="Tag" /></div>
		<button class="panel rounded-md px-3 py-2" onclick={() => (day = addDays(day, 1))}>→</button>
		<button class="text-sm text-muted" onclick={() => (day = formatBerlinDate(new Date()))}>Heute</button>
	</div>

	{#if error}
		<p class="text-sm text-stop">{error}</p>
	{/if}

	<div class="panel space-y-3 rounded-xl p-4">
		<h2 class="text-lg">{editing == null ? 'Neuer Eintrag' : 'Eintrag bearbeiten'}</h2>
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<TimeField bind:hours={fromH} bind:minutes={fromM} label="Von" />
			<TimeField bind:hours={toH} bind:minutes={toM} label="Bis" />
			<NamedSelect label="Task" items={tasks} bind:value={taskId} />
			<NamedSelect label="Projekt" items={projects} bind:value={projectId} optional />
			<NamedSelect label="Aufgabe" items={aufgaben} bind:value={aufgabeId} optional />
		</div>
		<div class="flex gap-2">
			<button class="rounded-md bg-amber px-4 py-2 text-sm font-semibold text-bg" onclick={save}
				>{editing == null ? 'Erfassen' : 'Speichern'}</button
			>
			{#if editing != null}
				<button class="text-sm text-muted" onclick={() => (editing = null)}>Abbrechen</button>
			{/if}
		</div>
	</div>

	<div class="panel overflow-hidden rounded-xl">
		<table class="w-full text-sm">
			<thead class="bg-panel-2 text-left text-xs uppercase tracking-wider text-muted">
				<tr>
					<th class="px-4 py-2">Start</th>
					<th class="px-4 py-2">Ende</th>
					<th class="px-4 py-2">Dauer</th>
					<th class="px-4 py-2">Task</th>
					<th class="px-4 py-2">Projekt</th>
					<th class="px-4 py-2">Aufgabe</th>
					<th class="px-4 py-2"></th>
				</tr>
			</thead>
			<tbody>
				{#each entries as entry}
					<tr class="border-t border-line">
						<td class="clock-face px-4 py-2">{formatBerlinTime(new Date(entry.start_at))}</td>
						<td class="clock-face px-4 py-2"
							>{entry.end_at ? formatBerlinTime(new Date(entry.end_at)) : 'läuft'}</td
						>
						<td class="clock-face px-4 py-2">{formatHm(durationBetween(entry.start_at, entry.end_at))}</td>
						<td class="px-4 py-2">
							{#if entry.status === 'needs_task'}
								<span class="text-stop">ohne Task</span>
								<button class="ml-2 text-xs underline" onclick={() => assignTask(entry)}>zuordnen</button>
							{:else}
								{entry.task_name ?? '—'}
							{/if}
						</td>
						<td class="px-4 py-2">{entry.project_name ?? '—'}</td>
						<td class="px-4 py-2">{entry.aufgabe_name ?? '—'}</td>
						<td class="px-4 py-2 text-right">
							<button class="text-xs text-muted" onclick={() => edit(entry)}>Bearbeiten</button>
							<button class="ml-2 text-xs text-stop" onclick={() => remove(entry.id)}>Löschen</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>
