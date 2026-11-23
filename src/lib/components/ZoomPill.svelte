<script lang="ts">
	// The controls floating over the lightbox, centred at its foot: zoom out and in with the
	// percentage between, back to the pane's width, and a quarter turn either way.
	import Icon from './Icon.svelte';
	import { formatPercent, messages } from '#lib/i18n/index.js';
	import { shortcut } from '#lib/platform.js';

	let {
		zoom,
		hint,
		onzoom,
		onfit,
		onturn
	}: {
		zoom: number;
		/** How to zoom with the wheel, on the percentage. */
		hint: string;
		/** Zooms by a factor. */
		onzoom: (factor: number) => void;
		onfit: () => void;
		/** Turns the page a quarter clockwise (1) or anticlockwise (-1). */
		onturn: (by: 1 | -1) => void;
	} = $props();
	const m = messages();
</script>

<div class="zoom">
	<button
		class="tool"
		aria-label={m.review.zoomOut}
		title="{m.review.zoomOut} ({shortcut(m.review.zoomOutShortcut)})"
		onclick={() => onzoom(1 / 1.25)}
	>
		<Icon name="zoom-out" />
	</button>
	<span class="zoom-value mono" title={hint}>{formatPercent(Math.round(zoom * 100) / 100)}</span>
	<button
		class="tool"
		aria-label={m.review.zoomIn}
		title="{m.review.zoomIn} ({shortcut(m.review.zoomInShortcut)})"
		onclick={() => onzoom(1.25)}
	>
		<Icon name="zoom-in" />
	</button>
	<span class="zoom-rule" aria-hidden="true"></span>
	<button
		class="tool wide"
		title="{m.review.fitWidth} ({shortcut(m.review.zoomFitShortcut)})"
		onclick={onfit}
	>
		{m.review.fitWidth}
	</button>
	<span class="zoom-rule" aria-hidden="true"></span>
	<button
		class="tool"
		aria-label={m.review.turnLeft}
		title={m.review.turnLeft}
		onclick={() => onturn(-1)}
	>
		<Icon name="turn-left" />
	</button>
	<button
		class="tool"
		aria-label={m.review.turnRight}
		title={m.review.turnRight}
		onclick={() => onturn(1)}
	>
		<Icon name="turn-right" />
	</button>
</div>

<style>
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
	.tool:hover {
		background: rgba(255, 255, 255, 0.08);
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
</style>
