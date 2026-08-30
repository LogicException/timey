<script lang="ts">
	import type { BreakViolation } from '$lib/break-compliance';

	let {
		violations,
		compact = false
	}: {
		violations: BreakViolation[];
		compact?: boolean;
	} = $props();
</script>

{#if violations.length > 0}
	<div class={compact ? 'space-y-1' : 'space-y-1 rounded-md border border-stop/40 px-3 py-2'}>
		{#each violations as violation (violation.kind + violation.message)}
			<p class={compact ? 'text-xs text-stop' : 'text-sm text-stop'}>{violation.message}</p>
		{/each}
		<p class="text-[11px] text-muted">Hinweis nach ArbZG § 4, keine Rechtsberatung.</p>
	</div>
{/if}
