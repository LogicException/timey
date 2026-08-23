<script lang="ts">
	import { formatHm } from '$lib/format';
	import { colorForIndex, type ChartSlice } from '$lib/report-chart';

	let { slices }: { slices: ChartSlice[] } = $props();

	const maxSeconds = $derived(Math.max(0, ...slices.map((item) => item.seconds)));

	function widthPercent(seconds: number): number {
		if (maxSeconds <= 0) return 0;
		return (seconds / maxSeconds) * 100;
	}
</script>

<div class="max-h-[28rem] space-y-3 overflow-y-auto p-4">
	{#each slices as item, index}
		<div>
			<div class="mb-1 flex items-baseline justify-between gap-3 text-sm">
				<span>{item.label}</span>
				<span class="clock-face">{formatHm(item.seconds)}</span>
			</div>
			<div class="h-3 overflow-hidden rounded-full bg-panel-2" aria-hidden="true">
				<div
					class="h-full rounded-full"
					style="width: {widthPercent(item.seconds)}%; background: {colorForIndex(index)}"
				></div>
			</div>
		</div>
	{/each}
</div>
