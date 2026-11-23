<script lang="ts">
	// What can be done with text selected in the text review, opened from the selection (a right
	// click, the menu key or Shift+F10): copy it, or ask a model to correct it. Opened only
	// where the selection is in the text; elsewhere the system's own menu stays. At the pointer
	// or under the selection, kept inside the window; closed by Escape, a click elsewhere or a
	// choice, the focus going back to where it was.
	import ActionMenu from './ActionMenu.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { copyText } from '#lib/menu.js';

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
</script>

<ActionMenu
	{x}
	{y}
	label={m.passage.menu}
	width="19rem"
	{onclose}
	actions={[
		{ label: m.passage.copy, icon: 'copy', run: () => void copyText(text) },
		{ label: m.passage.correct, icon: 'text_correction', run: oncorrect }
	]}
/>
