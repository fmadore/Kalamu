<script lang="ts">
	// The frame every review shares. The masthead: back to the job, the file, the menu of pages,
	// the actions and "mark as reviewed". Below it the source (a scan on its lightbox with zoom, a
	// text file set as text, or a player), the text or answers beside it, and a third pane while a
	// compared text is open, with handles on the boundaries; the strip of pages at the foot. In a
	// narrow window the panes stack. Ctrl Enter marks the piece reviewed; Page Up and Page Down,
	// or Alt and the arrows, turn the pages; Ctrl F opens the search, under the masthead.
	import { tick, type Snippet } from 'svelte';
	import Icon from './Icon.svelte';
	import SourcePane from './SourcePane.svelte';
	import Splitter from './Splitter.svelte';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import { messages } from '#lib/i18n/index.js';
	import { columns, layout } from '#lib/review.svelte.js';
	import { go } from '#lib/state.svelte.js';

	let {
		jobId,
		jobName,
		fileName,
		error,
		pager,
		actions,
		search,
		searching = $bindable(false),
		reviewed = null,
		onreviewed,
		onstep,
		source,
		piece = null,
		paging = null,
		preview = null,
		text = null,
		find = '',
		player,
		witness,
		label,
		tools,
		children,
		foot,
		panel,
		textPane = $bindable(null),
		third,
		thirdLabel = '',
		thirdBar,
		thirdPane = $bindable(null),
		strip
	}: {
		jobId: string;
		jobName: string | null;
		fileName: string;
		error: string | null;
		/** The menu of pages, in the masthead. */
		pager?: Snippet;
		/** The masthead's actions, before "mark as reviewed". */
		actions?: Snippet;
		/** The search, under the masthead while it is open. */
		search?: Snippet;
		searching?: boolean;
		/** Whether the piece is reviewed; null where it cannot be marked (while correcting). */
		reviewed?: boolean | null;
		onreviewed?: () => void;
		/** Turns the page, by one forwards or backwards. */
		onstep?: (delta: number) => void;
		/** How the source is shown: a page of a scan, a text file's text, or a player. */
		source: 'scan' | 'text' | 'player';
		/** The piece shown, after "Source" over it. */
		piece?: string | null;
		/** Buttons to turn the scan's pages over it, where there is no strip. */
		paging?: { at: number; count: number } | null;
		preview?: PagePreview | null;
		text?: string | null;
		/** Words to find in a text file, marked where they occur. */
		find?: string;
		player?: Snippet;
		/** Whose text or answers lead, over them. */
		witness: { label: string; siglum?: string | null; note?: string | null; corrected: boolean };
		/** What the text pane holds, for screen readers. */
		label: string;
		/** The text pane's tools, at the end of its bar. */
		tools?: Snippet;
		children: Snippet;
		/** One line under the text: model, cost, scores. */
		foot?: Snippet;
		/** A panel under the text, scrolling on its own. */
		panel?: Snippet;
		textPane?: HTMLElement | null;
		/** A third pane: given only while it is open. */
		third?: Snippet;
		thirdLabel?: string;
		thirdBar?: Snippet;
		thirdPane?: HTMLElement | null;
		strip?: Snippet;
	} = $props();
	const m = messages();

	let width = $state(0);
	const comparing = $derived(third !== undefined);
	const cuts = $derived(layout.cuts(comparing));

	let findRow = $state<HTMLElement | null>(null);

	/** Opens the search, or brings the focus back to it, its words selected. */
	async function openSearch() {
		searching = true;
		await tick();
		const input = findRow?.querySelector('input');
		input?.focus();
		input?.select();
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.ctrlKey && e.key.toLowerCase() === 'f' && search) {
			e.preventDefault();
			void openSearch();
			return;
		}
		if (e.ctrlKey && e.key === 'Enter') {
			if (reviewed === null || !onreviewed) return;
			e.preventDefault();
			onreviewed();
			return;
		}
		if ((e.target as HTMLElement).closest('input, textarea, select')) return;
		const delta =
			e.key === 'PageDown' || (e.altKey && e.key === 'ArrowRight')
				? 1
				: e.key === 'PageUp' || (e.altKey && e.key === 'ArrowLeft')
					? -1
					: 0;
		if (delta !== 0 && onstep) {
			e.preventDefault();
			onstep(delta);
		}
	}
</script>

<svelte:window {onkeydown} />

