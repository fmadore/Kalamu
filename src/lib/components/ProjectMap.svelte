<script lang="ts">
	// The map of a project's documents, beside its search: one point per document from the text
	// prepared for the search by meaning, grouped by collection or tag, and always beside it a
	// list reached by keyboard: every document placed, the one selected and the documents nearest
	// to it with their similarity, so that all the map says can be read without seeing it. The
	// screen says which model drew it and when, what was placed and what was not and why, and
	// that positions are approximate. The projection and the neighbours are computed in Rust.
	import { onMount } from 'svelte';
	import MapCanvas from './MapCanvas.svelte';
	import MapShape from './MapShape.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { MapView } from '#lib/bindings/MapView.js';
	import type { MeaningStatus } from '#lib/bindings/MeaningStatus.js';
	import type { Neighbour } from '#lib/bindings/Neighbour.js';
	import { describeError } from '#lib/errors.js';
	import { formatDateTime, formatPercent, messages } from '#lib/i18n/index.js';
	import {
		isOther,
		mapGroups,
		similarityText,
		type MapGroup,
		type MapGrouping
	} from '#lib/meaning.js';
	import { folded } from '#lib/search.js';
	import { go } from '#lib/state.svelte.js';
	import { keepFor } from '#lib/stored.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		collections,
		onsearch
	}: {
		projectId: string;
		collections: CollectionView[];
		/** Opens the project's search. */
		onsearch: () => void;
	} = $props();
	const m = messages();
	const t = $derived(m.meaning.map);

	let status = $state<MeaningStatus | null>(null);
	let map = $state<MapView | null>(null);
	let error = $state<string | null>(null);
	let noModel = $state(false);
	let grouping = $state<MapGrouping>('collection');
	let selected = $state<string | null>(null);
	let near = $state<Neighbour[] | null>(null);

	const groups = $derived(map ? mapGroups(map.points, grouping, collections) : null);
	const byTitle = $derived(
		map ? [...map.points].sort((a, b) => folded(a.title).localeCompare(folded(b.title))) : []
	);
	const chosen = $derived(map?.points.find((p) => p.document_id === selected) ?? null);
	const sim = (v: number) => similarityText(v, m.locale.tag);

	function groupName(group: MapGroup | undefined): string {
		if (!group) return '';
		if (group.key === '') return t.inNone;
		if (isOther(group)) return t.others;
		return group.label;
	}

	async function load() {
		error = null;
		try {
			status = await api.meaningStatus(projectId);
			if (!status.choice) {
				noModel = true;
				return;
			}
			map = await api.meaningMap(projectId);
		} catch (e) {
			if ((e as { id?: string })?.id === 'meaning.no_model') noModel = true;
			else error = describeError(e, m);
		}
	}

	// The documents nearest to the one selected, asked of Rust each time it changes.
	let ticket = 0;
	$effect(() => {
		const id = selected;
		near = null;
		if (!id) return;
		const mine = ++ticket;
		api
			.meaningNeighbours(projectId, id)
			.then((found) => {
				if (mine === ticket) near = found;
			})
			.catch((e) => {
				if (mine === ticket) error = describeError(e, m);
			});
	});

	function open(id: string) {
		go({ name: 'projects', projectId, documentId: id });
	}

	function toSearch() {
		keepFor('kalamu.search.kind', projectId, 'meaning', 100);
		onsearch();
	}

	onMount(() => {
		void load();
	});
</script>

