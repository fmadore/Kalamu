<script lang="ts">
	// The review of a table of contents: the scan on its lightbox beside the entries, the strip of
	// pages below. Clicking an entry shows its first page; turning the page brings its entries
	// into view, marked. Under the entries, what the first step read on that page, to check an
	// entry against its source. The entries can be corrected, added and removed; the model's own
	// stay kept apart.
	import { onMount, tick } from 'svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
	import PageStrip from './PageStrip.svelte';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import {
		asAnswers,
		type Draft,
		entriesOf,
		entryFields,
		fromDraft,
		PAGES,
		pagesOf,
		TITLE,
		toDraft,
		valueText
	} from '#lib/contents.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let views = $state<PieceView[]>([]);
	let preview = $state<PagePreview | null>(null);
	/** The page shown, as an index among the source's pages. */
	let page = $state(0);
	let zoom = $state(1);
	let error = $state<string | null>(null);
	/** The entries being corrected, one draft each. */
	let editing = $state<Draft[] | null>(null);

	const source = $derived(job?.sources.find((s) => s.id === sourceId));
	const setup = $derived(job?.setups[0]);
	const fields = $derived(entryFields(setup));
	const details = $derived(fields.filter((f) => f.key !== TITLE && f.key !== PAGES));
	const mine = $derived(views.filter((v) => v.setup_id === setup?.id));
	const pageViews = $derived(mine.filter((v) => v.piece.kind === 'page'));
	const combined = $derived(mine.find((v) => v.piece.kind === 'whole'));
	const entries = $derived(entriesOf(combined?.corrected_fields ?? combined?.ai_fields ?? []));
	const shownPage = $derived(pageViews[page]);
	const pageNumber = $derived(shownPage?.piece.kind === 'page' ? shownPage.piece.number : page + 1);
	/** What the first step read on the page shown. */
	const readHere = $derived(entriesOf(shownPage?.ai_fields ?? []));
	const noteHere = $derived((shownPage?.ai_fields ?? []).find((a) => a.key === 'page_note')?.value);
	const cost = $derived(mine.reduce((sum, v) => sum + (v.cost_usd ?? 0), 0));

	async function load() {
		try {
			job = await api.getJob(jobId);
			views = await api.sourcePieces(jobId, sourceId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	$effect(() => {
		const p = page;
		preview = null;
		api
			.piecePreview(jobId, sourceId, p)
			.then((r) => {
				if (page === p) preview = r;
			})
			.catch(() => (preview = { mime: 'none', data: '' }));
	});

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
		const here = entries.findIndex(onPage);
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

	const stripItems = $derived(
		pageViews.map((v, i) => {
			const number = v.piece.kind === 'page' ? v.piece.number : i + 1;
			const half = v.piece.kind === 'page' && v.piece.half ? v.piece.half : null;
			return {
				index: v.index,
				label: m.review.page(half ? `${number} (${half})` : String(number)),
				number: `${number}${half ? half[0] : ''}`,
				done: false
			};
		})
	);

	function startEditing() {
		editing = entries.map((e) => toDraft(fields, e));
	}

	function addEntry() {
		const draft = toDraft(fields, []);
		draft[PAGES] = String(pageNumber);
		editing = [...(editing ?? []), draft];
	}

	async function save(answers: FieldAnswer[] | null, reviewed: boolean) {
		if (!setup || !combined) return;
		error = null;
		try {
			await api.saveCorrection({
				job_id: jobId,
				setup_id: setup.id,
				source_id: sourceId,
				index: combined.index,
				text: null,
				fields: answers ?? undefined,
				reviewed,
				updated_at: 0
			});
			editing = null;
			views = await api.sourcePieces(jobId, sourceId);
			toast(m.review.saved);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	function onKey(e: KeyboardEvent) {
		if (editing || (e.target as HTMLElement).closest('input, textarea, select')) return;
		if (e.key === 'ArrowRight' && page < pageViews.length - 1) page += 1;
		if (e.key === 'ArrowLeft' && page > 0) page -= 1;
	}

	onMount(() => {
		void load();
	});
</script>

<svelte:window onkeydown={onKey} />

<div class="review">
	<header class="masthead">
		<div class="crumbs">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })} title={m.review.back}
				>{job?.name ?? m.review.title}</button
			>
			<span class="slash" aria-hidden="true">/</span>
			<h1 class="file mono">{source?.file_name ?? ''}</h1>
		</div>
		<div class="actions">
			{#if combined?.status === 'done' || combined?.status === 'cached'}
				{#if !editing}
					<button class="btn" onclick={startEditing}
						><Icon name="edit" size={14} />{m.review.editEntries}</button
					>
					{#if combined.reviewed}
						<button
							class="btn"
							aria-pressed="true"
							title={m.review.markNotReviewed}
							onclick={() => save(combined.corrected_fields, false)}
							><Icon name="check" size={14} />{m.review.reviewed}</button
						>
					{:else}
						<button
							class="btn primary"
							aria-pressed="false"
							onclick={() => save(combined.corrected_fields, true)}
							><Icon name="check" size={14} />{m.review.markReviewed}</button
						>
					{/if}
				{/if}
			{/if}
		</div>
	</header>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="work">
		<section class="pane source stage" aria-label={m.review.source}>
			<div class="bar">
				<span class="bar-title mono">{m.review.source} · {m.review.page(String(pageNumber))}</span>
				{#if pageViews.length > 1}
					<span class="paging">
						<button
							class="zoom-btn"
							aria-label={m.review.previous}
							disabled={page === 0}
							onclick={() => (page -= 1)}><Icon name="previous" /></button
						>
						<span class="mono count">{page + 1} / {pageViews.length}</span>
						<button
							class="zoom-btn"
							aria-label={m.review.next}
							disabled={page >= pageViews.length - 1}
							onclick={() => (page += 1)}><Icon name="chevron" /></button
						>
					</span>
				{/if}
			</div>
			<div class="desk"><PageImage {preview} {zoom} /></div>
			<div class="zoom">
				<button
					class="zoom-btn"
					aria-label={m.review.zoomOut}
					title={m.review.zoomOut}
					onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}><Icon name="zoom-out" /></button
				>
				<span class="zoom-value mono">{Math.round(zoom * 100)}%</span>
				<button
					class="zoom-btn"
					aria-label={m.review.zoomIn}
					title={m.review.zoomIn}
					onclick={() => (zoom = Math.min(3, zoom + 0.2))}><Icon name="zoom-in" /></button
				>
			</div>
		</section>

		<section class="pane edition" aria-label={m.review.contentsTitle}>
			<div class="bar">
				<span class="witness-head">
					<span class="mono">{setup?.label ?? ''}</span>
					{#if setup?.merge && setup.merge.model !== setup.model}
						<span class="muted">+ {setup.merge.model}</span>
					{/if}
					{#if combined?.corrected_fields}<span class="chip hand">{m.review.corrected}</span>{/if}
				</span>
				<span class="preset">{m.review.entries(entries.length)}</span>
			</div>

			<div class="page">
				{#if editing}
					<p class="editing-of">{m.review.correcting} <span class="mono">{setup?.label}</span></p>
					{#each editing as draft, i (i)}
						<div class="entry-edit">
							{#each fields as f (f.key)}
								<label class="field">
									<span>{f.label}</span>
									{#if f.kind.type === 'choice'}
										<select bind:value={draft[f.key]}>
											{#each f.kind.values as v (v)}<option value={v}>{v}</option>{/each}
										</select>
									{:else if f.kind.type === 'text' && (f.key === TITLE || f.key === PAGES)}
										<input type="text" bind:value={draft[f.key]} />
									{:else}
										<textarea
											class="editor"
											class:short={f.kind.type !== 'text'}
											bind:value={draft[f.key]}></textarea>
									{/if}
								</label>
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
					<div class="edit-actions">
						<button
							class="btn primary"
							onclick={() =>
								save(asAnswers((editing ?? []).map((d) => fromDraft(fields, d))), true)}
							>{m.review.save}</button
						>
						<button class="btn ghost" onclick={() => (editing = null)}>{m.review.discard}</button>
						{#if combined?.corrected_fields}
							<button class="btn ghost" onclick={() => save(null, combined?.reviewed ?? false)}
								>{m.review.revert}</button
							>
						{/if}
					</div>
				{:else if combined?.status === 'failed'}
					<p class="error-text">{combined.error}</p>
				{:else if !combined || entries.length === 0}
					<p class="muted">{m.review.notCombined}</p>
				{:else}
					<ol class="entries" bind:this={list}>
						{#each entries as entry, i (i)}
							<li class:here={onPage(entry)}>
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
			</div>

			<div class="foot">
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
					<p class="error-text small">{shownPage.error}</p>
				{/if}
				{#if typeof noteHere === 'string' && noteHere}
					<p class="muted small">{m.review.pageNote}: {noteHere}</p>
				{/if}
				{#if cost > 0}<p class="muted small mono">{formatUsd(cost)}</p>{/if}
			</div>
		</section>
	</div>

	{#if pageViews.length > 1}
		<PageStrip
			{jobId}
			{sourceId}
			paged
			current={shownPage?.index ?? -1}
			items={stripItems}
			onselect={(index) => {
				const i = pageViews.findIndex((v) => v.index === index);
				if (i >= 0) page = i;
			}}
		/>
	{/if}
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
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.error {
		color: var(--danger-text);
		padding: var(--space-2) var(--space-5);
	}
	.error-text {
		color: var(--danger-text);
	}
	.work {
		flex: 1;
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		min-height: 0;
	}
	.pane {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-height: 0;
		position: relative;
	}
	.bar {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		min-height: 2.75rem;
		padding: var(--space-1) var(--space-4);
		border-bottom: 1px solid var(--line-soft);
	}
	.bar-title {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The lightbox under a scan, as in the text review. */
	.stage {
		background: var(--stage);
		border-right: 1px solid var(--stage-line);
	}
	.stage .bar {
		border-bottom-color: var(--stage-line);
	}
	.stage .bar-title {
		color: var(--on-stage-muted);
	}
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
	.desk {
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-5) 4.5rem;
	}
	.stage .desk :global(.muted) {
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
	.zoom-btn {
		display: grid;
		place-items: center;
		min-width: var(--control-sm);
		height: var(--control-sm);
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.zoom-btn:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.08);
	}
	.zoom-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.zoom-btn:focus-visible {
		outline-color: var(--on-stage);
	}
	.zoom-value {
		min-width: 3rem;
		text-align: center;
		font-size: var(--type-xs);
	}
	.edition {
		grid-template-rows: auto minmax(0, 1fr) auto;
		background: var(--paper);
	}
	.witness-head {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.preset {
		margin-left: auto;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.page {
		overflow: auto;
		min-height: 0;
		padding: var(--space-4) var(--space-6) var(--space-6);
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
	.foot {
		display: grid;
		gap: var(--space-1);
		padding: var(--space-3) var(--space-6);
		border-top: 1px solid var(--line);
		background: var(--surface-1);
		max-height: 11rem;
		overflow: auto;
	}
	.foot h2 {
		margin: 0;
	}
	.read {
		display: grid;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: var(--type-sm);
	}
	.small {
		font-size: var(--type-xs);
	}
	.editing-of {
		margin: 0 0 var(--space-3);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.entry-edit {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) 0;
		border-bottom: 1px solid var(--line);
	}
	.entry-edit .remove {
		justify-self: start;
	}
	.editor {
		min-height: 5rem;
		font-family: var(--font-text);
	}
	.editor.short {
		min-height: 3rem;
	}
	.add {
		margin-top: var(--space-3);
		border-style: dashed;
	}
	.edit-actions {
		display: flex;
		gap: var(--space-2);
		margin-top: var(--space-4);
	}
	@media (max-width: 900px) {
		.work {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: minmax(16rem, 45vh) minmax(0, 1fr);
		}
		.stage {
			border-right: 0;
		}
	}
</style>
