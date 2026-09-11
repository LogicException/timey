<script lang="ts">
	import type { WorkStatus, WorkTransportAction } from '$lib/work-transport';
	import { workTransportActions } from '$lib/work-transport';

	let {
		status,
		compact = false,
		onAction
	}: {
		status: WorkStatus;
		compact?: boolean;
		onAction: (action: WorkTransportAction) => void;
	} = $props();

	const actions = $derived(workTransportActions(status));

	const labels: Record<WorkTransportAction, string> = {
		start: 'Start',
		pause: 'Pausieren',
		resume: 'Fortsetzen',
		stop: 'Stoppen'
	};

	function buttonClass(action: WorkTransportAction): string {
		const size = compact
			? 'grid h-8 w-8 place-items-center rounded-md'
			: 'grid h-9 w-9 place-items-center rounded-md';
		if (action === 'start' || action === 'resume') return `${size} bg-go text-bg`;
		if (action === 'stop') return `${size} bg-stop`;
		return `${size} bg-panel-2`;
	}
</script>

<div class="flex gap-2">
	{#each actions as action (action)}
		<button
			type="button"
			class={buttonClass(action)}
			aria-label={labels[action]}
			title={labels[action]}
			onclick={() => onAction(action)}
		>
			{#if action === 'start' || action === 'resume'}
				<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
					<polygon points="6,4 20,12 6,20" fill="currentColor" />
				</svg>
			{:else if action === 'pause'}
				<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
					<rect x="5" y="4" width="5" height="16" fill="currentColor" />
					<rect x="14" y="4" width="5" height="16" fill="currentColor" />
				</svg>
			{:else}
				<svg class="pointer-events-none h-4 w-4" viewBox="0 0 24 24" aria-hidden="true">
					<rect x="5" y="5" width="14" height="14" fill="currentColor" />
				</svg>
			{/if}
		</button>
	{/each}
</div>
