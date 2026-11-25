<script lang="ts">
	// A project's search by meaning: a question or a description, the documents closest to it
	// listed by their best passage, each passage opening the review where it begins. The same
	// filters as the search by words; what was compared said with every search. No answer, no
	// summary. The set-up (model, consent, estimate, preparation) sits above it, folded away once
	// the text is prepared. The question is embedded and compared in Rust.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import MeaningSetup from './MeaningSetup.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { MeaningDocument } from '#lib/bindings/MeaningDocument.js';
	import type { MeaningPassage } from '#lib/bindings/MeaningPassage.js';
	import type { MeaningRequest } from '#lib/bindings/MeaningRequest.js';
	import type { MeaningStatus } from '#lib/bindings/MeaningStatus.js';
	import type { MeaningView } from '#lib/bindings/MeaningView.js';
	import type { Neighbour } from '#lib/bindings/Neighbour.js';
	import type { ReviewFilter } from '#lib/bindings/ReviewFilter.js';
	import type { TagView } from '#lib/bindings/TagView.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		blankMeaningRequest,
		canAsk,
		meaningFind,
		meaningStep,
		similarityText
	} from '#lib/meaning.js';
	import { TEXT_TASKS } from '#lib/profiles.js';
	import { hitPlace } from '#lib/projectSearch.js';
	import { go } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		collections,
		tags
	}: { projectId: string; collections: CollectionView[]; tags: TagView[] } = $props();
	const m = messages();
	const REVIEW: ReviewFilter[] = ['any', 'reviewed', 'not_reviewed'];

	let status = $state<MeaningStatus | null>(null);
	let request = $state<MeaningRequest>(blankMeaningRequest());
	let view = $state<MeaningView | null>(null);
	let error = $state<string | null>(null);
	let busy = $state(false);
	let showSetup = $state<boolean | null>(null);
	/** The documents like one listed, by document, once asked for. */
	let alike = $state<Record<string, Neighbour[] | 'loading'>>({});

	const ready = $derived(!!status && canAsk(status) && request.query.trim().length > 0);
	/** The set-up open unless everything is prepared, or as the researcher left it. */
	const setupOpen = $derived(
		showSetup ?? (!status || meaningStep(status) !== 'ready' || !canAsk(status))
	);

	async function reload() {
		try {
			status = await api.meaningStatus(projectId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	let ticket = 0;
	async function run() {
		if (!ready) return;
		const asked = $state.snapshot(request);
		const mine = ++ticket;
		busy = true;
		error = null;
		try {
			const found = await api.meaningSearch(projectId, asked);
			if (mine !== ticket) return;
			view = found;
			alike = {};
		} catch (e) {
			if (mine === ticket) error = describeError(e, m);
		} finally {
			if (mine === ticket) busy = false;
		}
		// What a question cost is part of the record.
		void reload();
	}

	function refilter() {
		if (view) void run();
	}

	function open(p: MeaningPassage) {
		go({ name: 'review', jobId: p.job_id, sourceId: p.source_id, find: meaningFind(p) });
	}

	async function like(d: MeaningDocument) {
		if (alike[d.document_id]) {
			const { [d.document_id]: _, ...rest } = alike;
			alike = rest;
			return;
		}
		alike = { ...alike, [d.document_id]: 'loading' };
		try {
			alike = { ...alike, [d.document_id]: await api.meaningNeighbours(projectId, d.document_id) };
		} catch (e) {
			error = describeError(e, m);
		}
	}

	const resultName = (p: MeaningPassage) =>
		[m.task[p.task], p.setup_label, p.job_name !== p.setup_label ? p.job_name : null]
			.filter(Boolean)
			.join(' · ');
	const sim = (v: number) => similarityText(v, m.locale.tag);

	onMount(() => {
		void reload();
	});
</script>

<div class="meaning">
	<p class="lede muted">{m.meaning.lede}</p>

	{#if status}
		{#if meaningStep(status) !== 'no_models'}
			<button
				class="btn ghost sm toggle"
				aria-expanded={setupOpen}
				onclick={() => (showSetup = !setupOpen)}
			>
				<Icon name={setupOpen ? 'expand' : 'chevron'} size={14} />
				{setupOpen ? m.meaning.setup.hide : m.meaning.setup.show}
			</button>
		{/if}
		{#if setupOpen || meaningStep(status) === 'no_models'}
			<MeaningSetup {projectId} {status} {reload} />
		{/if}
	{/if}

	<form
		class="query"
		role="search"
		onsubmit={(e) => {
			e.preventDefault();
			void run();
		}}
	>
		<label class="table-search">
			<Icon name="search" size={14} />
			<input
				type="search"
				bind:value={request.query}
				data-find
				placeholder={m.meaning.placeholder}
				aria-label={m.meaning.query}
				disabled={!status || !canAsk(status)}
			/>
		</label>
		<button class="btn primary sm" disabled={!ready || busy}>{m.meaning.go}</button>
	</form>

	<div class="table-filters">
		{#if collections.length > 0}
			<label class="pick">
				<span>{m.search.scope}</span>
				<select bind:value={request.collection} onchange={refilter}>
					<option value={null}>{m.search.wholeProject}</option>
					{#each collections as c (c.collection.id)}
						<option value={c.collection.id}>{c.collection.name}</option>
					{/each}
				</select>
			</label>
		{/if}
		{#if tags.length > 0}
			<label class="pick">
				<span>{m.search.tag}</span>
				<select bind:value={request.tag} onchange={refilter}>
					<option value={null}>{m.search.anyTag}</option>
					{#each tags as t (t.name)}
						<option value={t.name}>{t.name}</option>
					{/each}
				</select>
			</label>
		{/if}
		<label class="pick">
			<span>{m.search.task}</span>
			<select bind:value={request.task} onchange={refilter}>
				<option value={null}>{m.search.anyTask}</option>
				{#each TEXT_TASKS as t (t)}
					<option value={t}>{m.task[t]}</option>
				{/each}
			</select>
		</label>
		<label class="pick">
			<span>{m.search.review}</span>
			<select bind:value={request.review} onchange={refilter}>
				{#each REVIEW as r (r)}
					<option value={r}>{m.search.reviewFilter[r]}</option>
				{/each}
			</select>
		</label>
		<label class="pick">
			<span>{m.search.results}</span>
			<select bind:value={request.results} onchange={refilter}>
				<option value="preferred">{m.search.resultChoice.preferred}</option>
				<option value="all">{m.search.resultChoice.all}</option>
			</select>
		</label>
	</div>

	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if busy}<p class="muted" role="status">{m.meaning.searching}</p>{/if}

	{#if view}
		<div class="summary" aria-live="polite">
			<p class="num">
				{view.compared === 0
					? m.meaning.none
					: m.meaning.found(view.documents.length, view.compared)}
			</p>
			<ul class="coverage muted num">
				<li>{m.meaning.coverage.compared(view.coverage.compared, view.coverage.documents)}</li>
				{#if view.coverage.not_prepared > 0}
					<li>{m.meaning.coverage.notPrepared(view.coverage.not_prepared)}</li>
				{/if}
				{#if view.coverage.unread > 0}<li>{m.search.unread(view.coverage.unread)}</li>{/if}
				{#if view.coverage.pieces_missing > 0}
					<li>{m.meaning.coverage.missing(view.coverage.pieces_missing)}</li>
				{/if}
				{#if view.coverage.pieces_changed > 0}
					<li>{m.meaning.coverage.changed(view.coverage.pieces_changed)}</li>
				{/if}
				{#if view.coverage.pieces_unread > 0}
					<li>{m.search.piecesUnread(view.coverage.pieces_unread)}</li>
				{/if}
				{#if view.coverage.without_preference > 0}
					<li>{m.search.withoutPreference(view.coverage.without_preference)}</li>
				{/if}
				<li>{m.meaning.coverage.absence}</li>
			</ul>
			{#if view.documents.length > 0}
				<p class="hint muted">{m.meaning.similarityHint}</p>
			{/if}
		</div>

		<ol class="documents">
			{#each view.documents as d (d.document_id)}
				<li class="document">
					<h3>
						<button
							class="link"
							onclick={() => go({ name: 'projects', projectId, documentId: d.document_id })}
						>
							{d.title}
						</button>
						<span class="sim num" title={m.meaning.similarityHint}>
							{m.meaning.similarity}{m.common.colon}
							{sim(d.similarity)}
						</span>
					</h3>
					<ol class="hits">
						{#each d.passages as p (`${p.job_id}/${p.setup_id}/${p.hit.index}/${p.hit.nth}/${p.hit.term}`)}
							{@const place = hitPlace(p.hit, m)}
							<li>
								<button class="hit" aria-label={m.meaning.open(place)} onclick={() => open(p)}>
									<span class="place num">{place}</span>
									<span class="snippet own-direction">
										<span class="origin">{resultName(p)}</span>
										{p.hit.words}
									</span>
									<span class="state">
										<span class="sim num">{sim(p.similarity)}</span>
										{#if p.hit.corrected}<span class="chip hand">{m.review.corrected}</span>{/if}
										{#if p.hit.reviewed}<span class="chip ok">{m.review.reviewed}</span>{/if}
									</span>
								</button>
							</li>
						{/each}
					</ol>
					<button
						class="btn ghost sm"
						aria-expanded={!!alike[d.document_id]}
						onclick={() => like(d)}
					>
						{m.meaning.likeThis}
					</button>
					{#if alike[d.document_id] && alike[d.document_id] !== 'loading'}
						{@const near = alike[d.document_id] as Neighbour[]}
						{#if near.length === 0}
							<p class="muted small">{m.meaning.map.noNeighbours}</p>
						{:else}
							<ol class="alike">
								{#each near.slice(0, 5) as n (n.document_id)}
									<li>
										<button
											class="link"
											onclick={() => go({ name: 'projects', projectId, documentId: n.document_id })}
										>
											{n.title}
										</button>
										<span class="sim num">{sim(n.similarity)}</span>
									</li>
								{/each}
							</ol>
						{/if}
					{/if}
				</li>
			{/each}
		</ol>
	{/if}
</div>

<style>
	.meaning {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-4);
		align-content: start;
		min-width: 0;
	}
	.lede {
		max-width: 44rem;
		font-size: var(--type-sm);
	}
	.toggle {
		justify-self: start;
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
	}
	.query {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
	.query .table-search {
		max-width: 32rem;
	}
	.pick {
		display: inline-flex;
		flex-wrap: wrap;
		max-width: 100%;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.pick select {
		max-width: 100%;
	}
	.summary {
		display: grid;
		gap: var(--space-1);
	}
	.summary > p:first-child {
		color: var(--text-bright);
		font-weight: 500;
	}
	.coverage {
		margin: 0;
		padding-left: 1.1rem;
		font-size: var(--type-sm);
	}
	.hint,
	.small {
		max-width: 44rem;
		font-size: var(--type-xs);
	}
	.documents {
		display: grid;
		gap: var(--space-3);
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.document {
		display: grid;
		gap: var(--space-2);
		padding-top: var(--space-3);
		border-top: 1px solid var(--line);
	}
	.document h3 {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-3);
		font-size: var(--type-lg);
	}
	.document > .btn {
		justify-self: start;
	}
	.sim {
		color: var(--text-muted);
		font-size: var(--type-xs);
		font-weight: 400;
		white-space: nowrap;
	}
	.hits,
	.alike {
		display: grid;
		gap: 1px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.alike li {
		display: flex;
		gap: var(--space-3);
		align-items: baseline;
		padding: var(--space-1) var(--space-2);
		font-size: var(--type-sm);
	}
	.hit {
		display: grid;
		grid-template-columns: 6.5rem minmax(0, 1fr) auto;
		gap: var(--space-3);
		align-items: baseline;
		width: 100%;
		padding: var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: none;
		color: var(--text-secondary);
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
	}
	.hit:hover,
	.hit:focus-visible {
		background: var(--wash);
		color: var(--text-bright);
	}
	.place {
		color: var(--text-muted);
		white-space: nowrap;
	}
	.snippet {
		min-width: 0;
		overflow-wrap: anywhere;
		font-family: var(--font-text, inherit);
	}
	.origin {
		display: block;
		color: var(--text-muted);
		font-family: var(--font-ui, inherit);
		font-size: var(--type-xs);
	}
	.state {
		display: flex;
		gap: var(--space-1);
		align-items: baseline;
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font: inherit;
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	@media (max-width: 640px) {
		.hit {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
