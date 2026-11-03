<script lang="ts">
	// What to check across a project, a collection or a selection: the pages and answers that need
	// the researcher, each with its reason, failures kept apart from answers. The engine gathers
	// them from what is stored, so opening, filtering and stepping through call no model. The
	// coverage above the list counts every result read, whatever the filters show. Opening an item
	// opens its review at its page, with the next and previous items a click away.
	import Icon from './Icon.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { QueueItem } from '#lib/bindings/QueueItem.js';
	import type { QueueSelection } from '#lib/bindings/QueueSelection.js';
	import type { QueueView } from '#lib/bindings/QueueView.js';
	import type { ResultKey } from '#lib/bindings/ResultKey.js';
	import type { Signal } from '#lib/bindings/Signal.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import {
		DEFAULT_FILTER,
		REASONS,
		grouped,
		itemKey,
		placeOf,
		reasonCounts,
		reasonText,
		shownItems,
		type QueueFilter
	} from '#lib/queue.js';
	import { app, go } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		collections,
		selection = []
	}: { projectId: string; collections: CollectionView[]; selection?: string[] } = $props();
	const m = messages();
	const q = m.queue;
	/** Rows drawn at first in each group; the rest on request, so a large collection stays quick. */
	const PAGE = 200;

	// Coming back from a review keeps the list as it was set.
	// svelte-ignore state_referenced_locally
	const kept = app.queue?.projectId === projectId ? app.queue : null;
	// A selection from the table opens the list on it.
	// svelte-ignore state_referenced_locally
	const fromTable = selection.length > 0;
	// svelte-ignore state_referenced_locally
	const chosen: string[] = fromTable ? selection : (kept?.selection ?? []);
	let scope = $state(fromTable ? 'selection' : (kept?.scope ?? ''));
	let excluded = $state<ResultKey[]>(kept?.excluded ?? []);
	let filter = $state<QueueFilter>(
		kept ? { ...kept.filter, reasons: [...kept.filter.reasons] } : { ...DEFAULT_FILTER }
	);
	let view = $state<QueueView | null>(null);
	let error = $state<string | null>(null);
	let loading = $state(false);
	let limit = $state({ processing: PAGE, answers: PAGE });

	const items = $derived(view?.queue.items ?? []);
	const counts = $derived(view?.queue.counts);
	const tally = $derived(reasonCounts(items));
	const shown = $derived(shownItems(items, filter));
	const groups = $derived(grouped(shown));
	/** The order the review steps through: what is not read first, as listed. */
	const trail = $derived([...groups.processing, ...groups.answers]);

	function selectionOf(s: string): QueueSelection {
		if (s === 'selection') return { kind: 'documents', ids: chosen };
		if (s) return { kind: 'collection', id: s };
		return { kind: 'project' };
	}

	async function load() {
		loading = true;
		error = null;
		try {
			view = await api.reviewQueue(projectId, selectionOf(scope), excluded);
		} catch (e) {
			error = describeError(e, m);
		} finally {
			loading = false;
		}
	}

	function setAside(key: ResultKey, included: boolean) {
		const others = excluded.filter((k) => k.job_id !== key.job_id || k.setup_id !== key.setup_id);
		excluded = included ? others : [...others, key];
	}

	function toggleReason(reason: Signal['kind']) {
		filter.reasons = filter.reasons.includes(reason)
			? filter.reasons.filter((r) => r !== reason)
			: [...filter.reasons, reason];
	}

	function open(item: QueueItem) {
		app.queue = {
			projectId,
			scope,
			selection: chosen,
			excluded,
			filter: { ...filter, reasons: [...filter.reasons] },
			items: trail,
			at: trail.findIndex((i) => itemKey(i) === itemKey(item))
		};
		go({
			name: 'review',
			jobId: item.job_id,
			sourceId: item.source_id,
			index: item.index ?? undefined
		});
	}

	const tone = (s: Signal) =>
		s.kind === 'failed'
			? 'danger'
			: s.kind === 'not_reviewed' || s.kind === 'not_read'
				? ''
				: s.kind === 'disagrees'
					? 'hand'
					: 'warn';

	$effect(() => {
		void scope;
		void excluded;
		void load();
	});
</script>

