<script lang="ts">
	import { api } from '$lib/api';
	import { formatHms } from '$lib/format';
	import type { WorkSnapshot } from '$lib/types';
	import type { WorkTransportAction } from '$lib/work-transport';
	import {
		liveElapsedSeconds,
		workSessionPath,
		workTransportBarVisible
	} from '$lib/work-transport';
	import TransportControls from './TransportControls.svelte';

	let {
		work,
		onRefresh
	}: {
		work: WorkSnapshot | null;
		onRefresh: () => Promise<void>;
	} = $props();

	let displaySeconds = $state(0);
	let error = $state('');

	const visible = $derived(workTransportBarVisible(work?.status ?? null));

	$effect(() => {
		const base = work?.elapsed_seconds ?? 0;
		const status = work?.status ?? null;
		const origin = Date.now();
		displaySeconds = liveElapsedSeconds(base, status, origin, Date.now());
		if (status !== 'running') return;
		const id = setInterval(() => {
			displaySeconds = liveElapsedSeconds(base, status, origin, Date.now());
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
</script>

{#if visible}
	<div class="sticky top-0 z-50 border-b border-line bg-panel">
		<div class="flex h-9 items-center justify-center gap-4">
			<p class="text-[10px] uppercase tracking-[0.22em] text-muted">Arbeitszeit</p>
			<p class="clock-face text-sm">{formatHms(displaySeconds)}</p>
			<TransportControls status={work?.status ?? null} compact onAction={onAction} />
		</div>
		{#if error}
			<p class="pb-1 text-center text-xs text-stop">{error}</p>
		{/if}
	</div>
{/if}
