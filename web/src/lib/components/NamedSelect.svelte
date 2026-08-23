<script lang="ts">
	import { namedSelectOptions } from '$lib/named-select';
	import type { NamedItem } from '$lib/types';

	let {
		label,
		items,
		value = $bindable(),
		optional = false,
		currentLabel = null
	}: {
		label: string;
		items: NamedItem[];
		value: number | null;
		optional?: boolean;
		currentLabel?: string | null;
	} = $props();

	const options = $derived(namedSelectOptions(items, value, currentLabel));
</script>

<label class="block">
	<p class="mb-1 text-xs uppercase tracking-[0.18em] text-muted">{label}</p>
	<select
		class="panel w-full rounded-md px-3 py-2 text-sm"
		value={value ?? ''}
		onchange={(event) => {
			const raw = (event.currentTarget as HTMLSelectElement).value;
			value = raw === '' ? null : Number(raw);
		}}
	>
		{#if optional}
			<option value="">—</option>
		{/if}
		{#each options as item}
			<option value={item.id}>{item.name}</option>
		{/each}
	</select>
</label>
