<script lang="ts">
	import { onMount } from 'svelte';
	import { Calendar } from '@fullcalendar/core';
	import timeGridPlugin from '@fullcalendar/timegrid';
	import interactionPlugin from '@fullcalendar/interaction';
	import deLocale from '@fullcalendar/core/locales/de';
	import { api } from '$lib/api';
	import { daysWithBreakViolations, type DayBreakWarnings } from '$lib/break-compliance';
	import BreakWarnings from '$lib/components/BreakWarnings.svelte';
	import NamedSelect from '$lib/components/NamedSelect.svelte';
	import TimeField from '$lib/components/TimeField.svelte';
	import {
		endOfWeek,
		formatBerlinDate,
		isoToBerlinDate,
		isoToBerlinHoursMinutes,
		startOfWeek
	} from '$lib/dates';
	import { applyBerlinTimes, entryToEditState, savePayload } from '$lib/week-entry';
	import { weekTimeGridLayout } from '$lib/week-calendar';
	import { workIntervalEvents } from '$lib/week-work-intervals';
	import type { Entry, NamedItem, UserSettings, WorkDaySummary } from '$lib/types';
	import { DEFAULT_WORK_END, DEFAULT_WORK_START, weekSlotTimes } from '$lib/working-hours';

	let el: HTMLDivElement;
	let calendar: Calendar | undefined;
	let entriesById = new Map<number, Entry>();
	let tasks = $state<NamedItem[]>([]);
	let projects = $state<NamedItem[]>([]);
	let error = $state('');
	let open = $state(false);
	let editingId = $state<number | null>(null);
	let startIso = $state('');
	let endIso = $state('');
	let fromH = $state(8);
	let fromM = $state(0);
	let toH = $state(9);
	let toM = $state(0);
	let taskId = $state<number | null>(null);
	let projectId = $state<number | null>(null);
	let breakWarningDays = $state<DayBreakWarnings[]>([]);

	async function loadRange(from: string, to: string): Promise<Entry[]> {
		return api(`/api/entries?from=${from}&to=${to}`);
	}

	async function loadWorkDays(from: string, to: string): Promise<WorkDaySummary[]> {
		return api(`/api/work-sessions?from=${from}&to=${to}`);
	}

	function eventTitle(entry: Entry): string {
		return [entry.task_name ?? 'ohne Task', entry.project_name].filter(Boolean).join(' · ');
	}

	function syncTimeFields() {
		const from = isoToBerlinHoursMinutes(startIso);
		const to = isoToBerlinHoursMinutes(endIso);
		fromH = from.hours;
		fromM = from.minutes;
		toH = to.hours;
		toM = to.minutes;
	}

	function closeModal() {
		open = false;
		editingId = null;
		error = '';
	}

	function openEdit(id: number) {
		const entry = entriesById.get(id);
		if (!entry) return;
		const state = entryToEditState(entry);
		if (!state) return;
		startIso = state.startIso;
		endIso = state.endIso;
		taskId = state.taskId;
		projectId = state.projectId;
		syncTimeFields();
		editingId = id;
		error = '';
		open = true;
	}

	async function refreshEvents() {
		if (!calendar) return;
		const start = formatBerlinDate(calendar.view.activeStart);
		const end = formatBerlinDate(new Date(calendar.view.activeEnd.getTime() - 1));
		const from = startOfWeek(start);
		const to = endOfWeek(end);
		const [entries, workDays] = await Promise.all([loadRange(from, to), loadWorkDays(from, to)]);
		entriesById = new Map();
		breakWarningDays = daysWithBreakViolations(workDays);
		calendar.removeAllEvents();
		for (const entry of entries) {
			if (!entry.end_at) continue;
			entriesById.set(entry.id, entry);
			calendar.addEvent({
				id: String(entry.id),
				title: eventTitle(entry),
				start: entry.start_at,
				end: entry.end_at
			});
		}
		for (const event of workIntervalEvents(workDays)) {
			calendar.addEvent(event);
		}
	}

	onMount(() => {
		let destroyed = false;
		void (async () => {
			let workStart = DEFAULT_WORK_START;
			let workEnd = DEFAULT_WORK_END;
			try {
				const settings = await api<UserSettings>('/api/settings');
				workStart = settings.work_start;
				workEnd = settings.work_end;
			} catch {
				// keep defaults
			}
			if (destroyed) return;
			const slots = weekSlotTimes(workStart, workEnd);
			calendar = new Calendar(el, {
				plugins: [timeGridPlugin, interactionPlugin],
				initialView: 'timeGridWeek',
				locale: deLocale,
				firstDay: 1,
				allDaySlot: false,
				slotMinTime: slots.min,
				slotMaxTime: slots.max,
				slotDuration: '00:15:00',
				snapDuration: '00:15:00',
				...weekTimeGridLayout(),
				selectable: true,
				selectMirror: true,
				nowIndicator: true,
				height: 'auto',
				headerToolbar: { left: 'prev,next today', center: 'title', right: '' },
				select: (info) => {
					startIso = info.start.toISOString();
					endIso = info.end.toISOString();
					taskId = tasks[0]?.id ?? null;
					projectId = null;
					editingId = null;
					syncTimeFields();
					error = '';
					open = true;
					calendar?.unselect();
				},
				eventDidMount: (info) => {
					if (info.event.display === 'background') {
						info.el.style.pointerEvents = 'none';
						return;
					}
					const entryId = Number(info.event.id);
					if (!entriesById.has(entryId)) return;
					info.el.addEventListener('dblclick', () => {
						openEdit(entryId);
					});
				},
				datesSet: () => {
					void refreshEvents();
				}
			});
			if (destroyed) {
				calendar.destroy();
				return;
			}
			calendar.render();
			void Promise.all([
				api<NamedItem[]>('/api/tasks'),
				api<NamedItem[]>('/api/projects')
			]).then(([t, p]) => {
				tasks = t;
				projects = p;
				taskId = t[0]?.id ?? null;
			});
		})();
		return () => {
			destroyed = true;
			calendar?.destroy();
		};
	});

	async function saveEntry() {
		if (taskId == null) {
			error = 'Task ist erforderlich';
			return;
		}
		error = '';
		startIso = applyBerlinTimes(startIso, fromH, fromM);
		endIso = applyBerlinTimes(endIso, toH, toM);
		const body = savePayload({
			startIso,
			endIso,
			taskId,
			projectId
		});
		try {
			if (editingId == null) {
				await api('/api/entries', {
					method: 'POST',
					body: JSON.stringify(body)
				});
			} else {
				await api(`/api/entries/${editingId}`, {
					method: 'PATCH',
					body: JSON.stringify(body)
				});
			}
			closeModal();
			await refreshEvents();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Speichern fehlgeschlagen';
		}
	}

	function spanLabel(): string {
		if (!startIso) return '';
		return isoToBerlinDate(startIso);
	}
