<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api } from '$lib/api';
	import { fetchMe, logout } from '$lib/auth';
	import Timers from '$lib/components/Timers.svelte';
	import { setContext } from 'svelte';
	import { REFRESH_TIMERS_KEY } from '$lib/timers-context';
	import { formatBerlinDate } from '$lib/dates';
	import type { Entry, NamedItem, User, WorkDaySummary, WorkInterval, WorkSnapshot } from '$lib/types';

	let { children } = $props();

	let user = $state<User | null>(null);
	let ready = $state(false);
	let work = $state<WorkSnapshot | null>(null);
	let workIntervals = $state<WorkInterval[]>([]);
	let timer = $state<Entry | null>(null);
	let tasks = $state<NamedItem[]>([]);
	let projects = $state<NamedItem[]>([]);

	const publicPath = $derived(page.url.pathname === '/login');

	async function refreshSession() {
		user = await fetchMe();
		if (!user) {
			work = null;
			workIntervals = [];
			timer = null;
			if (!publicPath) await goto('/login');
			return;
		}
		if (publicPath) {
			await goto('/');
			return;
		}
		await refreshTimers();
	}

	async function refreshTimers() {
		const today = formatBerlinDate(new Date());
		const [workRes, timerRes, taskRes, projectRes, workDays] = await Promise.all([
			api<WorkSnapshot>('/api/work-sessions/current'),
			api<Entry | null>('/api/entries/timer'),
			api<NamedItem[]>('/api/tasks'),
			api<NamedItem[]>('/api/projects'),
			api<WorkDaySummary[]>(`/api/work-sessions?from=${today}&to=${today}`)
		]);
		work = workRes;
		timer = timerRes;
		tasks = taskRes;
		projects = projectRes;
		workIntervals = workDays.flatMap((day) => day.intervals ?? []);
	}

	setContext(REFRESH_TIMERS_KEY, refreshTimers);

	$effect(() => {
		void page.url.pathname;
		void refreshSession().finally(() => {
			ready = true;
		});
	});
</script>

<svelte:head>
	<title>Timey</title>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if !ready}
	<div class="grid min-h-screen place-items-center text-muted">lädt …</div>
{:else if publicPath}
	{@render children()}
{:else if user}
	<div class="mx-auto flex min-h-screen max-w-6xl flex-col gap-4 px-4 py-6">
		<header class="flex flex-wrap items-end justify-between gap-4">
			<div>
				<p class="text-[11px] uppercase tracking-[0.28em] text-amber">Zeiterfassung</p>
				<h1 class="text-3xl font-bold">Timey</h1>
			</div>
			<nav class="flex flex-wrap gap-3 text-sm text-muted">
				<a href="/day" class="hover:text-ink">Tag</a>
				<a href="/week" class="hover:text-ink">Woche</a>
				<a href="/report" class="hover:text-ink">Auswertung</a>
				<a href="/projects" class="hover:text-ink">Projekte</a>
				<a href="/settings" class="hover:text-ink">Tasks</a>
				<a href="/preferences" class="hover:text-ink">Einstellungen</a>
				{#if user.role === 'admin'}
					<a href="/admin/users" class="hover:text-ink">Benutzer</a>
				{/if}
				<button
					class="hover:text-ink"
					onclick={async () => {
						await logout();
						user = null;
						await goto('/login');
					}}>Abmelden</button
				>
			</nav>
		</header>
		<Timers {work} {workIntervals} {timer} {tasks} {projects} onRefresh={refreshTimers} />
		<main class="flex-1">{@render children()}</main>
	</div>
{/if}
