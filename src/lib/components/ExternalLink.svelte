<script lang="ts">
	// A link to a web page. A real link, so it reads as one to a screen reader and shows its
	// address on hover, but the click goes through the opener, which uses the default browser
	// rather than a WebView2 popup.
	import type { Snippet } from 'svelte';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { toast } from '#lib/state.svelte.js';
	import { openUrl } from '#lib/tauri.js';

	let { href, children }: { href: string; children: Snippet } = $props();
	const m = messages();

	async function open(event: MouseEvent) {
		event.preventDefault();
		try {
			await openUrl(href);
		} catch (e) {
			toast(describeError(e, m));
		}
	}
</script>

<a {href} onclick={open}
	>{@render children()}<span class="sr-only">{m.common.opensInBrowser}</span></a
>
