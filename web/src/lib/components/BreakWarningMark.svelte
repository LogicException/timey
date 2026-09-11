<script lang="ts">
	import { breakWarningTooltip, type BreakViolation } from '$lib/break-compliance';
	import type { Snippet } from 'svelte';
	import BreakWarnings from './BreakWarnings.svelte';

	let {
		violations,
		tooltipId,
		children
	}: {
		violations: BreakViolation[];
		tooltipId: string;
		children: Snippet;
	} = $props();

	const tooltip = $derived(breakWarningTooltip(violations));
</script>

<div class="group/break relative flex items-center gap-1.5">
	{#if tooltip}
		<button
			type="button"
			class="grid h-4 w-4 place-items-center rounded-full bg-stop text-[11px] font-bold leading-none text-bg"
			aria-describedby={tooltipId}
			aria-label="Pausenhinweis"
		>
			!
		</button>
		<div
			id={tooltipId}
			role="tooltip"
			class="pointer-events-none invisible absolute left-0 top-full z-[60] mt-1 w-96 rounded-md border border-line bg-panel px-3 py-2 shadow-lg group-hover/break:visible group-focus-within/break:visible"
		>
			<BreakWarnings {violations} compact />
		</div>
	{/if}
	{@render children()}
</div>