</script>

<div class="space-y-3">
	{#if breakWarningDays.length > 0}
		<div class="panel space-y-3 rounded-xl p-4">
			{#each breakWarningDays as day (day.local_date)}
				<div>
					<p class="mb-1 text-xs uppercase tracking-wider text-muted">{day.local_date}</p>
					<BreakWarnings violations={day.violations} compact />
				</div>
			{/each}
		</div>
	{/if}
	<div class="panel rounded-xl p-3">
		<div bind:this={el}></div>
	</div>
</div>

{#if open}
	<div class="fixed inset-0 z-30 flex items-center justify-center bg-black/50 p-4">
		<div class="panel w-full max-w-xl space-y-3 rounded-xl p-5">
			<h2 class="text-lg">{editingId == null ? 'Zeitspanne erfassen' : 'Eintrag bearbeiten'}</h2>
			<p class="clock-face text-sm text-amber">{spanLabel()}</p>
			<div class="grid gap-3 sm:grid-cols-2">
				<TimeField bind:hours={fromH} bind:minutes={fromM} label="Von" />
				<TimeField bind:hours={toH} bind:minutes={toM} label="Bis" />
			</div>
			<NamedSelect
				label="Task"
				items={tasks}
				bind:value={taskId}
				currentLabel={editingId == null ? null : (entriesById.get(editingId)?.task_name ?? null)}
			/>
			<NamedSelect label="Projekt" items={projects} bind:value={projectId} optional />
			{#if error}
				<p class="text-sm text-stop">{error}</p>
			{/if}
			<div class="flex justify-end gap-2">
				<button class="px-3 py-2 text-sm text-muted" onclick={closeModal}>Abbrechen</button>
				<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg" onclick={saveEntry}>Speichern</button>
			</div>
		</div>
	</div>
{/if}
