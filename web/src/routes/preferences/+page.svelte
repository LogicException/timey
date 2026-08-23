<script lang="ts">
	import { api } from '$lib/api';
	import TimeField from '$lib/components/TimeField.svelte';
	import type { DefaultView } from '$lib/default-view';
	import type { UserSettings } from '$lib/types';
	import { DEFAULT_WORK_END, DEFAULT_WORK_START, joinHm, splitHm } from '$lib/working-hours';

	let startH = $state(7);
	let startM = $state(30);
	let endH = $state(16);
	let endM = $state(15);
	let defaultView = $state<DefaultView>('day');
	let error = $state('');
	let saved = $state(false);

	function apply(settings: UserSettings) {
		const start = splitHm(settings.work_start);
		const end = splitHm(settings.work_end);
		startH = start.hours;
		startM = start.minutes;
		endH = end.hours;
		endM = end.minutes;
		defaultView = settings.default_view;
	}

	async function load() {
		const settings = await api<UserSettings>('/api/settings');
		apply(settings);
	}

	$effect(() => {
		void load().catch((err) => {
			error = err instanceof Error ? err.message : 'Laden fehlgeschlagen';
			apply({
				work_start: DEFAULT_WORK_START,
				work_end: DEFAULT_WORK_END,
				default_view: 'day'
			});
		});
	});

	async function save() {
		error = '';
		saved = false;
		try {
			const settings = await api<UserSettings>('/api/settings', {
				method: 'PATCH',
				body: JSON.stringify({
					work_start: joinHm(startH, startM),
					work_end: joinHm(endH, endM),
					default_view: defaultView
				})
			});
			apply(settings);
			saved = true;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Speichern fehlgeschlagen';
		}
	}
</script>

<section class="panel max-w-xl space-y-4 rounded-xl p-4">
	<h2 class="text-lg">Einstellungen</h2>
	<form
		class="space-y-6"
		onsubmit={(event) => {
			event.preventDefault();
			void save();
		}}
	>
		<div class="space-y-3">
			<h3 class="text-base">Übliche Arbeitszeit</h3>
			<p class="text-sm text-muted">
				Die Wochenansicht beginnt 30 Minuten vor Arbeitsbeginn und endet 30 Minuten nach Arbeitsende.
			</p>
			<div class="grid gap-3 sm:grid-cols-2">
				<TimeField bind:hours={startH} bind:minutes={startM} label="Beginn" />
				<TimeField bind:hours={endH} bind:minutes={endM} label="Ende" />
			</div>
		</div>
		<fieldset class="space-y-3">
			<legend class="text-base">Startansicht</legend>
			<p class="text-sm text-muted">Nach dem Öffnen der App zuerst diese Ansicht zeigen.</p>
			<label class="flex items-center gap-2 text-sm">
				<input type="radio" name="default_view" value="day" bind:group={defaultView} />
				Tagesansicht
			</label>
			<label class="flex items-center gap-2 text-sm">
				<input type="radio" name="default_view" value="week" bind:group={defaultView} />
				Wochenansicht
			</label>
		</fieldset>
		{#if error}
			<p class="text-sm text-stop">{error}</p>
		{/if}
		{#if saved}
			<p class="text-sm text-muted">Gespeichert</p>
		{/if}
		<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Speichern</button>
	</form>
</section>
