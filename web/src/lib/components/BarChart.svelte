<script lang="ts">
	import { formatHm } from '$lib/format';
	import { colorForIndex, type ChartGroup, type ChartSlice } from '$lib/report-chart';

	let { slices = [], groups }: { slices?: ChartSlice[]; groups?: ChartGroup[] } = $props();

	const bars = $derived(groups ? groups.flatMap((group) => group.bars) : slices);
	const maxSeconds = $derived(Math.max(0, ...bars.map((item) => item.seconds)));

	function widthPercent(seconds: number): number {
		if (maxSeconds <= 0) return 0;
		return (seconds / maxSeconds) * 100;
	}

	function colorIndex(groupIndex: number, barIndex: number): number {
		if (!groups) return barIndex;
		let index = 0;
		for (let i = 0; i < groupIndex; i += 1) {
			index += groups[i]?.bars.length ?? 0;
		}
		return index + barIndex;
	}
</script>

{#snippet barRow(item: ChartSlice, index: number)}
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
{/snippet}

<div class="max-h-[28rem] space-y-3 overflow-y-auto p-4">
	{#if groups}
		{#each groups as group, groupIndex}
			<div class="space-y-3">
				<div class="flex items-baseline justify-between gap-3 text-sm font-medium">
					<span>{group.label}</span>
					<span class="clock-face">{formatHm(group.seconds)}</span>
				</div>
				<div class="space-y-3 pl-4">
					{#each group.bars as item, barIndex}
						{@render barRow(item, colorIndex(groupIndex, barIndex))}
					{/each}
				</div>
			</div>
		{/each}
	{:else}
		{#each slices as item, index}
			{@render barRow(item, index)}
		{/each}
	{/if}
</div>
