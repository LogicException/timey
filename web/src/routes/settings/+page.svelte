<script lang="ts">
	import { api } from '$lib/api';
	import { isReservedTaskName, renameIfChanged } from '$lib/catalog-name';
	import type { NamedItem } from '$lib/types';

	let tasks = $state<NamedItem[]>([]);
	let name = $state('');
	let error = $state('');
	let editingId = $state<number | null>(null);
	let draft = $state('');
	let skipBlur = $state(false);

	async function load() {
		tasks = await api('/api/tasks?include_archived=true');
	}

	$effect(() => {
		void load();
	});

	async function createTask() {
		error = '';
		if (isReservedTaskName(name)) {
			error = 'Name ist reserviert';
			return;
		}
		try {
			await api('/api/tasks', { method: 'POST', body: JSON.stringify({ name }) });
			name = '';
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function archive(item: NamedItem) {
		await api(`/api/tasks/${item.id}`, {
			method: 'PATCH',
			body: JSON.stringify({ archived: !item.archived })
		});
		await load();
	}

	async function remove(item: NamedItem) {
		if (!confirm('Einträge werden auf „unbestimmt“ umgebucht.')) {
			return;
		}
		error = '';
		try {
			await api(`/api/tasks/${item.id}`, { method: 'DELETE' });
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	function startEdit(item: NamedItem) {
		editingId = item.id;
		draft = item.name;
		skipBlur = false;
	}

	function focusDraft(node: HTMLInputElement) {
		node.focus();
		node.select();
		return {};
	}

	function cancelEdit() {
		skipBlur = true;
		editingId = null;
		draft = '';
	}

	async function commitEdit(item: NamedItem) {
		if (skipBlur) {
			skipBlur = false;
			return;
		}
		const next = renameIfChanged(item.name, draft);
		editingId = null;
		draft = '';
		if (next == null) return;
		error = '';
		if (isReservedTaskName(next)) {
			error = 'Name ist reserviert';
			return;
		}
		try {
			await api(`/api/tasks/${item.id}`, {
				method: 'PATCH',
				body: JSON.stringify({ name: next })
			});
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}
</script>

<section class="panel space-y-3 rounded-xl p-4">
	<h2 class="text-lg">Eigene Tasks</h2>
	<form
		class="flex gap-2"
		onsubmit={(event) => {
			event.preventDefault();
			void createTask();
		}}
	>
		<input
			class="panel flex-1 rounded-md px-3 py-2"
			bind:value={name}
			placeholder="z. B. E-Mail schreiben, Meeting teilnehmen"
		/>
		<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Anlegen</button>
	</form>
	<ul class="space-y-2">
		{#each tasks as item}
			<li class="flex items-center justify-between gap-3 text-sm">
				{#if editingId === item.id}
					<input
						class="panel min-w-0 flex-1 rounded-md px-2 py-1"
						use:focusDraft
						bind:value={draft}
						onblur={() => void commitEdit(item)}
						onkeydown={(event) => {
							if (event.key === 'Enter') {
								event.preventDefault();
								void commitEdit(item);
							}
							if (event.key === 'Escape') {
								event.preventDefault();
								cancelEdit();
							}
						}}
					/>
				{:else}
					<button
						class="min-w-0 flex-1 truncate text-left {item.archived ? 'text-muted line-through' : ''}"
						onclick={() => startEdit(item)}>{item.name}</button
					>
				{/if}
				<div class="flex shrink-0 gap-3">
					<button class="text-xs text-muted" onclick={() => archive(item)}
						>{item.archived ? 'Reaktivieren' : 'Archivieren'}</button
					>
					<button class="text-xs text-stop" onclick={() => void remove(item)}>Löschen</button>
				</div>
			</li>
		{/each}
	</ul>
</section>
{#if error}
	<p class="mt-3 text-sm text-stop">{error}</p>
{/if}
