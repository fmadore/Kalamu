<script lang="ts">
	// The frame of a correction under way, in every review: whose text or answers are being
	// corrected, the editor itself (the caller's), and saving, discarding or going back to the
	// model's output. Ctrl Enter (⌘ Enter on a Mac) saves, which marks the piece reviewed, as it does outside the
	// editor and as the Save button does. The buttons stay at the foot of the pane while the
	// editor, as tall as its text, scrolls under them, so a short window never hides them.
	import type { Snippet } from 'svelte';
	import { messages } from '#lib/i18n/index.js';
	import { primary, shortcut } from '#lib/platform.js';

	let {
		label,
		siglum = null,
		onsave,
		ondiscard,
		onrevert,
		children
	}: {
		/** The set-up being corrected. */
		label: string;
		/** Its siglum, in a comparison. */
		siglum?: string | null;
		onsave: () => void;
		ondiscard: () => void;
		/** Drops the correction kept: given only when there is one. */
		onrevert?: () => void;
		children: Snippet;
	} = $props();
	const m = messages();

	function onkeydown(e: KeyboardEvent) {
		if (primary(e) && e.key === 'Enter') {
			e.preventDefault();
			e.stopPropagation();
			onsave();
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="correction" {onkeydown}>
	<p class="of">
		{m.review.correcting}
		{#if siglum}<i class="siglum">{siglum}</i>{/if}
		<span class="mono">{label}</span>
	</p>
	{@render children()}
	<div class="actions">
		<button class="btn primary" onclick={onsave}>
			{m.review.save}<span class="kbd">{shortcut(m.review.reviewedShortcut)}</span>
		</button>
		<button class="btn ghost" onclick={ondiscard}>{m.review.discard}</button>
		{#if onrevert}
			<button class="btn ghost" onclick={onrevert}>{m.review.revert}</button>
		{/if}
	</div>
</div>

<style>
	.of {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin-bottom: var(--space-3);
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	/* At the foot of the pane, over the editor scrolling under them: in a short window the
	   buttons stay in reach without scrolling the text. Where the webview keeps them above the
	   pane's bottom padding, their shadow, in the paper's colour, hides the text there. */
	.actions {
		position: sticky;
		bottom: 0;
		z-index: 1;
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin-top: var(--space-4);
		padding: var(--space-3) 0;
		border-top: 1px solid var(--line-soft);
		background: var(--paper);
		box-shadow: 0 var(--space-7) 0 2px var(--paper);
	}
</style>
