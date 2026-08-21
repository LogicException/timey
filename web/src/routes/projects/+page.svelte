<script lang="ts">
	import { api } from '$lib/api';
	import { fetchMe } from '$lib/auth';
	import type { NamedItem, User } from '$lib/types';

	let projects = $state<NamedItem[]>([]);
	let name = $state('');
	let error = $state('');
	let user = $state<User | null>(null);

	async function load() {
		user = await fetchMe();
		projects = await api('/api/projects?include_archived=true');
	}

	$effect(() => {
		void load();
	});

	async function create() {
		error = '';
		try {
			await api('/api/projects', { method: 'POST', body: JSON.stringify({ name }) });
			name = '';
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function archive(item: NamedItem) {
		await api(`/api/projects/${item.id}`, {
			method: 'PATCH',
			body: JSON.stringify({ archived: !item.archived })
		});
		await load();
	}

	const isAdmin = $derived(user?.role === 'admin');
</script>

<section class="panel max-w-xl space-y-3 rounded-xl p-4">
	<h2 class="text-lg">Projekte (geteilt)</h2>
	<p class="text-sm text-muted">Für alle Benutzer sichtbar, z. B. Kunde XYZ.</p>
	{#if isAdmin}
		<form
			class="flex gap-2"
			onsubmit={(event) => {
				event.preventDefault();
				void create();
			}}
		>
			<input class="panel flex-1 rounded-md px-3 py-2" bind:value={name} placeholder="Projektname" />
			<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Anlegen</button>
		</form>
	{/if}
	{#if error}
		<p class="text-sm text-stop">{error}</p>
	{/if}
	<ul class="space-y-2">
		{#each projects as item}
			<li class="flex items-center justify-between text-sm">
				<span class={item.archived ? 'text-muted line-through' : ''}>{item.name}</span>
				{#if isAdmin}
					<button class="text-xs text-muted" onclick={() => archive(item)}
						>{item.archived ? 'Reaktivieren' : 'Archivieren'}</button
					>
				{/if}
			</li>
		{/each}
	</ul>
</section>
