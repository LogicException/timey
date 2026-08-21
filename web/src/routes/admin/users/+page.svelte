<script lang="ts">
	import { api } from '$lib/api';

	type AdminUser = {
		id: number;
		username: string;
		role: 'admin' | 'user';
		disabled: boolean;
		auth_provider: string;
	};

	let users = $state<AdminUser[]>([]);
	let username = $state('');
	let password = $state('');
	let role = $state<'admin' | 'user'>('user');
	let error = $state('');

	async function load() {
		users = await api('/api/admin/users');
	}

	$effect(() => {
		void load();
	});

	async function create() {
		error = '';
		try {
			await api('/api/admin/users', {
				method: 'POST',
				body: JSON.stringify({ username, password, role })
			});
			username = '';
			password = '';
			await load();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Fehler';
		}
	}

	async function toggleDisabled(user: AdminUser) {
		await api(`/api/admin/users/${user.id}`, {
			method: 'PATCH',
			body: JSON.stringify({ disabled: !user.disabled })
		});
		await load();
	}
</script>

<section class="panel max-w-2xl space-y-4 rounded-xl p-4">
	<h2 class="text-lg">Benutzer</h2>
	<form
		class="grid gap-2 sm:grid-cols-4"
		onsubmit={(event) => {
			event.preventDefault();
			void create();
		}}
	>
		<input class="panel rounded-md px-3 py-2 sm:col-span-1" bind:value={username} placeholder="Username" />
		<input class="panel rounded-md px-3 py-2 sm:col-span-1" type="password" bind:value={password} placeholder="Passwort" />
		<select class="panel rounded-md px-3 py-2" bind:value={role}>
			<option value="user">user</option>
			<option value="admin">admin</option>
		</select>
		<button class="rounded-md bg-amber px-3 py-2 text-sm text-bg">Anlegen</button>
	</form>
	{#if error}
		<p class="text-sm text-stop">{error}</p>
	{/if}
	<table class="w-full text-sm">
		<thead class="text-left text-xs uppercase tracking-wider text-muted">
			<tr>
				<th class="py-2">Name</th>
				<th>Rolle</th>
				<th>Provider</th>
				<th>Status</th>
			</tr>
		</thead>
		<tbody>
			{#each users as user}
				<tr class="border-t border-line">
					<td class="py-2">{user.username}</td>
					<td>{user.role}</td>
					<td>{user.auth_provider}</td>
					<td>
						<button class="text-xs" onclick={() => toggleDisabled(user)}
							>{user.disabled ? 'deaktiviert' : 'aktiv'}</button
						>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</section>
