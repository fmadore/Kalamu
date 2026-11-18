<script lang="ts">
	// What can be done with text selected in the text review, opened from the selection (a right
	// click, the menu key or Shift+F10): copy it, or ask a model to correct it. Opened only
	// where the selection is in the text; elsewhere the system's own menu stays. At the pointer
	// or under the selection, kept inside the window; closed by Escape, a click elsewhere or a
	// choice, the focus going back to where it was.
	import ContextMenu from './ContextMenu.svelte';
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';

	let {
		text,
		x,
		y,
		oncorrect,
		onclose
	}: {
		/** The text selected, to copy. */
		text: string;
		/** Where it opens, in the window. */
		x: number;
		y: number;
		oncorrect: () => void;
		onclose: () => void;
	} = $props();
	const m = messages();

	let menu = $state<ContextMenu | null>(null);

	async function copy() {
		menu?.close();
		try {
			await navigator.clipboard.writeText(text);
		} catch {
			// A webview that refuses the clipboard to a script still copies a selection.
			document.execCommand('copy');
		}
	}
</script>

<ContextMenu bind:this={menu} {x} {y} label={m.passage.menu} width="19rem" {onclose}>
	<button type="button" role="menuitem" class="item" onclick={copy}
		><Icon name="copy" size={14} />{m.passage.copy}</button
	>
	<button
		type="button"
		role="menuitem"
		class="item"
		onclick={() => {
			menu?.close();
			oncorrect();
		}}><Icon name="text_correction" size={14} />{m.passage.correct}</button
	>
</ContextMenu>

<style>
	.item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		min-height: 2rem;
		padding: 0 var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
	}
	.item:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
	}
</style>
