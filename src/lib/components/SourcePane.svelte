<script lang="ts">
	// The source in a review, beside the text: a page of a scan on its lightbox, dark in both
	// themes so the page is the brightest thing on screen, with zoom; a text file set as text,
	// with words looked for marked; or a player for a recording or a video. Where no strip of
	// pages runs below, buttons over the scan turn its pages.
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
	import { findTerm } from '#lib/analysis.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		kind,
		piece = null,
		paging = null,
		onstep,
		preview = null,
		text = null,
		find = '',
		player
	}: {
		kind: 'scan' | 'text' | 'player';
		/** The piece shown, after "Source". */
		piece?: string | null;
		paging?: { at: number; count: number } | null;
		onstep?: (delta: number) => void;
		preview?: PagePreview | null;
		text?: string | null;
		/** Words to find in a text file. */
		find?: string;
		player?: Snippet;
	} = $props();
	const m = messages();

	let zoom = $state(1);
</script>

<section class="pane source" class:stage={kind === 'scan'} aria-label={m.review.source}>
	<div class="pane-bar">
		<span class="title mono">{m.review.source}{piece ? ` · ${piece}` : ''}</span>
		{#if paging && paging.count > 1 && onstep}
			<span class="paging">
				<button
					class="tool"
					aria-label={m.review.previous}
					title={m.review.previous}
					disabled={paging.at <= 0}
					onclick={() => onstep(-1)}><Icon name="previous" /></button
				>
				<span class="count mono">{paging.at + 1} / {paging.count}</span>
				<button
					class="tool"
					aria-label={m.review.next}
					title={m.review.next}
					disabled={paging.at >= paging.count - 1}
					onclick={() => onstep(1)}><Icon name="chevron" /></button
				>
			</span>
		{/if}
	</div>
	{#if kind === 'scan'}
		<div class="desk"><PageImage {preview} {zoom} /></div>
		<div class="zoom">
			<button
				class="tool"
				aria-label={m.review.zoomOut}
				title={m.review.zoomOut}
				onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}><Icon name="zoom-out" /></button
			>
			<span class="zoom-value mono">{Math.round(zoom * 100)}%</span>
			<button
				class="tool"
				aria-label={m.review.zoomIn}
				title={m.review.zoomIn}
				onclick={() => (zoom = Math.min(3, zoom + 0.2))}><Icon name="zoom-in" /></button
			>
			<span class="zoom-rule" aria-hidden="true"></span>
			<button class="tool wide" onclick={() => (zoom = 1)}>{m.review.fitWidth}</button>
		</div>
	{:else if kind === 'text'}
		<div class="desk document" lang="">
			{#if text === null}
				<p class="muted">{m.common.loading}</p>
			{:else}
				{#each findTerm(text, find) as part, k (k)}{#if part.hit}<mark>{part.text}</mark
						>{:else}{part.text}{/if}{/each}
			{/if}
		</div>
	{:else}
		<div class="desk">{@render player?.()}</div>
	{/if}
</section>

<style>
	.source {
		border-right: 1px solid var(--line);
		background: var(--surface-1);
	}
	.title {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.stage {
		background: var(--stage);
		border-right-color: var(--stage-line);
	}
	.stage .pane-bar {
		border-bottom-color: var(--stage-line);
	}
	.stage .title {
		color: var(--on-stage-muted);
	}
	.desk {
		overflow: auto;
		min-height: 0;
		padding: var(--space-4);
	}
	.stage .desk {
		padding: var(--space-5) var(--space-5) 4.5rem;
	}
	.stage .desk :global(.muted) {
		color: var(--on-stage-muted);
	}
	/* A text file, set in the reading face. */
	.document {
		padding: var(--space-5) var(--space-6);
		background: var(--paper);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.65;
		color: var(--text-body);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	mark {
		background: var(--marker);
		color: var(--on-marker);
		border-radius: 2px;
		box-shadow: 0 0 0 2px var(--marker);
	}
	/* The buttons over the lightbox: turning its pages, zooming. */
	.paging {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 2px;
		color: var(--on-stage);
	}
	.count {
		font-size: var(--type-xs);
		color: var(--on-stage-muted);
	}
	.zoom {
		position: absolute;
		left: 50%;
		bottom: var(--space-4);
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 2px;
		padding: var(--space-1);
		border-radius: 9px;
		background: var(--stage-raised);
		box-shadow:
			0 0 0 1px var(--stage-line),
			0 8px 24px rgba(0, 0, 0, 0.35);
		color: var(--on-stage);
	}
	.tool {
		display: grid;
		place-items: center;
		min-width: var(--control-sm);
		height: var(--control-sm);
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.tool.wide {
		padding: 0 0.625rem;
		white-space: nowrap;
	}
	.tool:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.08);
	}
	.tool:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.tool:focus-visible {
		outline-color: var(--on-stage);
	}
	.zoom-value {
		min-width: 3rem;
		text-align: center;
		font-size: var(--type-xs);
	}
	.zoom-rule {
		width: 1px;
		height: 1.125rem;
		margin: 0 var(--space-1);
		background: var(--stage-line);
	}
	/* A narrow window stacks the panes: the source on top. */
	@media (max-width: 1000px) {
		.source {
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
	}
</style>
