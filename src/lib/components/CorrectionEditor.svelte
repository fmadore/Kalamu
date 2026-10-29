<script lang="ts">
	// The frame of a correction under way, in every review: whose text or answers are being
	// corrected, the editor itself (the caller's), and saving, discarding or going back to the
	// model's output. Ctrl Enter saves, which marks the piece reviewed, as it does outside the
	// editor.
	import type { Snippet } from 'svelte';
	import { messages } from '#lib/i18n/index.js';

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
		if (e.ctrlKey && e.key === 'Enter') {
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
		<button class="btn primary" onclick={onsave}
			>{m.review.save}<span class="kbd">{m.review.reviewedShortcut}</span></button
		>
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
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin-top: var(--space-4);
	}
</style>
