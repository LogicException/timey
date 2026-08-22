<script lang="ts">
	import { api } from '$lib/api';
	import TimeField from '$lib/components/TimeField.svelte';
	import type { UserSettings } from '$lib/types';
	import { DEFAULT_WORK_END, DEFAULT_WORK_START, joinHm, splitHm } from '$lib/working-hours';

	let startH = $state(7);
	let startM = $state(30);
	let endH = $state(16);
	let endM = $state(15);
	let error = $state('');
	let saved = $state(false);

	function apply(settings: UserSettings) {
		const start = splitHm(settings.work_start);
		const end = splitHm(settings.work_end);
		startH = start.hours;
		startM = start.minutes;
		endH = end.hours;
		endM = end.minutes;
	}

	async function load() {
		const settings = await api<UserSettings>('/api/settings');
		apply(settings);
	}

	$effect(() => {
		void load().catch((err) => {
			error = err instanceof Error ? err.message : 'Laden fehlgeschlagen';
			apply({ work_start: DEFAULT_WORK_START, work_end: DEFAULT_WORK_END });
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
					work_end: joinHm(endH, endM)
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
	<h2 class="text-lg">Übliche Arbeitszeit</h2>
	<p class="text-sm text-muted">
		Die Wochenansicht beginnt 30 Minuten vor Arbeitsbeginn und endet 30 Minuten nach Arbeitsende.
	</p>
	<form
		class="space-y-4"
		onsubmit={(event) => {
			event.preventDefault();
			void save();
		}}
	>
		<div class="grid gap-3 sm:grid-cols-2">
			<TimeField bind:hours={startH} bind:minutes={startM} label="Beginn" />
			<TimeField bind:hours={endH} bind:minutes={endM} label="Ende" />
		</div>
		{#if error}
			<p class="text-sm text-stop">{error}</p>
		{/if}
		{#if saved}
			<p class="text-sm text-muted">Gespeichert</p>
		{/if}
		<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Speichern</button>
	</form>
</section>
