<script lang="ts">
	// The review of a table of contents: the scan on its lightbox beside the entries, the strip of
	// pages below. Clicking an entry shows its first page; turning the page brings its entries
	// into view, marked. Under the entries, what the first step read on that page, to check an
	// entry against its source. The entries can be corrected, added and removed; the model's own
	// stay kept apart. A search keeps the entries that hold its words, marked where they stand.
	import { onMount, tick } from 'svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import FieldInput from './FieldInput.svelte';
	import Icon from './Icon.svelte';
	import PageStrip from './PageStrip.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import SearchBar from './SearchBar.svelte';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import {
		asAnswers,
		entriesOf,
		entryFields,
		holds,
		PAGES,
		pagesOf,
		TITLE,
		valueText
	} from '#lib/contents.js';
	import { type Draft, fromDraft, toDraft } from '#lib/fields.js';
	import { clearFound, markFound } from '#lib/found.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { pieceLabel, pieceNumber } from '#lib/pieces.js';
	import { Review } from '#lib/review.svelte.js';
	import { searchable } from '#lib/search.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();
	// svelte-ignore state_referenced_locally
	const review = new Review(jobId, sourceId);

	/** The page shown, as an index among the source's pages. */
	let page = $state(0);
	/** The entries being corrected, one draft each. */
	let editing = $state<Draft[] | null>(null);

	const setup = $derived(review.setups[0]);
	const fields = $derived(entryFields(setup));
	const details = $derived(fields.filter((f) => f.key !== TITLE && f.key !== PAGES));
	const mine = $derived(review.views.filter((v) => v.setup_id === setup?.id));
	const pageViews = $derived(mine.filter((v) => v.piece.kind === 'page'));
	const combined = $derived(mine.find((v) => v.piece.kind === 'whole'));
	const entries = $derived(entriesOf(combined?.corrected_fields ?? combined?.ai_fields ?? []));

	let searching = $state(false);
	let query = $state('');
	$effect(() => {
		if (!searching) query = '';
	});
	const filtering = $derived(searching && searchable(query));
	/** The entries listed, each with its number in the table: those holding the words searched
	 * for, or all of them. */
	const shown = $derived(
		entries
			.map((entry, i) => ({ entry, n: i + 1 }))
			.filter(({ entry }) => !filtering || holds(entry, query))
	);
	const shownPage = $derived(pageViews[page]);
	const pageNumber = $derived(shownPage?.piece.kind === 'page' ? shownPage.piece.number : page + 1);
	/** What the first step read on the page shown. */
	const readHere = $derived(entriesOf(shownPage?.ai_fields ?? []));
	const noteHere = $derived((shownPage?.ai_fields ?? []).find((a) => a.key === 'page_note')?.value);
	const cost = $derived(mine.reduce((sum, v) => sum + (v.cost_usd ?? 0), 0));
	const done = $derived(combined?.status === 'done' || combined?.status === 'cached');

	$effect(() => review.showPage(page));

	function step(delta: number) {
		page = Math.min(Math.max(page + delta, 0), Math.max(pageViews.length - 1, 0));
	}

	/** Shows the first page an entry names. */
	function showEntry(entry: FieldAnswer[]) {
		const first = pagesOf(valueText(entry, PAGES))[0];
		if (first === undefined) return;
		const index = pageViews.findIndex((v) => v.piece.kind === 'page' && v.piece.number === first);
		if (index >= 0 && index !== page) {
			// The entry clicked is already in view: the list stays where it is.
			keepList = true;
			page = index;
		}
	}

	const onPage = (entry: FieldAnswer[]) => pagesOf(valueText(entry, PAGES)).includes(pageNumber);

	let list = $state<HTMLElement | null>(null);
	/** Set when the page changes from an entry, which needs no scrolling to. */
	let keepList = false;

	// A new page brings its first entry to the top of the list.
	$effect(() => {
		const here = shown.findIndex(({ entry }) => onPage(entry));
		if (keepList) {
			keepList = false;
			return;
		}
		if (here < 0 || !list) return;
		void tick().then(() => {
			const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
			list?.children[here]?.scrollIntoView({
				block: 'start',
				behavior: still ? 'auto' : 'smooth'
			});
		});
	});

	// The words searched for, marked in the entries listed.
	$effect(() => {
		void shown;
		if (!list || !filtering) return;
		markFound(list, '.entry-title, .entry-pages, dd', query);
		return clearFound;
	});

	function addEntry() {
		const draft = toDraft(fields, []);
		draft[PAGES] = String(pageNumber);
		editing = [...(editing ?? []), draft];
	}

	async function save(answers: FieldAnswer[] | null, reviewed: boolean) {
		if (!setup || !combined) return;
		const index = combined.index;
		if (await review.save({ setup, index, fields: answers, reviewed })) editing = null;
	}

	onMount(() => {
		void review.load();
	});
