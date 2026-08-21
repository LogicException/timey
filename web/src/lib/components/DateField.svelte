<script lang="ts">
	import { addDays, startOfWeek } from '$lib/dates';

	let {
		value = $bindable(),
		label = 'Datum'
	}: {
		value: string;
		label?: string;
	} = $props();

	let open = $state(false);
	let cursor = $derived(value.slice(0, 7) + '-01');

	let viewMonth = $state('');
	$effect(() => {
		if (!viewMonth) viewMonth = cursor;
	});

	function monthLabel(date: string): string {
		const [y, m] = date.split('-').map(Number);
		return new Date(Date.UTC(y, m - 1, 1)).toLocaleDateString('de-DE', {
			month: 'long',
			year: 'numeric',
			timeZone: 'UTC'
		});
	}

	function daysInGrid(monthStart: string): string[] {
		const first = startOfWeek(monthStart);
		return Array.from({ length: 42 }, (_, i) => addDays(first, i));
	}

	function shiftMonth(delta: number) {
		const [y, m] = (viewMonth || cursor).split('-').map(Number);
		const next = new Date(Date.UTC(y, m - 1 + delta, 1));
		viewMonth = `${next.getUTCFullYear()}-${String(next.getUTCMonth() + 1).padStart(2, '0')}-01`;
	}

	function select(day: string) {
		value = day;
		open = false;
	}
</script>

<div class="relative">
	<p class="mb-1 text-xs uppercase tracking-[0.18em] text-muted">{label}</p>
	<button
		type="button"
		class="clock-face panel w-full rounded-md px-3 py-2 text-left text-sm"
		onclick={() => {
			viewMonth = `${value.slice(0, 7)}-01`;
			open = !open;
		}}
	>
		{value}
	</button>
	{#if open}
		<div class="panel absolute z-20 mt-2 w-72 rounded-lg p-3">
			<div class="mb-2 flex items-center justify-between">
				<button type="button" class="text-muted" onclick={() => shiftMonth(-1)}>‹</button>
				<span class="text-sm capitalize">{monthLabel(viewMonth || cursor)}</span>
				<button type="button" class="text-muted" onclick={() => shiftMonth(1)}>›</button>
			</div>
			<div class="mb-1 grid grid-cols-7 gap-1 text-center text-[10px] uppercase tracking-wider text-muted">
				{#each ['Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa', 'So'] as dow}
					<span>{dow}</span>
				{/each}
			</div>
			<div class="grid grid-cols-7 gap-1">
				{#each daysInGrid(viewMonth || cursor) as day}
					<button
						type="button"
						class="rounded py-1 text-sm {day === value
							? 'bg-amber text-bg'
							: day.startsWith((viewMonth || cursor).slice(0, 7))
								? 'hover:bg-panel-2'
								: 'text-muted/50'}"
						onclick={() => select(day)}
					>
						{Number(day.slice(8))}
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>
