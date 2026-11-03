<script lang="ts">
	// A table of contents' items (#50): each article, letter or item as pages of the document,
	// made once from the corrected entries and then the researcher's to correct, join and split.
	// The selected items' pages go to a new job, each item a PDF of its own, or are exported to
	// a folder with a list of them. The engine parses the pages and keeps the items; this only
	// shows them and passes the researcher's changes on.
	import Icon from './Icon.svelte';
	import type { DocumentUnit } from '#lib/bindings/DocumentUnit.js';
	import type { PieceRef } from '#lib/bindings/PieceRef.js';
	import type { UnitsView } from '#lib/bindings/UnitsView.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { enginePages, pieceLabel, typedPages } from '#lib/pieces.js';
	import { go } from '#lib/state.svelte.js';
	import { api, pickFolder } from '#lib/tauri.js';

	let {
		jobId,
		setupId,
		sourceId,
		shown,
		onshow
	}: {
		jobId: string;
		setupId: string;
		sourceId: string;
		/** The page shown beside the list, whose items are marked. */
		shown: PieceRef | null;
		/** Shows a page of the document. */
		onshow: (page: PieceRef) => void;
	} = $props();

	const m = messages();
	const u = m.units;

	let view = $state<UnitsView | null>(null);
	let selected = $state<string[]>([]);
	let editing = $state<{ id: string | null; title: string; pages: string } | null>(null);
	let notes = $state<string[]>([]);
	let error = $state<string | null>(null);
	let busy = $state(false);

	const units = $derived(view?.units ?? []);
	const chosen = $derived(units.filter((v) => selected.includes(v.unit.id)).map((v) => v.unit.id));
	const titleOf = (unit: DocumentUnit) => unit.title || u.untitled;
	/** A page as typed in a list of pages: "3", "8 (left)", in the interface language. */
	const typedPage = (page: PieceRef) =>
		page.kind !== 'page'
			? ''
			: page.half
				? `${page.number} (${m.units.halves[page.half]})`
				: String(page.number);

	async function load() {
		try {
			view = await api.unitsView(jobId, setupId, sourceId);
			const ids = new Set(view.units.map((v) => v.unit.id));
			selected = selected.filter((id) => ids.has(id));
		} catch (e) {
			error = describeError(e, m);
		}
	}

	/** Runs a change, then shows the items as they now are. */
	async function act(change: () => Promise<unknown>): Promise<boolean> {
		busy = true;
		error = null;
		notes = [];
		try {
			await change();
			await load();
			return true;
		} catch (e) {
			error = describeError(e, m);
			return false;
		} finally {
			busy = false;
		}
	}

	async function make() {
		await act(async () => {
			const report = await api.seedUnits(jobId, setupId, sourceId);
			notes = [
				u.made(report.made),
				...report.problems.map((p) =>
					p.unknown.length > 0
						? u.unknownPages(p.entry, typedPages(p.unknown.join(', '), m))
						: u.unreadPages(p.entry, p.pages)
				)
			];
		});
	}

	async function save() {
		const draft = editing;
		if (!draft) return;
		const saved = await act(() =>
			api.saveUnit({
				id: draft.id ?? undefined,
				job_id: jobId,
				setup_id: setupId,
				source_id: sourceId,
				title: draft.title,
				pages: enginePages(draft.pages, m)
			})
		);
		if (saved) editing = null;
	}

	function isHere(unit: DocumentUnit): boolean {
		const page = shown;
		if (page?.kind !== 'page') return false;
		return unit.pages.some(
			(p) =>
				p.kind === 'page' &&
				p.number === page.number &&
				(!p.half || !page.half || p.half === page.half)
		);
	}

	function toggle(id: string) {
		selected = selected.includes(id) ? selected.filter((s) => s !== id) : [...selected, id];
	}

	/** The selected items' pages, frozen, in a new job of the task chosen. */
	async function send(task: 'text_extraction' | 'analysis') {
		busy = true;
		error = null;
		try {
			const sources = await api.unitSources(chosen);
			go({ name: 'new', task, sources });
		} catch (e) {
			error = describeError(e, m);
		} finally {
			busy = false;
		}
	}

	async function exportChosen() {
		const folder = await pickFolder();
		if (!folder) return;
		await act(async () => {
			const written = await api.exportUnits(chosen, folder);
			notes = [u.exported(written.length)];
		});
	}

	/** The other items a shared page holds, by title. */
	function sharedWith(ids: string[]): string {
		return ids
			.map((id) => units.find((v) => v.unit.id === id)?.unit)
			.filter((unit) => unit !== undefined)
			.map((unit) => m.common.quoted(titleOf(unit)))
			.join(', ');
	}

	$effect(() => {
		void [jobId, setupId, sourceId];
		void load();
	});