</script>

{#snippet actions()}
	{#if done && !editing}
		<button class="btn" onclick={() => (editing = entries.map((e) => toDraft(fields, e)))}
			><Icon name="edit" size={14} />{m.review.editEntries}</button
		>
	{/if}
{/snippet}

{#snippet tools()}
	{filtering
		? m.review.entriesFound(shown.length, entries.length)
		: m.review.entries(entries.length)}
{/snippet}

{#snippet panel()}
	<h2 class="label">{m.review.pageReading} · {m.review.page(String(pageNumber))}</h2>
	{#if readHere.length > 0}
		<ul class="read">
			{#each readHere as entry, i (i)}
				<li>
					<span>{valueText(entry, TITLE)}</span>
					{#if valueText(entry, 'continues')}<span class="muted"
							>· {valueText(entry, 'continues')}</span
						>{/if}
				</li>
			{/each}
		</ul>
	{:else if shownPage?.status === 'failed'}
		<p class="error small">{shownPage.error}</p>
	{/if}
	{#if typeof noteHere === 'string' && noteHere}
		<p class="muted small">{m.review.pageNote}: {noteHere}</p>
	{/if}
	{#if cost > 0}<p class="muted small mono">{formatUsd(cost)}</p>{/if}
{/snippet}

{#snippet strip()}
	{#if pageViews.length > 1}
		<PageStrip
			{jobId}
			{sourceId}
			paged
			current={shownPage?.index ?? -1}
			items={pageViews.map((v, i) => ({
				index: v.index,
				label: pieceLabel(v.piece, m),
				number: pieceNumber(v.piece, i, m),
				done: false
			}))}
			onselect={(index) => {
				const i = pageViews.findIndex((v) => v.index === index);
				if (i >= 0) page = i;
			}}
		/>
	{/if}
{/snippet}

{#snippet searchBar()}
	<SearchBar
		bind:query
		placeholder={m.review.searchEntries}
		summary={!filtering
			? ''
			: shown.length === 0
				? m.review.notFound
				: m.review.entriesFound(shown.length, entries.length)}
		onclose={() => (searching = false)}
	/>
{/snippet}

<ReviewFrame
	{jobId}
	jobName={review.job?.name ?? null}
	fileName={review.source?.file_name ?? ''}
	error={review.error}
	{actions}
	search={searchBar}
	bind:searching
	reviewed={combined && done && !editing ? combined.reviewed : null}
	onreviewed={() => combined && save(combined.corrected_fields, !combined.reviewed)}
	onstep={step}
	source="scan"
	piece={shownPage ? pieceLabel(shownPage.piece, m) : m.review.page(String(pageNumber))}
	paging={{ at: page, count: pageViews.length }}
	preview={review.preview}
	witness={{
		label: setup?.label ?? '',
		note: setup?.merge && setup.merge.model !== setup.model ? `+ ${setup.merge.model}` : null,
		corrected: combined?.corrected_fields != null
	}}
	label={m.review.contentsTitle}
	{tools}
	{panel}
	{strip}
>
	{#if editing}
		<CorrectionEditor
			label={setup?.label ?? ''}
			onsave={() => save(asAnswers((editing ?? []).map((d) => fromDraft(fields, d))), true)}
			ondiscard={() => (editing = null)}
			onrevert={combined?.corrected_fields
				? () => save(null, combined?.reviewed ?? false)
				: undefined}
		>
			{#each editing as draft, i (i)}
				<div class="entry-edit">
					{#each fields as f (f.key)}
						<div class="field">
							<span class="field-label" aria-hidden="true">{f.label}</span>
							<FieldInput
								field={f}
								bind:value={draft[f.key]}
								line={f.kind.type === 'text' && (f.key === TITLE || f.key === PAGES)}
							/>
						</div>
					{/each}
					<button
						class="btn ghost sm remove"
						onclick={() => (editing = editing?.filter((_, j) => j !== i) ?? null)}
						>{m.review.removeEntry}</button
					>
				</div>
			{/each}
			<button class="btn add" onclick={addEntry}
				><Icon name="plus" size={14} />{m.review.addEntry}</button
			>
		</CorrectionEditor>
	{:else if combined?.status === 'failed'}
		<p class="error">{combined.error}</p>
	{:else if !combined || entries.length === 0}
		<p class="muted">{m.review.notCombined}</p>
	{:else}
		{#if shown.length === 0}<p class="muted">{m.review.noEntryFound}</p>{/if}
		<ol class="entries" bind:this={list}>
			{#each shown as { entry, n } (n)}
				<li value={n} class:here={onPage(entry)}>
					<button class="entry" onclick={() => showEntry(entry)}>
						<span class="entry-title">{valueText(entry, TITLE)}</span>
						<span class="entry-pages mono" title={m.review.goToPage(valueText(entry, PAGES))}
							>{valueText(entry, PAGES)}</span
						>
					</button>
					<dl>
						{#each details as f (f.key)}
							{@const v = valueText(entry, f.key)}
							{#if v}
								<div class="detail">
									<dt>{f.label}</dt>
									<dd>{v}</dd>
								</div>
							{/if}
						{/each}
					</dl>
				</li>
			{/each}
		</ol>
	{/if}
</ReviewFrame>

<style>
	.error {
		color: var(--danger-text);
	}
	/* The entries as a numbered table of contents, set in the reading face. */
	.entries {
		margin: 0;
		padding: 0 0 0 1.75rem;
		font-family: var(--font-text);
		color: var(--text-body);
	}
	.entries li {
		padding: var(--space-3) var(--space-3) var(--space-3) var(--space-2);
		border-bottom: 1px solid var(--line-soft);
		border-radius: var(--radius-control);
		scroll-margin-top: var(--space-3);
	}
	.entries li::marker {
		font-family: var(--font-mono);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The entries on the page shown, marked as with the highlighter's edge. */
	.entries li.here {
		background: var(--surface-1);
		box-shadow: inset 3px 0 0 var(--marker);
	}
	.entry {
		display: flex;
		align-items: baseline;
		gap: var(--space-3);
		width: 100%;
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.entry-title {
		flex: 1;
		font-size: var(--type-lg);
		font-weight: 600;
		line-height: 1.35;
		color: var(--text-bright);
	}
	.entry:hover .entry-title {
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.entry-pages {
		flex: none;
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	dl {
		display: grid;
		gap: var(--space-1);
		margin: var(--space-2) 0 0;
	}
	.detail {
		display: grid;
		grid-template-columns: minmax(5rem, 9rem) minmax(0, 1fr);
		gap: var(--space-3);
		font-size: var(--type-sm);
		line-height: 1.5;
	}
	.detail dt {
		font-family: var(--font-ui);
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-secondary);
		padding-top: 0.15em;
	}
	.detail dd {
		margin: 0;
	}
	.label {
		margin: 0;
	}
	.read {
		display: grid;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	.small {
		font-size: var(--type-xs);
	}
	.entry-edit {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) 0;
		border-bottom: 1px solid var(--line);
	}
	.field {
		display: grid;
		gap: var(--space-1);
	}
	.field-label {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-secondary);
	}
	.entry-edit .remove {
		justify-self: start;
	}
	.add {
		margin-top: var(--space-3);
		border-style: dashed;
	}
</style>
