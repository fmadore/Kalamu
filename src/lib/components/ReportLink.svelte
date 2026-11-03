<script lang="ts">
	// Report a problem: GitHub's issue forms, a bug or feedback, or harmful output as Microsoft
	// Store policy 11.16 asks, in the default browser. A small button in Settings, an icon in a
	// review's masthead.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { openLink } from '#lib/state.svelte.js';
	import { reportUrl } from '#lib/support.js';

	let { iconOnly = false }: { iconOnly?: boolean } = $props();
	const m = messages();
	const href = reportUrl();

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
