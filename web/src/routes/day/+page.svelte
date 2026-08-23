<script lang="ts">
	import { api } from '$lib/api';
	import DateField from '$lib/components/DateField.svelte';
	import NamedSelect from '$lib/components/NamedSelect.svelte';
	import TimeField from '$lib/components/TimeField.svelte';
	import { closeModalState, entryToForm, savePayload } from '$lib/day-entry';
	import { addDays, formatBerlinDate, formatBerlinTime } from '$lib/dates';
	import { durationBetween, formatHm, totalDurationSeconds } from '$lib/format';
	import type { Entry, NamedItem } from '$lib/types';

	let day = $state(formatBerlinDate(new Date()));
	let entries = $state<Entry[]>([]);
	let tasks = $state<NamedItem[]>([]);
	let projects = $state<NamedItem[]>([]);
	let error = $state('');

	let fromH = $state(8);
	let fromM = $state(0);
	let toH = $state(9);
	let toM = $state(0);
	let taskId = $state<number | null>(null);
	let projectId = $state<number | null>(null);
	let editing = $state<number | null>(null);
	let open = $state(false);

	async function load() {
		const [entryRes, taskRes, projectRes] = await Promise.all([
			api<Entry[]>(`/api/entries?from=${day}&to=${day}`),
			api<NamedItem[]>('/api/tasks'),
			api<NamedItem[]>('/api/projects')
		]);
		entries = entryRes;
		tasks = taskRes;
		projects = projectRes;
		if (taskId == null && tasks[0]) taskId = tasks[0].id;
	}

	$effect(() => {
		void day;
		void load().catch((err) => {
			error = err instanceof Error ? err.message : 'Laden fehlgeschlagen';
		});
	});

	function closeModal() {
		const next = closeModalState();
		open = next.open;
		editing = next.editing;
		error = '';
	}

	function openCreate() {
		editing = null;
		error = '';
		open = true;
	}

	async function save() {
		error = '';
		const body = savePayload(day, {
			fromH,
			fromM,
			toH,
			toM,
			taskId,
			projectId
		});
		try {
			if (editing == null) {
				await api('/api/entries', { method: 'POST', body: JSON.stringify(body) });
			} else {
				await api(`/api/entries/${editing}`, { method: 'PATCH', body: JSON.stringify(body) });
			}
			closeModal();
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Speichern fehlgeschlagen';
		}
	}

	function edit(entry: Entry) {
		const form = entryToForm(entry);
		editing = entry.id;
		fromH = form.fromH;
		fromM = form.fromM;
		toH = form.toH;
		toM = form.toM;
		taskId = form.taskId;
		projectId = form.projectId;
		error = '';
		open = true;
	}

	async function remove(id: number) {
		await api(`/api/entries/${id}`, { method: 'DELETE' });
		if (editing === id) closeModal();
		await load();
	}

	async function assignTask(entry: Entry) {
		if (taskId == null) return;
		await api(`/api/entries/${entry.id}`, {
			method: 'PATCH',
			body: JSON.stringify({
				task_id: taskId,
				project_id: projectId,
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
		<button class="ml-auto rounded-md bg-amber px-4 py-2 text-sm font-semibold text-bg" onclick={openCreate}
			>Neuer Eintrag</button
		>
	</div>

	{#if error && !open}
		<p class="text-sm text-stop">{error}</p>
	{/if}

	<div class="panel overflow-hidden rounded-xl">
		<table class="w-full text-sm">
			<thead class="bg-panel-2 text-left text-xs uppercase tracking-wider text-muted">
				<tr>
					<th class="px-4 py-2">Start</th>
					<th class="px-4 py-2">Ende</th>
					<th class="px-4 py-2">Dauer</th>
					<th class="px-4 py-2">Task</th>
					<th class="px-4 py-2">Projekt</th>
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
						<td class="px-4 py-2 text-right">
							<button class="text-xs text-muted" onclick={() => edit(entry)}>Bearbeiten</button>
							<button class="ml-2 text-xs text-stop" onclick={() => remove(entry.id)}>Löschen</button>
						</td>
					</tr>
				{/each}
			</tbody>
			<tfoot>
				<tr class="border-t border-line bg-panel-2">
					<td class="px-4 py-2 text-xs uppercase tracking-wider text-muted" colspan="2">Summe</td>
					<td class="clock-face px-4 py-2">{formatHm(totalDurationSeconds(entries))}</td>
					<td colspan="3"></td>
				</tr>
			</tfoot>
		</table>
	</div>
</div>

{#if open}
	<div class="fixed inset-0 z-30 flex items-center justify-center bg-black/50 p-4">
		<div class="panel w-full max-w-xl space-y-3 rounded-xl p-5">
			<h2 class="text-lg">{editing == null ? 'Neuer Eintrag' : 'Eintrag bearbeiten'}</h2>
			<div class="grid gap-3 sm:grid-cols-2">
				<TimeField bind:hours={fromH} bind:minutes={fromM} label="Von" />
				<TimeField bind:hours={toH} bind:minutes={toM} label="Bis" />
			</div>
			<div class="grid gap-3 sm:grid-cols-2">
				<NamedSelect
					label="Task"
					items={tasks}
					bind:value={taskId}
					currentLabel={entries.find((entry) => entry.id === editing)?.task_name ?? null}
				/>
				<NamedSelect label="Projekt" items={projects} bind:value={projectId} optional />
			</div>
			{#if error}
				<p class="text-sm text-stop">{error}</p>
			{/if}
			<div class="flex justify-end gap-2">
				<button class="px-3 py-2 text-sm text-muted" onclick={closeModal}>Abbrechen</button>
				<button class="rounded-md bg-amber px-4 py-2 text-sm font-semibold text-bg" onclick={save}
					>{editing == null ? 'Erfassen' : 'Speichern'}</button
				>
			</div>
		</div>
	</div>
{/if}
