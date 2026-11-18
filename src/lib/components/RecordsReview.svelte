<script lang="ts">
	// The review of records (#171): the page on its lightbox beside the rows read on it, as a
	// table, with the strip of pages below; turning the page brings its rows. Every cell is a field
	// of its own: a change is saved when it is left (or on Enter), a row is added after another or
	// at the end and removed by its own button, each change made by the engine in the page's
	// corrected answers, the model's rows kept apart. A page not read says so, and holds no rows to
	// take for complete. Ctrl Enter marks the page reviewed, as in every review.
	import { tick } from 'svelte';
	import Icon from './Icon.svelte';
	import PageStrip from './PageStrip.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import type { Field } from '#lib/bindings/Field.js';
	import type { RowEdit } from '#lib/bindings/RowEdit.js';
	import { messages } from '#lib/i18n/index.js';
	import { pieceCost, pieceLabel, pieceNumber } from '#lib/pieces.js';
	import { cellText, columnsOf, rowsOf } from '#lib/records.js';
	import { ownRegions, placeOf } from '#lib/regions.svelte.js';
	import type { Review } from '#lib/review.svelte.js';

	let {
		review,
		start
	}: {
		/** The analysis review's job and pieces, loaded once. */
		review: Review;
		/** The piece to open on, from 0: a page chosen in the list of what to check. */
		start?: number;
	} = $props();
	const m = messages();

	/** The set-up whose rows are shown, in a comparison; the first otherwise. */
	let chosen = $state<string | null>(null);
	const setups = $derived(review.setups);
	const setup = $derived(setups.find((s) => s.id === chosen) ?? setups[0]);
	const schema = $derived(setup?.prompt.schema);
	const columns = $derived(columnsOf(schema));
	const pageViews = $derived(review.views.filter((v) => v.setup_id === setup?.id));

	/** The page shown, as an index among the source's pages. */
	let page = $state(0);
	let started: number | undefined;
	$effect(() => {
		const at = pageViews.findIndex((v) => v.index === start);
		if (start !== undefined && at >= 0 && started !== start) {
			started = start;
			page = at;
		}
	});
	const shown = $derived(pageViews[page]);
	const done = $derived(shown?.status === 'done' || shown?.status === 'cached');
	const rows = $derived(rowsOf(schema, shown?.corrected_fields ?? shown?.ai_fields ?? []));
	const isText = $derived(review.source?.kind === 'text');

	$effect(() => review.showPage(page));
	/** The researcher's regions on the scan page shown (#84). */
	// svelte-ignore state_referenced_locally
	const regions = ownRegions(review.jobId, review.sourceId, () =>
		review.paged ? placeOf(shown?.piece) : null
	);

	function step(delta: number) {
		page = Math.min(Math.max(page + delta, 0), Math.max(pageViews.length - 1, 0));
	}

	let table = $state<HTMLElement | null>(null);

	/** The changes sent, one after another: each is made on the rows the one before left. */
	let sending: Promise<boolean> = Promise.resolve(true);

	/** Sends one change of the page's rows to the engine; the pieces are read again. */
	function edit(change: RowEdit): Promise<boolean> {
		const [target, view] = [setup, shown];
		if (!target || !view) return Promise.resolve(false);
		sending = sending.then(() => review.editRows(target, view.index, change));
		return sending;
	}

	/** A cell left changed: saved, unless it reads as it did. */
	function commit(row: number, column: Field, value: string) {
		if (value === cellText(rows[row] ?? [], column.key)) return;
		void edit({ kind: 'cell', row, key: column.key, value });
	}

	/** Adds an empty row at `at`, its first cell then given the focus. */
	async function addRow(at: number) {
		if (!(await edit({ kind: 'add', at }))) return;
		await tick();
		table?.querySelector<HTMLElement>(`[data-row="${at}"] :is(input, select)`)?.focus();
	}

	/** Removes a row, the focus going to the row that takes its place, or the last. */
	async function removeRow(row: number) {
		if (!(await edit({ kind: 'remove', row }))) return;
		await tick();
		const next = Math.min(row, rows.length - 1);
		const target =
			next >= 0
				? table?.querySelector<HTMLElement>(`[data-row="${next}"] :is(input, select)`)
				: table?.querySelector<HTMLElement>('.add');
		target?.focus();
	}

	function toggleReviewed() {
		if (setup && shown && done) {
			void review.save({
				setup,
				index: shown.index,
				fields: shown.corrected_fields,
				reviewed: !shown.reviewed
			});
		}
	}
</script>