</script>

<section class="units" aria-busy={busy}>
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#each notes as note (note)}<p class="note">{note}</p>{/each}

	{#if view && units.length === 0}
		<div class="empty">
			<p>{u.lede}</p>
			{#if view.entries > 0}
				<button class="btn primary" onclick={make} disabled={busy}
					><Icon name="plus" size={14} />{u.make}</button
				>
				<p class="muted small">{u.makeHint}</p>
			{:else}
				<p class="muted">{u.notCombined}</p>
			{/if}
		</div>
	{:else if view}
		<div class="bar">
			<label class="all">
				<input
					type="checkbox"
					aria-label={u.selectAll}
					checked={chosen.length === units.length}
					indeterminate={chosen.length > 0 && chosen.length < units.length}
					onchange={() =>
						(selected = chosen.length === units.length ? [] : units.map((v) => v.unit.id))}
				/>
				<span class="muted small"
					>{chosen.length > 0
						? u.selected(chosen.length, units.length)
						: u.count(units.length)}</span
				>
			</label>
			<div class="send">
				<button
					class="btn sm"
					title={u.extractHint}
					disabled={busy || chosen.length === 0 || !view.readable}
					onclick={() => send('text_extraction')}
					><Icon name="text_extraction" size={14} />{u.extract}</button
				>
				<button
					class="btn sm"
					title={u.analyseHint}
					disabled={busy || chosen.length === 0 || !view.readable}
					onclick={() => send('analysis')}><Icon name="analysis" size={14} />{u.analyse}</button
				>
				<button
					class="btn sm"
					title={u.exportHint}
					disabled={busy || chosen.length === 0 || !view.readable}
					onclick={exportChosen}>{u.export}</button
				>
			</div>
		</div>
		{#if !view.readable}<p class="muted small">{u.pdfOnly}</p>{/if}

		<ol class="list">
			{#each units as { unit, pages, shared }, i (unit.id)}
				{@const next = units[i + 1]?.unit}
				<li class:here={isHere(unit)}>
					{#if editing?.id === unit.id}
						{@render editor()}
					{:else}
						<div class="head">
							<input
								type="checkbox"
								aria-label={u.select(titleOf(unit))}
								checked={selected.includes(unit.id)}
								onchange={() => toggle(unit.id)}
							/>
							<button
								class="item"
								disabled={unit.pages.length === 0}
								onclick={() => unit.pages[0] && onshow(unit.pages[0])}
							>
								<span class="item-title">{titleOf(unit)}</span>
								<span class="item-pages mono">{typedPages(pages, m) || u.noPages}</span>
							</button>
						</div>
						{#each shared as s (pieceLabel(s.page, m))}
							<p class="shared small">{u.shares(pieceLabel(s.page, m), sharedWith(s.with))}</p>
						{/each}
						<div class="actions">
							<button
								class="btn ghost sm"
								aria-label={u.editItem(titleOf(unit))}
								onclick={() =>
									(editing = { id: unit.id, title: unit.title, pages: typedPages(pages, m) })}
								><Icon name="edit" size={14} />{u.edit}</button
							>
							{#if next}
								<button
									class="btn ghost sm"
									title={u.joinHint}
									disabled={busy}
									onclick={() => act(() => api.joinUnits(unit.id, next.id))}>{u.joinNext}</button
								>
							{/if}
							{#if unit.pages.length > 1}
								<select
									class="split"
									aria-label={u.split}
									title={u.splitHint}
									disabled={busy}
									onchange={(e) => {
										const at = Number(e.currentTarget.value);
										e.currentTarget.value = '';
										if (at > 0) void act(() => api.splitUnit(unit.id, at));
									}}
								>
									<option value="">{u.split}</option>
									{#each unit.pages.slice(1) as page, k (k)}
										<option value={k + 1}>{u.splitAt(pieceLabel(page, m))}</option>
									{/each}
								</select>
							{/if}
							<button
								class="btn ghost sm danger"
								aria-label={u.removeItem(titleOf(unit))}
								disabled={busy}
								onclick={() => act(() => api.deleteUnit(unit.id))}>{u.remove}</button
							>
						</div>
					{/if}
				</li>
			{/each}
		</ol>
		{#if editing?.id === null}
			{@render editor()}
		{:else}
			<button
				class="btn add"
				onclick={() =>
					(editing = {
						id: null,
						title: '',
						pages: shown?.kind === 'page' ? typedPage(shown) : ''
					})}><Icon name="plus" size={14} />{u.add}</button
			>
		{/if}
		{#if units.some((v) => v.shared.length > 0)}<p class="muted small">{u.precision}</p>{/if}
	{/if}
</section>

{#snippet editor()}
	{#if editing}
		<form
			class="edit"
			onsubmit={(e) => {
				e.preventDefault();
				void save();
			}}
		>
			<label class="field">
				<span class="field-label">{u.title}</span>
				<input type="text" bind:value={editing.title} />
			</label>
			<label class="field">
				<span class="field-label">{u.pages}</span>
				<input type="text" class="mono" bind:value={editing.pages} aria-describedby="pages-hint" />
				<span class="muted small" id="pages-hint">{u.pagesHint}</span>
			</label>
			<div class="actions">
				<button class="btn primary sm" type="submit" disabled={busy}>{u.save}</button>
				<button class="btn ghost sm" type="button" onclick={() => (editing = null)}
					>{u.cancel}</button
				>
			</div>
		</form>
	{/if}
{/snippet}

<style>
	.units {
		display: grid;
		gap: var(--space-3);
	}
	.error {
		margin: 0;
		color: var(--danger-text);
	}
	.note {
		margin: 0;
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	.empty {
		display: grid;
		gap: var(--space-3);
		justify-items: start;
		max-width: 40rem;
	}
	.empty p {
		margin: 0;
	}
	.small {
		font-size: var(--type-xs);
	}
	.bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
	.all {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.send {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	/* The items as a numbered list, set in the reading face like the entries. */
	.list {
		margin: 0;
		padding: 0 0 0 1.75rem;
		font-family: var(--font-text);
		color: var(--text-body);
	}
	.list li {
		padding: var(--space-3) var(--space-3) var(--space-3) var(--space-2);
		border-bottom: 1px solid var(--line-soft);
		border-radius: var(--radius-control);
	}
	.list li::marker {
		font-family: var(--font-mono);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The items on the page shown, marked as the entries are. */
	.list li.here {
		background: var(--surface-1);
		box-shadow: inset 3px 0 0 var(--marker);
	}
	.head {
		display: flex;
		align-items: baseline;
		gap: var(--space-2);
	}
	.item {
		display: flex;
		flex: 1;
		align-items: baseline;
		gap: var(--space-3);
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.item:disabled {
		cursor: default;
	}
	.item-title {
		flex: 1;
		font-size: var(--type-lg);
		font-weight: 600;
		line-height: 1.35;
		color: var(--text-bright);
	}
	.item:hover:not(:disabled) .item-title {
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.item-pages {
		flex: none;
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.shared {
		margin: var(--space-1) 0 0;
		color: var(--text-secondary);
		font-family: var(--font-ui);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1);
		margin-top: var(--space-2);
		font-family: var(--font-ui);
	}
	.split {
		font: inherit;
		font-size: var(--type-xs);
	}
	.edit {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-2) 0;
		font-family: var(--font-ui);
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
	.add {
		justify-self: start;
		border-style: dashed;
	}
</style>
