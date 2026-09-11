<script lang="ts">
	import { api } from '$lib/api';
	import { evaluateBreakCompliance } from '$lib/break-compliance';
	import { suggestedTaskId } from '$lib/default-task';
	import { formatHms } from '$lib/format';
	import type { Entry, NamedItem, WorkInterval, WorkSnapshot } from '$lib/types';
	import {
		taskTimerBarText,
		taskTimerCanStart,
		taskTimerStopError,
		timerAssignmentPayload
	} from '$lib/work-summary';
	import type { WorkTransportAction } from '$lib/work-transport';
	import {
		liveElapsedSeconds,
		workSessionPath,
		workTransportBarVisible
	} from '$lib/work-transport';
	import BreakWarningMark from './BreakWarningMark.svelte';
	import NamedSelect from './NamedSelect.svelte';
	import TransportControls from './TransportControls.svelte';

	let {
		work,
		workIntervals = [],
		timer,
		tasks,
		projects,
		defaultTaskId = null,
		onRefresh
	}: {
		work: WorkSnapshot | null;
		workIntervals?: WorkInterval[];
		timer: Entry | null;
		tasks: NamedItem[];
		projects: NamedItem[];
		defaultTaskId?: number | null;
		onRefresh: () => Promise<void>;
	} = $props();

	let displaySeconds = $state(0);
	let nowMs = $state(Date.now());
	let error = $state('');
	let assignmentOpen = $state(false);
	let draftTaskId = $state<number | null>(null);
	let draftProjectId = $state<number | null>(null);
	let modalError = $state('');

	const visible = $derived(workTransportBarVisible(work?.status ?? null));
	const canStartTask = $derived(taskTimerCanStart(work, timer));
	const stopError = $derived(taskTimerStopError(timer?.task_id ?? null));
	const breakViolations = $derived(evaluateBreakCompliance(workIntervals, new Date(nowMs)));

	$effect(() => {
		const base = work?.elapsed_seconds ?? 0;
		const status = work?.status ?? null;
		const origin = Date.now();
		nowMs = Date.now();
		displaySeconds = liveElapsedSeconds(base, status, origin, nowMs);
		const openInterval = workIntervals.some((interval) => interval.open);
		if (status !== 'running' && !openInterval) return;
		const id = setInterval(() => {
			nowMs = Date.now();
			displaySeconds = liveElapsedSeconds(base, status, origin, nowMs);
		}, 1000);
		return () => clearInterval(id);
	});

	async function onAction(action: WorkTransportAction) {
		error = '';
		try {
			await api(workSessionPath(action), { method: 'POST', body: '{}' });
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function startTaskTimer() {
		error = '';
		try {
			const started = await api<Entry>('/api/entries/timer/start', { method: 'POST', body: '{}' });
			await onRefresh();
			openAssignment(started);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	function openAssignment(entry: Entry) {
		draftTaskId = entry.task_id ?? suggestedTaskId(tasks, defaultTaskId, 'none');
		draftProjectId = entry.project_id;
		modalError = '';
		assignmentOpen = true;
	}

	function closeAssignment() {
		assignmentOpen = false;
		modalError = '';
	}

	async function saveAssignment() {
		if (!timer) return;
		const result = timerAssignmentPayload(timer, draftTaskId, draftProjectId);
		if (!result.ok) {
			modalError = result.error;
			return;
		}
		modalError = '';
		try {
			await api(`/api/entries/${timer.id}`, {
				method: 'PATCH',
				body: JSON.stringify(result.body)
			});
			await onRefresh();
			closeAssignment();
		} catch (err) {
			modalError = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function cancelTaskTimer() {
		if (!timer) return;
		error = '';
		try {
			await api(`/api/entries/${timer.id}`, { method: 'DELETE' });
			closeAssignment();
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function stopTaskTimer() {
		const message = taskTimerStopError(timer?.task_id ?? null);
		if (message) {
			error = message;
			if (timer) openAssignment(timer);
			return;
		}
		if (!timer?.task_id) return;
		error = '';
		try {
			await api('/api/entries/timer/stop', {
				method: 'POST',
				body: JSON.stringify({
					task_id: timer.task_id,
					project_id: timer.project_id
				})
			});
			await onRefresh();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}
</script>

{#if visible}
	<div class="sticky top-0 z-50 border-b border-line bg-panel">
		<div class="flex h-9 items-center justify-between gap-4 px-4">
			<div class="flex items-center gap-4">
				<BreakWarningMark violations={breakViolations} tooltipId="work-bar-break-warning">
					<p class="text-[10px] uppercase tracking-[0.22em] text-muted">Arbeitszeit</p>
				</BreakWarningMark>
				<p class="clock-face text-sm">{formatHms(displaySeconds)}</p>
				<TransportControls status={work?.status ?? null} compact onAction={onAction} />
			</div>
			<div class="flex min-w-0 items-center gap-3">
				{#if timer}
					<button
						type="button"
						class="flex min-w-0 items-center gap-3"
						aria-label="Task und Projekt ändern"
						title={taskTimerBarText(timer)}
						onclick={() => {
							if (timer) openAssignment(timer);
						}}
					>
						<span class="text-[10px] uppercase tracking-[0.22em] text-muted">Task</span>
						<span class="min-w-0 truncate text-left text-xs {timer.task_name ? 'text-ink' : 'text-muted'}"
							>{taskTimerBarText(timer)}</span
						>
					</button>
					<button
						type="button"
						class="grid h-8 w-8 shrink-0 place-items-center rounded-md bg-panel-2"
						aria-label="Abbrechen"
						title="Abbrechen"
						onclick={cancelTaskTimer}
					>
						<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
							<path
								d="M6 6l12 12M18 6L6 18"
								fill="none"
								stroke="currentColor"
								stroke-width="2.5"
								stroke-linecap="round"
							/>
						</svg>
					</button>
					<button
						type="button"
						class="grid h-8 w-8 shrink-0 place-items-center rounded-md bg-stop disabled:cursor-not-allowed disabled:opacity-40"
						aria-label="Stoppen"
						title={stopError ?? 'Stoppen'}
						disabled={stopError != null}
						onclick={stopTaskTimer}
					>
						<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
							<rect x="5" y="5" width="14" height="14" fill="currentColor" />
						</svg>
					</button>
				{:else}
					<p class="text-[10px] uppercase tracking-[0.22em] text-muted">Task</p>
					<button
						type="button"
						class="grid h-8 w-8 place-items-center rounded-md bg-go text-bg disabled:cursor-not-allowed disabled:opacity-40"
						aria-label="Neuer Eintrag"
						title="Neuer Eintrag"
						disabled={!canStartTask}
						onclick={startTaskTimer}
					>
						<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
							<polygon points="6,4 20,12 6,20" fill="currentColor" />
						</svg>
					</button>
				{/if}
			</div>
		</div>
		{#if error}
			<p class="pb-1 text-center text-xs text-stop">{error}</p>
		{/if}
	</div>
{/if}

{#if assignmentOpen && timer}
	<div class="fixed inset-0 z-[70] flex items-center justify-center bg-black/50 p-4">
		<div class="panel w-full max-w-xl space-y-3 rounded-xl p-5">
			<h2 class="text-lg">Task und Projekt</h2>
			<div class="grid gap-3 sm:grid-cols-2">
				<NamedSelect
					label="Task"
					items={tasks}
					bind:value={draftTaskId}
					currentLabel={timer.task_name}
				/>
				<NamedSelect label="Projekt" items={projects} bind:value={draftProjectId} optional />
			</div>
			{#if modalError}
				<p class="text-sm text-stop">{modalError}</p>
			{/if}
			<div class="flex justify-end gap-2">
				<button class="px-3 py-2 text-sm text-muted" onclick={closeAssignment}>Abbrechen</button>
				<button class="rounded-md bg-amber px-4 py-2 text-sm font-semibold text-bg" onclick={saveAssignment}
					>Speichern</button
				>
			</div>
		</div>
	</div>
{/if}
