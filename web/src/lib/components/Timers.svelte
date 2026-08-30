<script lang="ts">
	import { api } from '$lib/api';
	import { evaluateBreakCompliance } from '$lib/break-compliance';
	import { formatWorkDuration } from '$lib/format';
	import type { Entry, NamedItem, WorkInterval, WorkSnapshot } from '$lib/types';
	import { workAllowsTimer } from '$lib/work-summary';
	import BreakWarnings from './BreakWarnings.svelte';
	import NamedSelect from './NamedSelect.svelte';

	let {
		work,
		workIntervals = [],
		timer,
		tasks,
		projects,
		onRefresh
	}: {
		work: WorkSnapshot | null;
		workIntervals?: WorkInterval[];
		timer: Entry | null;
		tasks: NamedItem[];
		projects: NamedItem[];
		onRefresh: () => Promise<void>;
	} = $props();

	let displaySeconds = $state(0);
	let nowMs = $state(Date.now());
	let taskId = $state<number | null>(null);
	let projectId = $state<number | null>(null);
	let error = $state('');
	let boundTimerId = $state<number | null>(null);

	const breakViolations = $derived(evaluateBreakCompliance(workIntervals, new Date(nowMs)));

	$effect(() => {
		displaySeconds = work?.elapsed_seconds ?? 0;
		nowMs = Date.now();
		if (work?.status !== 'running') return;
		const started = Date.now();
		const base = work.elapsed_seconds;
		const id = setInterval(() => {
			displaySeconds = base + Math.floor((Date.now() - started) / 1000);
			nowMs = Date.now();
		}, 1000);
		return () => clearInterval(id);
	});

	$effect(() => {
		if (timer && timer.id !== boundTimerId) {
			boundTimerId = timer.id;
			taskId = timer.task_id;
			projectId = timer.project_id;
		}
		if (!timer) {
			boundTimerId = null;
		}
	});

	async function call(path: string) {
		error = '';
		try {
			await api(path, { method: 'POST', body: '{}' });
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function startTimer() {
		error = '';
		try {
			await api('/api/entries/timer/start', { method: 'POST', body: '{}' });
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function confirmStop() {
		if (taskId == null) {
			error = 'Task ist erforderlich';
			return;
		}
		error = '';
		try {
			await api('/api/entries/timer/stop', {
				method: 'POST',
				body: JSON.stringify({
					task_id: taskId,
					project_id: projectId
				})
			});
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function cancelTimer() {
		if (!timer) return;
		error = '';
		try {
			await api(`/api/entries/${timer.id}`, { method: 'DELETE' });
			taskId = null;
			projectId = null;
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}
</script>

<div class="panel space-y-3 rounded-xl px-4 py-3">
	<div class="flex flex-wrap items-center gap-4">
		<div>
			<p class="text-[10px] uppercase tracking-[0.22em] text-muted">Arbeitszeit</p>
			<p class="clock-face text-xl">{formatWorkDuration(displaySeconds)}</p>
			<p class="text-xs text-muted">
				{#if work?.status === 'running'}läuft{:else if work?.status === 'paused'}pausiert{:else}bereit{/if}
			</p>
		</div>
		<div class="flex gap-2">
			{#if work?.status !== 'running' && work?.status !== 'paused'}
				<button class="rounded-md bg-go px-3 py-2 text-sm text-bg" onclick={() => call('/api/work-sessions/start')}
					>Start</button
				>
			{/if}
			{#if work?.status === 'running'}
				<button class="rounded-md bg-panel-2 px-3 py-2 text-sm" onclick={() => call('/api/work-sessions/pause')}
					>Pause</button
				>
				<button class="rounded-md bg-stop px-3 py-2 text-sm" onclick={() => call('/api/work-sessions/stop')}
					>Stop</button
				>
			{/if}
			{#if work?.status === 'paused'}
				<button class="rounded-md bg-go px-3 py-2 text-sm text-bg" onclick={() => call('/api/work-sessions/resume')}
					>Weiter</button
				>
				<button class="rounded-md bg-stop px-3 py-2 text-sm" onclick={() => call('/api/work-sessions/stop')}
					>Stop</button
				>
			{/if}
		</div>
		<div class="ml-auto flex items-center gap-2">
			{#if !timer}
				<button
					class="rounded-full bg-go px-4 py-2 text-sm font-semibold text-bg disabled:cursor-not-allowed disabled:opacity-40"
					onclick={startTimer}
					disabled={!workAllowsTimer(work)}>Neuer Eintrag</button
				>
			{/if}
		</div>
	</div>
	{#if timer}
		<div class="border-t border-line pt-3">
			<p class="mb-2 text-xs uppercase tracking-[0.18em] text-muted">Eintrag läuft — Task und Projekt angeben</p>
			<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
				<NamedSelect
					label="Task"
					items={tasks}
					bind:value={taskId}
					currentLabel={timer?.task_name ?? null}
				/>
				<NamedSelect label="Projekt" items={projects} bind:value={projectId} optional />
				<div class="flex items-end gap-2">
					<button class="px-3 py-2 text-sm text-muted" onclick={cancelTimer}>Abbrechen</button>
					<button class="rounded-md bg-stop px-4 py-2 text-sm" onclick={confirmStop}>Stop</button>
				</div>
			</div>
		</div>
	{/if}
</div>
{#if breakViolations.length > 0}
	<div class="mt-2">
		<BreakWarnings violations={breakViolations} compact />
	</div>
{/if}
{#if error}
	<p class="mt-2 text-sm text-stop">{error}</p>
{/if}