<div class="review">
	<header class="masthead">
		<div class="crumbs">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })} title={m.review.back}
				>{jobName ?? m.review.title}</button
			>
			<span class="slash" aria-hidden="true">/</span>
			<h1 class="file mono">{fileName}</h1>
		</div>
		{#if pager}<div class="pager">{@render pager()}</div>{/if}
		<div class="actions">
			{#if search}
				<button
					class="btn icon toggle"
					aria-pressed={searching}
					aria-label={m.review.search}
					title="{m.review.search} ({m.review.searchShortcut})"
					onclick={() => (searching ? (searching = false) : openSearch())}
					><Icon name="search" size={14} /></button
				>
			{/if}
			{@render actions?.()}
			{#if reviewed !== null && onreviewed}
				{#if reviewed}
					<button
						class="btn"
						aria-pressed="true"
						title={m.review.markNotReviewed}
						onclick={onreviewed}
						><Icon name="check" size={14} />{m.review.reviewed}<span class="kbd"
							>{m.review.reviewedShortcut}</span
						></button
					>
				{:else}
					<button class="btn primary" aria-pressed="false" onclick={onreviewed}
						><Icon name="check" size={14} />{m.review.markReviewed}<span class="kbd"
							>{m.review.reviewedShortcut}</span
						></button
					>
				{/if}
			{/if}
		</div>
	</header>

	{#if search && searching}<div bind:this={findRow}>{@render search()}</div>{/if}

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="work" class:comparing style:--columns={columns(cuts)} bind:clientWidth={width}>
		<SourcePane kind={source} {piece} {paging} {onstep} {preview} {text} {find} {player} />

		<Splitter
			label={m.review.resizeSource}
			position={cuts[0]}
			onset={(p) => layout.move(0, p, width, comparing)}
			onreset={() => layout.reset(0, width, comparing)}
		/>

		<section class="pane text" aria-label={label}>
			<div class="pane-bar">
				<span class="witness">
					{#if witness.siglum}<i class="siglum">{witness.siglum}</i>{/if}
					<span class="mono">{witness.label}</span>
					{#if witness.note}<span class="muted">{witness.note}</span>{/if}
					{#if witness.corrected}<span class="chip hand">{m.review.corrected}</span>{/if}
				</span>
				{#if tools}<span class="tools">{@render tools()}</span>{/if}
			</div>
			<div class="page" bind:this={textPane}>{@render children()}</div>
			{#if foot}<div class="foot">{@render foot()}</div>{/if}
			{#if panel}<div class="panel">{@render panel()}</div>{/if}
		</section>

		{#if third}
			<Splitter
				label={m.review.resizeCompare}
				position={cuts[1]}
				onset={(p) => layout.move(1, p, width, true)}
				onreset={() => layout.reset(1, width, true)}
			/>
			<section class="pane third" aria-label={thirdLabel}>
				<div class="pane-bar">{@render thirdBar?.()}</div>
				<div class="page" bind:this={thirdPane}>{@render third()}</div>
			</section>
		{/if}
	</div>

	{@render strip?.()}
</div>

<style>
	.review {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.masthead .crumbs {
		flex: 1 1 16rem;
		align-items: baseline;
		flex-wrap: nowrap;
		gap: var(--space-2);
		min-width: 0;
	}
	.crumbs .back {
		font-size: var(--type-md);
		font-weight: 600;
		color: var(--text-bright);
		white-space: nowrap;
	}
	.crumbs .back:hover {
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.slash {
		color: var(--text-faint);
	}
	.file {
		font-size: var(--type-sm);
		font-weight: 400;
		letter-spacing: 0;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pager {
		display: flex;
		gap: var(--space-1);
		align-items: center;
		flex: none;
	}
	.actions {
		flex: 1 1 16rem;
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	/* A view that is on reads as pressed in, not as an action. */
	.toggle[aria-pressed='true'] {
		background: var(--wash);
		border-color: var(--line-strong);
	}
	.error {
		color: var(--danger-text);
		padding: var(--space-2) var(--space-5);
	}
	.work {
		--columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
		flex: 1;
		display: grid;
		grid-template-columns: var(--columns);
		min-height: 0;
		position: relative;
	}
	.text {
		background: var(--paper);
	}
	.witness {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.tools {
		margin-left: auto;
		display: inline-flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The text or answers being checked. */
	.page {
		position: relative;
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-6) var(--space-7);
	}
	.foot {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2) var(--space-4);
		min-height: 2.25rem;
		padding: var(--space-1) var(--space-4);
		border-top: 1px solid var(--line-soft);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.panel {
		display: grid;
		gap: var(--space-1);
		max-height: 11rem;
		overflow: auto;
		padding: var(--space-3) var(--space-6);
		border-top: 1px solid var(--line);
		background: var(--surface-1);
	}
	.third {
		border-left: 1px solid var(--line);
		background: var(--paper);
	}
	/* A narrow window: the panes stack, at fixed shares, with no handles between them. */
	@media (max-width: 1000px) {
		.work {
			grid-template-columns: 1fr;
			grid-template-rows: minmax(16rem, 42vh) minmax(0, 1fr);
		}
		.work.comparing {
			grid-template-rows: minmax(14rem, 34vh) minmax(0, 1fr) minmax(0, 1fr);
		}
		.work :global(.splitter) {
			display: none;
		}
		.third {
			border-left: 0;
			border-top: 1px solid var(--line);
		}
	}
	@media (max-width: 820px) {
		.page {
			padding: var(--space-4) var(--space-4) var(--space-6);
		}
		.panel {
			padding: var(--space-3) var(--space-4);
		}
	}
</style>
