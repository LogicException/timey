<script lang="ts">
	import { formatHm } from '$lib/format';
	import { colorForIndex, slicesToArcs, type ChartSlice } from '$lib/report-chart';

	let { slices }: { slices: ChartSlice[] } = $props();

	const arcs = $derived(slicesToArcs(slices));
	const cx = 100;
	const cy = 100;
	const radius = 80;

	function pointOnCircle(deg: number): { x: number; y: number } {
		const rad = (deg * Math.PI) / 180;
		return { x: cx + radius * Math.cos(rad), y: cy + radius * Math.sin(rad) };
	}

	function arcPath(startDeg: number, endDeg: number): string {
		const start = pointOnCircle(startDeg);
		const end = pointOnCircle(endDeg);
		const large = endDeg - startDeg > 180 ? 1 : 0;
		return `M ${cx} ${cy} L ${start.x} ${start.y} A ${radius} ${radius} 0 ${large} 1 ${end.x} ${end.y} Z`;
	}
</script>

<div class="flex flex-col gap-6 p-4 md:flex-row md:items-center">
	<svg viewBox="0 0 200 200" class="mx-auto h-56 w-56 shrink-0" role="img" aria-label="Tortendiagramm">
		{#each arcs as arc, index}
			{#if arc.kind === 'full'}
				<circle cx={cx} cy={cy} r={radius} fill={colorForIndex(index)} />
			{:else}
				<path d={arcPath(arc.startDeg, arc.endDeg)} fill={colorForIndex(index)} />
			{/if}
		{/each}
	</svg>
	<ul class="flex-1 space-y-2 text-sm">
		{#each arcs as arc, index}
			<li class="flex items-center justify-between gap-3">
				<span class="flex min-w-0 items-center gap-2">
					<span
						class="h-2.5 w-2.5 shrink-0 rounded-full"
						style="background: {colorForIndex(index)}"
						aria-hidden="true"
					></span>
					<span class="truncate">{arc.label}</span>
				</span>
				<span class="clock-face shrink-0">{formatHm(arc.seconds)} · {Math.round(arc.percent)}%</span>
			</li>
		{/each}
	</ul>
</div>