<div class="map-view">
	<p class="lede muted">{t.lede}</p>
	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}

	{#if noModel}
		<p class="muted">{t.noModel}</p>
		<button class="btn sm" onclick={toSearch}>{t.openSetup}</button>
	{:else if map && status}
		<ul class="facts muted num">
			<li>
				{status.prepared_at
					? t.made(status.model?.name ?? map.space.model, formatDateTime(status.prepared_at))
					: t.madeNever(status.model?.name ?? map.space.model)}
			</li>
			<li>{t.placed(map.placed, map.documents)}</li>
			{#if map.unread > 0}<li>{t.unread(map.unread)}</li>{/if}
			{#if map.not_prepared > 0}<li>{t.notPrepared(map.not_prepared)}</li>{/if}
			{#if map.changed > 0}<li>{t.changed(map.changed)}</li>{/if}
			{#if map.without_preference > 0}<li>{t.withoutPreference(map.without_preference)}</li>{/if}
			{#if map.points.length > 0}<li>{t.shown(formatPercent(map.shown))}</li>{/if}
		</ul>

		{#if map.points.length === 0}
			<p class="too-few">{t.tooFew(map.placed)}</p>
			<button class="btn sm" onclick={toSearch}>{t.openSetup}</button>
		{:else if groups}
			<div class="layout">
				<div class="drawing">
					<div class="above">
						<label class="pick">
							<span>{t.groupBy}</span>
							<select bind:value={grouping}>
								<option value="collection">{t.groups.collection}</option>
								<option value="tag">{t.groups.tag}</option>
							</select>
						</label>
						<ul class="legend" aria-label={t.legend}>
							{#each groups.legend as g (g.key)}
								<li>
									<svg viewBox="-1.4 -1.4 2.8 2.8" aria-hidden="true" class="swatch">
										<MapShape shape={g.shape} tone={g.tone} x={0} y={0} r={1} />
									</svg>
									<span>{groupName(g)}</span>
									<span class="muted num">{g.count}</span>
								</li>
							{/each}
						</ul>
					</div>
					<MapCanvas
						points={map.points}
						groupOf={groups.groupOf}
						{groupName}
						bind:selected
						onopen={open}
					/>
				</div>

				<aside class="side">
					<section aria-labelledby="map-selected">
						<h3 id="map-selected">{t.selected}</h3>
						{#if chosen}
							<p class="chosen">
								<span class="title">{chosen.title}</span>
								<span class="muted num">{t.chunks(chosen.chunks)}</span>
							</p>
							<button class="btn sm" onclick={() => open(chosen.document_id)}>
								{t.openDocument}
							</button>
							<h4>{t.nearest}</h4>
							{#if near && near.length === 0}
								<p class="muted">{t.noNeighbours}</p>
							{:else if near}
								<ol class="near" aria-label={t.nearest}>
									{#each near as n (n.document_id)}
										<li>
											<button class="link" onclick={() => (selected = n.document_id)}>
												{n.title}
											</button>
											<span class="muted num" title={m.meaning.similarityHint}>
												{sim(n.similarity)}
											</span>
										</li>
									{/each}
								</ol>
							{/if}
						{:else}
							<p class="muted">{t.noneSelected}</p>
						{/if}
					</section>
					<section aria-labelledby="map-documents">
						<h3 id="map-documents">{t.documents}</h3>
						<ul class="all">
							{#each byTitle as p (p.document_id)}
								<li>
									<button
										class="row"
										aria-pressed={p.document_id === selected}
										onclick={() => (selected = p.document_id)}
									>
										<span>{p.title}</span>
										<span class="muted small">{groupName(groups.groupOf.get(p.document_id))}</span>
									</button>
								</li>
							{/each}
						</ul>
					</section>
				</aside>
			</div>
		{/if}
	{/if}
</div>

<style>
	.map-view {
		display: grid;
		gap: var(--space-4);
		align-content: start;
		min-width: 0;
	}
	.lede {
		max-width: 44rem;
		font-size: var(--type-sm);
	}
	.facts {
		margin: 0;
		padding-left: 1.1rem;
		font-size: var(--type-sm);
		max-width: 44rem;
	}
	.too-few {
		max-width: 44rem;
	}
	.layout {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(14rem, 20rem);
		gap: var(--space-4);
		align-items: start;
	}
	.drawing {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
	}
	.above {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		align-items: center;
	}
	.pick {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: var(--type-xs);
	}
	.legend li {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
	}
	.swatch {
		width: 0.85rem;
		height: 0.85rem;
	}
	.side {
		display: grid;
		gap: var(--space-4);
		min-width: 0;
	}
	.side section {
		display: grid;
		gap: var(--space-2);
	}
	.side h3 {
		font-size: var(--type-sm);
		font-weight: 600;
		color: var(--text-bright);
	}
	.side h4 {
		font-size: var(--type-xs);
		font-weight: 600;
		color: var(--text-secondary);
	}
	.side .btn {
		justify-self: start;
	}
	.chosen {
		display: grid;
		gap: 2px;
	}
	.chosen .title {
		color: var(--text-bright);
		font-weight: 500;
	}
	.near,
	.all {
		display: grid;
		gap: 1px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: var(--type-sm);
	}
	.near li {
		display: flex;
		justify-content: space-between;
		gap: var(--space-2);
		align-items: baseline;
	}
	.all {
		max-height: 24rem;
		overflow-y: auto;
	}
	.row {
		display: flex;
		justify-content: space-between;
		gap: var(--space-2);
		width: 100%;
		padding: var(--space-1) var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: none;
		color: var(--text-secondary);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.row:hover,
	.row:focus-visible,
	.row[aria-pressed='true'] {
		background: var(--wash);
		color: var(--text-bright);
	}
	.small {
		font-size: var(--type-xs);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	@media (max-width: 900px) {
		.layout {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
