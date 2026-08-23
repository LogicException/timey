<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib/api';
	import { homePath, parseDefaultView } from '$lib/default-view';
	import type { UserSettings } from '$lib/types';

	$effect(() => {
		void api<UserSettings>('/api/settings')
			.then((settings) =>
				goto(homePath(parseDefaultView(settings.default_view)), { replaceState: true })
			)
			.catch(() => goto('/day', { replaceState: true }));
	});
</script>

<div class="grid min-h-[40vh] place-items-center text-muted">lädt …</div>
