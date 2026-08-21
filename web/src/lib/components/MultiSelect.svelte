<script lang="ts">
	import { filterNamedItems, toggleId } from '$lib/multiselect';
	import type { NamedItem } from '$lib/types';

	let {
		label,
		items,
		value = $bindable(),
		placeholder = 'Suchen …'
	}: {
		label: string;
		items: NamedItem[];
		value: number[];
		placeholder?: string;
	} = $props();

	let open = $state(false);
	let query = $state('');
	let root: HTMLDivElement | undefined;
	let searchEl = $state<HTMLInputElement | undefined>(undefined);

	const listId = $derived(`multiselect-${label.toLowerCase()}`);
	const selectedItems = $derived(items.filter((item) => value.includes(item.id)));
	const visible = $derived(filterNamedItems(items, query));
	const visibleChips = $derived(selectedItems.slice(0, 2));
	const extraCount = $derived(Math.max(0, selectedItems.length - visibleChips.length));

	$effect(() => {
		if (!open) return;
		queueMicrotask(() => searchEl?.focus());
		function onPointer(event: PointerEvent) {
			if (root && !root.contains(event.target as Node)) {
				open = false;
				query = '';
			}
		}
		function onKey(event: KeyboardEvent) {
			if (event.key === 'Escape') {
				open = false;
				query = '';
			}
		}
		document.addEventListener('pointerdown', onPointer);
		document.addEventListener('keydown', onKey);
		return () => {
			document.removeEventListener('pointerdown', onPointer);
			document.removeEventListener('keydown', onKey);
		};
	});

	function toggle(id: number) {
		value = toggleId(value, id);
	}

	function clearAll(event: MouseEvent) {
		event.stopPropagation();
		value = [];
	}
</script>

<div class="relative" bind:this={root}>
	<p class="mb-1 text-xs uppercase tracking-[0.18em] text-muted">{label}</p>
	<div
		class="panel flex min-h-11 w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-left"
		role="combobox"
		aria-controls={listId}
		aria-haspopup="listbox"
		aria-expanded={open}
		tabindex="0"
		onclick={() => (open = !open)}
		onkeydown={(event) => {
			if (event.key === 'Enter' || event.key === ' ') {
				event.preventDefault();
				open = !open;
			}
		}}
	>
		<div class="flex min-w-0 flex-1 flex-wrap items-center gap-1">
			{#if selectedItems.length === 0}
				<span class="px-1 text-sm text-muted">Alle</span>
			{:else}
				{#each visibleChips as item (item.id)}
					<span
						class="inline-flex max-w-full items-center gap-1 rounded-full border border-amber/40 bg-amber/15 py-0.5 pl-2 pr-1 text-xs text-amber"
					>
						<span class="truncate">{item.name}</span>
						<button
							type="button"
							class="px-1 text-amber/80 hover:text-ink"
							onclick={(event) => {
								event.stopPropagation();
								toggle(item.id);
							}}>×</button
						>
					</span>
				{/each}
				{#if extraCount > 0}
					<span class="clock-face text-xs text-muted">+{extraCount}</span>
				{/if}
			{/if}
		</div>
		{#if selectedItems.length > 0}
			<button type="button" class="px-1 text-xs text-muted hover:text-ink" onclick={clearAll}
				>leeren</button
			>
		{/if}
		<span class="text-muted">{open ? '▴' : '▾'}</span>
	</div>
	{#if open}
		<div class="panel absolute z-30 mt-1 w-full overflow-hidden rounded-md shadow-lg shadow-black/40">
			<div class="border-b border-line p-2">
				<input
					bind:this={searchEl}
					class="w-full bg-transparent px-1 text-sm outline-none placeholder:text-muted"
					{placeholder}
					bind:value={query}
				/>
			</div>
			<ul id={listId} class="max-h-56 overflow-y-auto py-1" role="listbox">
				{#each visible as item (item.id)}
					<li>
						<button
							type="button"
							class="flex w-full items-center justify-between px-3 py-2 text-left text-sm hover:bg-panel-2 {value.includes(
								item.id
							)
								? 'text-amber'
								: ''} {item.archived ? 'text-muted' : ''}"
							onclick={() => toggle(item.id)}
						>
							<span>{item.name}{item.archived ? ' (archiviert)' : ''}</span>
							{#if value.includes(item.id)}
								<span class="clock-face text-xs">✓</span>
							{/if}
						</button>
					</li>
				{:else}
					<li class="px-3 py-2 text-sm text-muted">Keine Treffer</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>
