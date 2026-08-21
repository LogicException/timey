<script lang="ts">
	import { api } from '$lib/api';
	import type { NamedItem } from '$lib/types';

	let tasks = $state<NamedItem[]>([]);
	let aufgaben = $state<NamedItem[]>([]);
	let name = $state('');
	let aufgabeName = $state('');
	let error = $state('');

	async function load() {
		tasks = await api('/api/tasks?include_archived=true');
		aufgaben = await api('/api/aufgaben?include_archived=true');
	}

	$effect(() => {
		void load();
	});

	async function createTask() {
		error = '';
		try {
			await api('/api/tasks', { method: 'POST', body: JSON.stringify({ name }) });
			name = '';
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function createAufgabe() {
		error = '';
		try {
			await api('/api/aufgaben', { method: 'POST', body: JSON.stringify({ name: aufgabeName }) });
			aufgabeName = '';
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function archive(kind: 'tasks' | 'aufgaben', item: NamedItem) {
		await api(`/api/${kind}/${item.id}`, {
			method: 'PATCH',
			body: JSON.stringify({ archived: !item.archived })
		});
		await load();
	}
</script>

<div class="grid gap-6 md:grid-cols-2">
	<section class="panel space-y-3 rounded-xl p-4">
		<h2 class="text-lg">Eigene Tasks</h2>
		<form
			class="flex gap-2"
			onsubmit={(event) => {
				event.preventDefault();
				void createTask();
			}}
		>
			<input class="panel flex-1 rounded-md px-3 py-2" bind:value={name} placeholder="Substantiv, z. B. Meeting, Review, Coding" />
			<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Anlegen</button>
		</form>
		<ul class="space-y-2">
			{#each tasks as item}
				<li class="flex items-center justify-between text-sm">
					<span class={item.archived ? 'text-muted line-through' : ''}>{item.name}</span>
					<button class="text-xs text-muted" onclick={() => archive('tasks', item)}
						>{item.archived ? 'Reaktivieren' : 'Archivieren'}</button
					>
				</li>
			{/each}
		</ul>
	</section>
	<section class="panel space-y-3 rounded-xl p-4">
		<h2 class="text-lg">Eigene Aufgaben</h2>
		<form
			class="flex gap-2"
			onsubmit={(event) => {
				event.preventDefault();
				void createAufgabe();
			}}
		>
			<input class="panel flex-1 rounded-md px-3 py-2" bind:value={aufgabeName} placeholder="z. B. beantworten, schreiben, teilnehmen" />
			<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Anlegen</button>
		</form>
		<ul class="space-y-2">
			{#each aufgaben as item}
				<li class="flex items-center justify-between text-sm">
					<span class={item.archived ? 'text-muted line-through' : ''}>{item.name}</span>
					<button class="text-xs text-muted" onclick={() => archive('aufgaben', item)}
						>{item.archived ? 'Reaktivieren' : 'Archivieren'}</button
					>
				</li>
			{/each}
		</ul>
	</section>
</div>
{#if error}
	<p class="mt-3 text-sm text-stop">{error}</p>
{/if}
