<script lang="ts">
	// Report a problem (Microsoft Store policy 11.16): the GitHub issue form for AI output, in the
	// default browser. A small button in Settings, an icon in a review's masthead.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { app, openLink } from '#lib/state.svelte.js';
	import { reportUrl } from '#lib/support.js';

	let { iconOnly = false }: { iconOnly?: boolean } = $props();
	const m = messages();
	const href = $derived(reportUrl(app.info?.version));

	function open(event: MouseEvent) {
		event.preventDefault();
		void openLink(href);
	}
</script>

{#if iconOnly}
	<a
		class="btn icon ghost"
		{href}
		aria-label={m.report.review}
		title={m.report.review}
		onclick={open}><Icon name="flag" size={14} /></a
	>
{:else}
	<a class="btn sm" {href} onclick={open}><Icon name="flag" size={14} />{m.report.link}</a>
{/if}

<style>
	a {
		text-decoration: none;
	}
</style>
