<script lang="ts">
	let {
		hours = $bindable(),
		minutes = $bindable(),
		label = 'Uhrzeit',
		disabled = false
	}: {
		hours: number;
		minutes: number;
		label?: string;
		disabled?: boolean;
	} = $props();

	function clamp(value: number, max: number): number {
		if (Number.isNaN(value)) return 0;
		return Math.min(max, Math.max(0, value));
	}

	function bumpHours(delta: number) {
		hours = (hours + delta + 24) % 24;
	}

	function bumpMinutes(delta: number) {
		const next = minutes + delta;
		if (next >= 60) {
			minutes = next % 60;
			bumpHours(1);
		} else if (next < 0) {
			minutes = 60 + next;
			bumpHours(-1);
		} else {
			minutes = next;
		}
	}
</script>

<div>
	<p class="mb-1 text-xs uppercase tracking-[0.18em] text-muted">{label}</p>
	<div class="panel flex items-center gap-1 rounded-md px-2 py-1 {disabled ? 'opacity-40' : ''}">
		<button type="button" class="px-1 text-muted" onclick={() => bumpHours(-1)} {disabled}>−</button>
		<input
			class="clock-face w-10 bg-transparent text-center text-lg outline-none"
			type="text"
			inputmode="numeric"
			value={String(hours).padStart(2, '0')}
			{disabled}
			oninput={(event) => {
				hours = clamp(Number((event.currentTarget as HTMLInputElement).value), 23);
			}}
		/>
		<span class="clock-face text-lg">:</span>
		<input
			class="clock-face w-10 bg-transparent text-center text-lg outline-none"
			type="text"
			inputmode="numeric"
			value={String(minutes).padStart(2, '0')}
			{disabled}
			oninput={(event) => {
				minutes = clamp(Number((event.currentTarget as HTMLInputElement).value), 59);
			}}
		/>
		<button type="button" class="px-1 text-muted" onclick={() => bumpHours(1)} {disabled}>+</button>
		<button type="button" class="ml-1 text-xs text-muted" onclick={() => bumpMinutes(-1)} {disabled}>−1m</button>
		<button type="button" class="text-xs text-muted" onclick={() => bumpMinutes(1)} {disabled}>+1m</button>
	</div>
</div>