{#snippet actions()}
	{#if setups.length > 1}
		<label class="by">
			<span>{m.review.rowsBy}</span>
			<select value={setup?.id ?? ''} onchange={(e) => (chosen = e.currentTarget.value)}>
				{#each setups as s (s.id)}
					<option value={s.id}>{s.label}</option>
				{/each}
			</select>
		</label>
	{/if}
	{#if setup && shown?.corrected_fields}
		{@const target = setup}
		{@const view = shown}
		<button
			class="btn"
			onclick={() =>
				review.save({ setup: target, index: view.index, fields: null, reviewed: view.reviewed })}
			>{m.review.revert}</button
		>
	{/if}
{/snippet}

{#snippet tools()}
	{done ? m.review.rows(rows.length) : ''}
{/snippet}

{#snippet foot()}
	<span class="mono">{shown?.model_version ?? setup?.model ?? ''}{pieceCost(shown, m)}</span>
{/snippet}

{#snippet strip()}
	{#if pageViews.length > 1}
		<PageStrip
			jobId={review.jobId}
			sourceId={review.sourceId}
			paged={review.paged}
			tally
			current={shown?.index ?? -1}
			items={pageViews.map((v, i) => ({
				index: v.index,
				label: pieceLabel(v.piece, m),
				number: pieceNumber(v.piece, i, m),
				done: v.reviewed
			}))}
			onselect={(index) => {
				const i = pageViews.findIndex((v) => v.index === index);
				if (i >= 0) page = i;
			}}
		/>
	{/if}
{/snippet}

<ReviewFrame
	jobId={review.jobId}
	sourceId={review.sourceId}
	jobName={review.job?.name ?? null}
	fileName={review.source?.file_name ?? ''}
	error={review.error}
	{actions}
	reviewed={shown && done ? shown.reviewed : null}
	onreviewed={toggleReviewed}
	onstep={pageViews.length > 1 ? step : undefined}
	source={isText ? 'text' : 'scan'}
	piece={shown && !isText ? pieceLabel(shown.piece, m) : null}
	original={shown?.original_file ?? null}
	paging={{ at: page, count: pageViews.length }}
	preview={review.preview}
	regions={regions.place ? regions : undefined}
	text={review.text}
	witness={{ label: setup?.label ?? '', corrected: shown?.corrected_fields != null }}
	label={m.review.rowsTitle}
	{tools}
	{foot}
	{strip}
>
	{#if !shown || shown.status === 'pending' || shown.status === 'running' || shown.status === 'submitted'}
		<p class="muted">{m.review.notDone}</p>
	{:else if shown.status === 'failed'}
		<p class="error">{shown.error}</p>
		<p class="muted">{m.review.rowsNotRead}</p>
	{:else}
		{#if shown.truncated}<p class="notice">{m.review.truncated}</p>{/if}
		<div class="rows" bind:this={table}>
			{#if rows.length === 0}
				<p class="muted">{m.review.noRows}</p>
			{:else}
				<table class="data-table">
					<thead>
						<tr>
							<th scope="col" class="n">{m.review.rowNumber}</th>
							{#each columns as column (column.key)}
								<th scope="col">{column.label}</th>
							{/each}
							<th scope="col" class="act"><span class="sr-only">{m.review.rowActions}</span></th>
						</tr>
					</thead>
					<tbody>
						{#each rows as row, r (r)}
							<tr data-row={r}>
								<td class="n mono">{r + 1}</td>
								{#each columns as column (column.key)}
									{@const value = cellText(row, column.key)}
									<td>
										{#if column.kind.type === 'choice'}
											<select
												class="cell"
												aria-label={m.review.cell(column.label, r + 1)}
												{value}
												onchange={(e) => commit(r, column, e.currentTarget.value)}
											>
												<option value="">{m.review.noChoice}</option>
												{#each column.kind.values as choice (choice)}
													<option value={choice}>{choice}</option>
												{/each}
											</select>
										{:else}
											<input
												class="cell"
												type="text"
												dir="auto"
												aria-label={m.review.cell(column.label, r + 1)}
												{value}
												onchange={(e) => commit(r, column, e.currentTarget.value)}
											/>
										{/if}
									</td>
								{/each}
								<td class="act">
									<button
										class="btn sm icon ghost"
										aria-label={m.review.insertRow(r + 1)}
										title={m.review.insertRow(r + 1)}
										onclick={() => addRow(r + 1)}><Icon name="plus" size={14} /></button
									>
									<button
										class="btn sm icon ghost"
										aria-label={m.review.removeRow(r + 1)}
										title={m.review.removeRow(r + 1)}
										onclick={() => removeRow(r)}><Icon name="close" size={14} /></button
									>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
			<button class="btn add" onclick={() => addRow(rows.length)}
				><Icon name="plus" size={14} />{m.review.addRow}</button
			>
			<p class="muted small">{m.review.rowsKept}</p>
		</div>
	{/if}
</ReviewFrame>

<style>
	.error {
		color: var(--danger-text);
	}
	.notice {
		margin-bottom: var(--space-3);
	}
	.by {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		font-size: var(--type-sm);
	}
	.rows {
		display: grid;
		gap: var(--space-3);
		justify-items: start;
	}
	.rows table {
		font-family: var(--font-text);
	}
	.n {
		width: 2.5rem;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.act {
		width: calc(var(--control-sm) * 2 + var(--space-1));
		white-space: nowrap;
	}
	/* A cell reads as the text it holds until it is pointed at or edited. */
	.cell {
		width: 100%;
		min-width: 4rem;
		border-color: transparent;
		background: transparent;
		font: inherit;
		color: var(--text-body);
	}
	.cell:hover {
		border-color: var(--control-line);
	}
	.cell:focus-visible {
		border-color: var(--text-bright);
	}
	.btn.icon {
		width: var(--control-sm);
		padding: 0;
	}
	.add {
		border-style: dashed;
	}
	.small {
		font-size: var(--type-xs);
	}
</style>
