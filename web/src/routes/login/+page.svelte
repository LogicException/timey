<script lang="ts">
	import { goto } from '$app/navigation';
	import { fetchAuthConfig, login } from '$lib/auth';
	import type { AuthConfig } from '$lib/types';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let config = $state<AuthConfig>({ local: true, oidc: false });

	$effect(() => {
		void fetchAuthConfig().then((value) => {
			config = value;
		});
	});

	async function submit(event: Event) {
		event.preventDefault();
		error = '';
		try {
			await login(username, password);
			await goto('/');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Anmeldung fehlgeschlagen';
		}
	}
</script>

<div class="grid min-h-screen place-items-center px-4">
	<form class="panel w-full max-w-sm space-y-4 rounded-2xl p-8" onsubmit={submit}>
		<p class="text-[11px] uppercase tracking-[0.28em] text-amber">Timey</p>
		<h1 class="text-3xl font-bold">Anmelden</h1>
		{#if config.local}
			<label class="block text-sm">
				<span class="text-muted">Benutzername</span>
				<input class="panel mt-1 w-full rounded-md px-3 py-2" bind:value={username} autocomplete="username" />
			</label>
			<label class="block text-sm">
				<span class="text-muted">Passwort</span>
				<input
					class="panel mt-1 w-full rounded-md px-3 py-2"
					type="password"
					bind:value={password}
					autocomplete="current-password"
				/>
			</label>
			{#if error}
				<p class="text-sm text-stop">{error}</p>
			{/if}
			<button class="w-full rounded-md bg-amber py-2 font-semibold text-bg">Einloggen</button>
		{/if}
		{#if config.oidc}
			<p class="text-sm text-muted">Keycloak-Login ist vorbereitet, in v1 noch nicht aktiv.</p>
		{/if}
	</form>
</div>