{#snippet row(item: QueueItem)}
	<li>
		<button
			class="item"
			class:resolved={!item.open}
			onclick={() => open(item)}
			aria-label={q.open(item.document, placeOf(item, m))}
		>
			<span class="body">
				<span class="line">
					<strong>{item.document}</strong>
					<span>{placeOf(item, m)}</span>
					<span class="muted">{item.setup_label} · {item.job_name}</span>
					{#if item.preferred}<span class="chip ok">{q.preferredBadge}</span>{/if}
				</span>
				<span class="line">
					{#each item.signals as s, i (i)}
						<span class="chip {tone(s)}">{reasonText(s, m, item.piece === null)}</span>
					{/each}
					{#if item.reviewed}<span class="chip ok">{q.reviewedBadge}</span>{/if}
					{#if item.revision}
						<span class="revision mono muted" title={q.revisionHint}
							>{q.revision(item.revision)}{#if item.answered_at}, {formatDateTime(
									item.answered_at
								)}{/if}</span
						>
					{/if}
				</span>
			</span>
			<Icon name="chevron" size={14} />
		</button>
	</li>
{/snippet}

<div class="queue">
	<div class="head">
		<h2>{q.title}</h2>
	</div>
	<p class="muted lede">{q.lede}</p>
	{#if error}<p class="chip danger banner" role="alert">{error}</p>{/if}

	<div class="table-filters">
		<label class="pick">
			<span>{q.scope}</span>
			<select bind:value={scope}>
				<option value="">{q.wholeProject}</option>
				{#each collections as c (c.collection.id)}
					<option value={c.collection.id}>{c.collection.name}</option>
				{/each}
				{#if chosen.length > 0}
					<option value="selection">{q.selection(chosen.length)}</option>
				{/if}
			</select>
		</label>
	</div>

	{#if counts}
		<p class="coverage num" title={q.coverageHint}>
			<span>{q.coverage.documents(counts.documents_read, counts.documents)}</span>
			<span>{q.coverage.pieces(counts.processed, counts.pieces)}</span>
			{#if counts.failed + counts.not_read > 0}
				<span>{q.coverage.unprocessed(counts.failed + counts.not_read)}</span>
			{/if}
			<span>{q.coverage.reviewed(counts.reviewed, counts.processed)}</span>
			<span>{q.coverage.flagged(counts.flagged, counts.unresolved)}</span>
		</p>
	{/if}

	{#if view && (view.choices.length > 0 || view.preferred > 0)}
		<details class="results">
			<summary>{q.results}</summary>
			<p class="hint">{q.resultsHint}</p>
			{#if view.preferred > 0}<p class="hint num">{q.preferred(view.preferred)}</p>{/if}
			<ul>
				{#each view.choices as c (`${c.key.job_id}/${c.key.setup_id}`)}
					<li>
						<label>
							<input
								type="checkbox"
								checked={c.included}
								onchange={(e) => setAside(c.key, e.currentTarget.checked)}
							/>
							<span>{q.resultChoice(c.setup_label, c.job_name, c.documents)}</span>
							<span class="muted">{m.task[c.task]}</span>
						</label>
					</li>
				{/each}
			</ul>
		</details>
	{/if}

	{#if view}
		<fieldset class="reasons-filter">
			<legend>{q.show}</legend>
			{#each REASONS as r (r)}
				<label class="toggle">
					<input
						type="checkbox"
						checked={filter.reasons.includes(r)}
						onchange={() => toggleReason(r)}
					/>
					<span>{q.reason[r]}</span>
					<span class="num muted">{tally[r]}</span>
				</label>
			{/each}
			<label class="toggle">
				<input type="checkbox" bind:checked={filter.resolved} />
				<span>{q.showResolved}</span>
			</label>
		</fieldset>

		{#if counts && counts.results === 0}
			<p class="muted">{q.empty}</p>
		{:else if shown.length === 0}
			<p class="muted">{q.none}</p>
		{/if}

		{#if groups.processing.length > 0}
			<section>
				<h3>{q.processing} <span class="num muted">{groups.processing.length}</span></h3>
				<p class="hint">{q.processingHint}</p>
				<ul class="items">
					{#each groups.processing.slice(0, limit.processing) as item (itemKey(item))}
						{@render row(item)}
					{/each}
				</ul>
				{#if groups.processing.length > limit.processing}
					<button class="btn ghost sm" onclick={() => (limit.processing += PAGE)}
						>{q.more(Math.min(PAGE, groups.processing.length - limit.processing))}</button
					>
				{/if}
			</section>
		{/if}

		{#if groups.answers.length > 0}
			<section>
				<h3>{q.answers} <span class="num muted">{groups.answers.length}</span></h3>
				<ul class="items">
					{#each groups.answers.slice(0, limit.answers) as item (itemKey(item))}
						{@render row(item)}
					{/each}
				</ul>
				{#if groups.answers.length > limit.answers}
					<button class="btn ghost sm" onclick={() => (limit.answers += PAGE)}
						>{q.more(Math.min(PAGE, groups.answers.length - limit.answers))}</button
					>
				{/if}
			</section>
		{/if}
	{:else if loading}
		<p class="muted">{q.loading}</p>
	{/if}
</div>

<style>
	.queue {
		display: grid;
		gap: var(--space-3);
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	h2 {
		margin: 0;
	}
	h3 {
		margin: var(--space-3) 0 var(--space-1);
		font-size: var(--type-sm);
		font-weight: 600;
	}
	.lede {
		margin: 0;
		max-width: 48rem;
	}
	.coverage {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-4);
		margin: 0;
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	.pick {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.hint {
		margin: 0 0 var(--space-2);
		max-width: 48rem;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.results {
		font-size: var(--type-sm);
	}
	.results summary {
		cursor: pointer;
		color: var(--text-secondary);
	}
	.results ul {
		display: grid;
		gap: var(--space-1);
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.results label,
	.toggle {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.reasons-filter {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-4);
		margin: 0;
		padding: 0;
		border: 0;
		font-size: var(--type-sm);
	}
	.reasons-filter legend {
		float: left;
		margin-right: var(--space-2);
		color: var(--text-secondary);
	}
	.items {
		display: grid;
		margin: 0;
		padding: 0;
		list-style: none;
		border-top: 1px solid var(--line);
	}
	.item {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		width: 100%;
		min-height: var(--control-md);
		padding: var(--space-2);
		border: 0;
		border-bottom: 1px solid var(--line);
		background: none;
		color: var(--text-secondary);
		font: inherit;
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.item.resolved {
		opacity: 0.75;
	}
	.body {
		display: grid;
		gap: var(--space-1);
		min-width: 0;
	}
	.line {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1) var(--space-2);
		min-width: 0;
	}
	.line strong {
		color: var(--text-bright);
		font-weight: 500;
		overflow-wrap: anywhere;
	}
	.line .chip {
		white-space: normal;
	}
	.revision {
		font-size: var(--type-xs);
	}
</style>
