<script lang="ts">
	// A project's search: every passage of its documents that holds some words, in the text as the
	// researcher corrected it, grouped by document and by reading, each opening its review where
	// it stands. What was searched is said beside the matches, so that finding nothing is not read
	// as absence. The search runs in Rust on a local copy of the results' text, prepared as soon as
	// the search opens, its progress shown.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { Hit } from '#lib/bindings/Hit.js';
	import type { MatchMode } from '#lib/bindings/MatchMode.js';
	import type { ResultHits } from '#lib/bindings/ResultHits.js';
	import type { ReviewFilter } from '#lib/bindings/ReviewFilter.js';
	import type { SearchRequest } from '#lib/bindings/SearchRequest.js';
	import type { SearchView } from '#lib/bindings/SearchView.js';
	import type { TagView } from '#lib/bindings/TagView.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { TEXT_TASKS } from '#lib/profiles.js';
	import {
		blankRequest,
		hitOrigin,
		hitPlace,
		hitWords,
		keepSearch,
		lastSearch,
		reviewFind,
		searchableQuery
	} from '#lib/projectSearch.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { api, listenWhileMounted, on } from '#lib/tauri.js';

	let {
		projectId,
		collections,
		tags
	}: { projectId: string; collections: CollectionView[]; tags: TagView[] } = $props();
	const m = messages();
	const GENERATED_TASKS: Task[] = ['analysis', 'contents'];
	const REVIEW: ReviewFilter[] = ['any', 'reviewed', 'not_reviewed'];
	const MODES: MatchMode[] = ['phrase', 'all_words', 'near'];

	// svelte-ignore state_referenced_locally
	const kept = lastSearch(projectId);
	let request = $state<SearchRequest>(kept?.request ?? blankRequest());
	let view = $state<SearchView | null>(kept?.view ?? null);
	let error = $state<string | null>(null);
	let busy = $state(false);
	/** How far the project's text is prepared for searching, while it is. */
	let preparing = $state<{ done: number; total: number } | null>(null);

	const tasks = $derived(request.generated ? [...TEXT_TASKS, ...GENERATED_TASKS] : TEXT_TASKS);
	const ready = $derived(searchableQuery(request.query, request.mode));

	/** Raised by each search, so only the newest one's answer, failure or end is shown. */
	let ticket = 0;

	async function run() {
		if (!ready) return;
		const asked = $state.snapshot(request);
		const mine = ++ticket;
		busy = true;
		error = null;
		try {
			const found = await api.projectSearch(projectId, asked);
			if (mine !== ticket) return;
			view = found;
			keepSearch(projectId, asked, found);
			preparing = null;
		} catch (e) {
			// A newer search replaced this one; its answer comes on its own.
			if (mine !== ticket || (e as { id?: string })?.id === 'search.stopped') return;
			error = describeError(e, m);
		} finally {
			if (mine === ticket) busy = false;
		}
	}

	// Typing searches once the words settle; a filter changed searches again at once.
	let timer: ReturnType<typeof setTimeout> | undefined;
	$effect(() => {
		void request.query;
		clearTimeout(timer);
		if (searchableQuery(request.query, request.mode)) timer = setTimeout(() => void run(), 400);
		return () => clearTimeout(timer);
	});

	function refilter() {
		// A task no longer offered once generated text is left out.
		if (request.task && !tasks.includes(request.task)) request.task = null;
		void run();
	}

	function open(result: ResultHits, hit: Hit) {
		go({
			name: 'review',
			jobId: result.job_id,
			sourceId: result.source_id,
			find: reviewFind(result, hit)
		});
	}

	// The text prepared before the first search, so that it need not wait; a search begun
	// meanwhile carries on from where the preparation got to.
	onMount(() => {
		const stop = listenWhileMounted(() =>
			on.searchProgress((p) => {
				if (p.project_id !== projectId) return;
				preparing = p.done < p.total ? { done: p.done, total: p.total } : null;
			})
		);
		void api
			.prepareProjectSearch(projectId)
			.catch(() => {
				// A search prepares what it needs itself, and says so if it cannot.
			})
			.finally(() => {
				if (!busy) preparing = null;
			});
		return stop;
	});

	async function rebuild() {
		error = null;
		try {
			await api.rebuildSearchIndex();
			toast(m.search.rebuilt);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	const resultName = (r: ResultHits) =>
		[m.task[r.task], r.setup_label, r.job_name !== r.setup_label ? r.job_name : null]
			.filter(Boolean)
			.join(' · ');
</script>

<div class="search-view">
	<p class="lede muted">{m.search.lede}</p>

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
				placeholder={m.search.placeholder}
				aria-label={m.search.query}
			/>
		</label>
		<button class="btn primary sm" disabled={!ready || busy}>{m.search.go}</button>
	</form>

	<div class="table-filters">
		<label class="pick">
			<span>{m.search.mode}</span>
			<select bind:value={request.mode} onchange={refilter}>
				{#each MODES as mode (mode)}
					<option value={mode}>{m.search.modes[mode]}</option>
				{/each}
			</select>
		</label>
		{#if request.mode === 'near'}
			<label class="pick">
				<span>{m.search.near}</span>
				<input
					class="distance num"
					type="number"
					min="0"
					max="100"
					bind:value={request.near}
					onchange={() => {
						request.near = Math.min(Math.max(Math.round(request.near) || 0, 0), 100);
						refilter();
					}}
				/>
				<span>{m.search.nearWords}</span>
			</label>
		{/if}
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
				{#each tasks as t (t)}
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
		<label class="check">
			<input type="checkbox" bind:checked={request.generated} onchange={refilter} />
			<span>{m.search.generated}</span>
		</label>
	</div>

	{#if request.mode !== 'phrase'}<p class="hint muted">{m.search.quotesHint}</p>{/if}
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if request.query.trim() && !ready}<p class="muted">{m.search.tooShort}</p>{/if}
	{#if preparing}
		<div class="preparing" role="status">
			<p class="muted num">{m.search.preparing(preparing.done, preparing.total)}</p>
			<ProgressBar done={preparing.done} total={preparing.total} label={m.search.title} />
		</div>
	{:else if busy}
		<p class="muted" role="status">{m.search.searching}</p>
	{/if}

	{#if view}
		<div class="summary" aria-live="polite">
			<p class="num">
				{view.matches === 0 ? m.search.none : m.search.matches(view.matches, view.documents.length)}
			</p>
			{#if view.listed < view.matches}
				<p class="muted">{m.search.listedOnly(view.listed, view.matches)}</p>
			{/if}
			{#if view.ignored.length > 0}
				<p class="muted">{m.search.ignored(view.ignored.join(', '))}</p>
			{/if}
			<ul class="coverage muted num">
				<li>{m.search.coverage(view.coverage.searched, view.coverage.documents)}</li>
				{#if view.coverage.unread > 0}<li>{m.search.unread(view.coverage.unread)}</li>{/if}
				{#if view.coverage.pieces_unread > 0}
					<li>{m.search.piecesUnread(view.coverage.pieces_unread)}</li>
				{/if}
				{#if view.coverage.without_preference > 0}
					<li>{m.search.withoutPreference(view.coverage.without_preference)}</li>
				{/if}
				{#if view.matches === 0}<li>{m.search.absence}</li>{/if}
			</ul>
		</div>

		{#each view.documents as d (d.document_id)}
			<section class="document">
				<h3>
					<button
						class="link"
						onclick={() => go({ name: 'projects', projectId, documentId: d.document_id })}
						>{d.title}</button
					>
				</h3>
				{#each d.results as r (r.job_id + r.setup_id)}
					<div class="result">
						<p class="result-name">
							<span>{resultName(r)}</span>
							{#if r.preferred}<span class="chip ok">{m.search.preferred}</span>{/if}
							{#if r.generated}<span class="chip">{m.search.generatedResult}</span>{/if}
							{#if r.stale}
								<span class="chip warn" title={m.search.staleHint}>{m.search.stale}</span>
							{/if}
						</p>
						<ol class="hits">
							{#each r.hits as h (`${h.index}/${h.term}/${h.nth}`)}
								{@const place = hitPlace(h, m)}
								{@const origin = hitOrigin(h, m)}
								<li>
									<button class="hit" aria-label={m.search.open(place)} onclick={() => open(r, h)}>
										<span class="place num">{place}</span>
										<span class="snippet">
											{#if origin}<span class="origin">{origin}</span>{/if}
											{h.before}{#each hitWords(h) as w, i (i)}{#if w.found}<mark>{w.text}</mark
													>{:else}{w.text}{/if}{/each}{h.after}
										</span>
										<span class="state">
											{#if h.corrected}<span class="chip hand">{m.review.corrected}</span>{/if}
											{#if h.reviewed}<span class="chip ok">{m.review.reviewed}</span>{/if}
										</span>
									</button>
								</li>
							{/each}
						</ol>
					</div>
				{/each}
			</section>
		{/each}
	{/if}

	<p class="rebuild muted">
		{m.search.rebuildHint}
		<button class="link" onclick={rebuild}>{m.search.rebuild}</button>
	</p>
</div>

<style>
	.search-view {
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
	.distance {
		width: 4.5rem;
	}
	.hint {
		max-width: 44rem;
		font-size: var(--type-xs);
	}
	.preparing {
		display: grid;
		gap: var(--space-1);
		max-width: 32rem;
	}
	.check {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
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
	.document {
		display: grid;
		gap: var(--space-2);
		padding-top: var(--space-3);
		border-top: 1px solid var(--line);
	}
	.document h3 {
		font-size: var(--type-lg);
	}
	.result {
		display: grid;
		gap: var(--space-1);
	}
	.result-name {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.hits {
		display: grid;
		gap: 1px;
		margin: 0;
		padding: 0;
		list-style: none;
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
		unicode-bidi: plaintext;
	}
	.origin {
		margin-right: var(--space-2);
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	/* Found words in ink on grey, as the review's search marks them. */
	mark {
		padding: 0 0.1em;
		border-radius: 2px;
		background: var(--found);
		color: var(--text-bright);
		font-weight: 600;
	}
	.state {
		display: flex;
		gap: var(--space-1);
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
	.rebuild {
		font-size: var(--type-xs);
		max-width: 44rem;
	}
	@media (max-width: 640px) {
		.hit {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
