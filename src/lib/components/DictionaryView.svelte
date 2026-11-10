<script lang="ts">
	// A project's dictionary: the names its analyses listed, field by field, gathered into
	// entries by the researcher. Every decision is the researcher's and is kept; spellings that
	// look alike are only pointed out, and a model's proposed merges wait for the researcher.
	import DictionaryDetails from './DictionaryDetails.svelte';
	import DictionaryProposals from './DictionaryProposals.svelte';
	import Icon from './Icon.svelte';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { DictionaryRow } from '#lib/bindings/DictionaryRow.js';
	import type { DictionaryView } from '#lib/bindings/DictionaryView.js';
	import type { Evidence } from '#lib/bindings/Evidence.js';
	import {
		itemOf,
		labelChoices,
		proposedLabel,
		shownRows,
		type RowFilter
	} from '#lib/dictionary.js';
	import { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { Selection } from '#lib/selection.svelte.js';
	import { toast } from '#lib/state.svelte.js';
	import { api, askForFolders, pickJsonSave } from '#lib/tauri.js';

	let {
		projectId,
		projectName,
		collections
	}: { projectId: string; projectName: string; collections: CollectionView[] } = $props();
	const m = messages();
	const filters: RowFilter[] = ['all', 'to_review', 'similar', 'reviewed', 'unseen'];

	let view = $state<DictionaryView | null>(null);
	/** After a change, the selection and the merge typed are dropped and the rows read again. */
	const action = new Action({
		after: async () => {
			selection.clear();
			merging = null;
			await load();
			if (open) await showEvidence(open);
		}
	});
	let scope = $state('');
	let field = $state('');
	let query = $state('');
	let filter = $state<RowFilter>('all');
	const selection = new Selection();
	let merging = $state<string | null>(null);
	let open = $state<string | null>(null);
	let evidence = $state<Evidence[]>([]);

	const namespace = $derived(
		view?.namespaces.some((n) => n.key === field) ? field : (view?.namespaces[0]?.key ?? '')
	);
	const rows = $derived(view ? shownRows(view.rows, namespace, query, filter) : []);
	const byKey = $derived(new Map((view?.rows ?? []).map((r) => [r.key, r])));
	const chosen = $derived(selection.of(rows, (r) => r.key));
	const allChosen = $derived(rows.length > 0 && chosen.length === rows.length);

	/** Raised by each reading, so a slower answer for an earlier scope is dropped. */
	let viewTicket = 0;

	async function load() {
		const ticket = ++viewTicket;
		try {
			const read = await api.dictionaryView(projectId, scope || null);
			if (ticket === viewTicket) view = read;
		} catch (e) {
			if (ticket === viewTicket) action.fail(e);
		}
	}

	function labelOf(key: string): string {
		return byKey.get(key)?.label ?? key;
	}

	/** Raised by each row opened or closed, so the evidence for a row opened earlier and read
	 * later is not shown under the one opened since. */
	let evidenceTicket = 0;

	async function showEvidence(key: string) {
		const ticket = ++evidenceTicket;
		const row = byKey.get(key);
		if (!row) {
			open = null;
			return;
		}
		open = key;
		const found = await api.dictionaryEvidence(
			projectId,
			row.namespace,
			row.forms.map((f) => f.form)
		);
		if (ticket === evidenceTicket) evidence = found;
	}

	async function toggleOpen(row: DictionaryRow) {
		if (open === row.key) {
			evidenceTicket++;
			open = null;
			return;
		}
		try {
			await showEvidence(row.key);
		} catch (e) {
			action.fail(e);
		}
	}

	function merge(event: SubmitEvent) {
		event.preventDefault();
		if (merging === null) return;
		const label = merging;
		void action.run(() => api.dictionaryMerge(projectId, namespace, chosen.map(itemOf), label));
	}

	async function exportAll() {
		const path = await pickJsonSave(m.dictionary.fileName(projectName));
		if (!path) return;
		try {
			// The CSV goes beside the JSON: in the Mac App Store's build, its folder is asked for.
			if (!(await askForFolders([path]))) return;
			await api.exportDictionary(projectId, path);
			toast(m.dictionary.exported);
		} catch (e) {
			action.fail(e);
		}
	}

	$effect(() => {
		void scope;
		void load();
	});
</script>

<div class="dictionary">
	<div class="head">
		<h2>{m.dictionary.title}</h2>
		{#if view && view.rows.length > 0}
			<button class="btn sm" onclick={exportAll}
				><Icon name="download" size={14} />{m.dictionary.export}</button
			>
		{/if}
	</div>
	<p class="muted lede">{m.dictionary.lede}</p>
	{#if action.error}<p class="chip danger banner" role="alert">{action.error}</p>{/if}

	{#if view}
		<p class="coverage num">{m.dictionary.coverage(view.documents_read, view.documents)}</p>
		<div class="table-filters">
			{#if view.namespaces.length > 0}
				<label class="pick">
					<span>{m.dictionary.field}</span>
					<select
						value={namespace}
						onchange={(e) => {
							field = e.currentTarget.value;
							selection.clear();
							open = null;
						}}
					>
						{#each view.namespaces as n (n.key)}
							<option value={n.key}>{m.dictionary.fieldCount(n.label, n.rows, n.to_review)}</option>
						{/each}
					</select>
				</label>
			{/if}
			{#if collections.length > 0}
				<label class="pick">
					<span>{m.dictionary.scope}</span>
					<select bind:value={scope}>
						<option value="">{m.dictionary.wholeProject}</option>
						{#each collections as c (c.collection.id)}
							<option value={c.collection.id}>{c.collection.name}</option>
						{/each}
					</select>
				</label>
			{/if}
			<label class="table-search">
				<Icon name="search" size={14} />
				<input
					type="text"
					bind:value={query}
					data-find
					placeholder={m.dictionary.search}
					aria-label={m.dictionary.search}
				/>
			</label>
			<label class="pick">
				<span>{m.dictionary.show}</span>
				<select bind:value={filter}>
					{#each filters as f (f)}
						<option value={f}>{m.dictionary.filter[f]}</option>
					{/each}
				</select>
			</label>
		</div>

		{#if view.rows.length === 0}
			<p class="muted">{m.dictionary.empty}</p>
		{:else}
			<DictionaryProposals
				{projectId}
				{namespace}
				fieldLabel={view.namespaces.find((n) => n.key === namespace)?.label ?? namespace}
				{scope}
				onchange={load}
			/>
			{#if chosen.length > 0}
				<div class="selection-bar" role="toolbar" aria-label={m.dictionary.selected(chosen.length)}>
					<span class="num">{m.dictionary.selected(chosen.length)}</span>
					<button class="btn ghost sm" onclick={() => selection.clear()}
						>{m.dictionary.clearSelection}</button
					>
					<span class="grow"></span>
					{#if merging !== null}
						<form class="merge" onsubmit={merge}>
							<input
								type="text"
								list="merge-labels"
								bind:value={merging}
								aria-label={m.dictionary.mergeLabel}
							/>
							<datalist id="merge-labels">
								{#each labelChoices(chosen) as label (label)}
									<option value={label}></option>
								{/each}
							</datalist>
							<button class="btn primary sm" type="submit" disabled={!merging.trim()}
								>{m.dictionary.mergeConfirm}</button
							>
							<button class="btn ghost sm" type="button" onclick={() => (merging = null)}
								>{m.dictionary.cancel}</button
							>
						</form>
					{:else}
						<button
							class="btn primary sm"
							disabled={chosen.length < 2}
							onclick={() => (merging = proposedLabel(chosen))}>{m.dictionary.merge}</button
						>
						<button
							class="btn sm"
							disabled={chosen.length !== 2}
							title={m.dictionary.keepApartHint}
							onclick={() =>
								action.run(() =>
									api.dictionaryKeepApart(
										projectId,
										namespace,
										itemOf(chosen[0]),
										itemOf(chosen[1])
									)
								)}>{m.dictionary.keepApart}</button
						>
						<button
							class="btn sm"
							onclick={() =>
								action.run(() =>
									api.dictionarySetReviewed(projectId, namespace, chosen.map(itemOf), true)
								)}><Icon name="check" size={14} />{m.dictionary.markReviewed}</button
						>
						<button
							class="btn ghost sm"
							onclick={() =>
								action.run(() =>
									api.dictionarySetReviewed(projectId, namespace, chosen.map(itemOf), false)
								)}>{m.dictionary.markToReview}</button
						>
					{/if}
				</div>
			{/if}

			{#if rows.length === 0}
				<p class="muted">{m.dictionary.noMatch}</p>
			{:else}
				<table class="data-table">
					<thead>
						<tr>
							<th class="check">
								<input
									type="checkbox"
									checked={allChosen}
									aria-label={m.dictionary.selectAll}
									onchange={() => selection.toggleAll(rows.map((r) => r.key))}
								/>
							</th>
							<th>{m.dictionary.columnLabel}</th>
							<th class="num">{m.dictionary.columnDocuments}</th>
							<th title={m.dictionary.agreementHint}>{m.dictionary.columnAgreement}</th>
							<th>{m.dictionary.columnStatus}</th>
						</tr>
					</thead>
					<tbody>
						{#each rows as row (row.key)}
							<tr class:chosen={selection.has(row.key)}>
								<td class="check">
									<input
										type="checkbox"
										checked={selection.has(row.key)}
										aria-label={m.dictionary.select(row.label)}
										onchange={() => selection.toggle(row.key)}
									/>
								</td>
								<td>
									<button
										class="link label"
										aria-expanded={open === row.key}
										aria-label={m.dictionary.details(row.label)}
										onclick={() => toggleOpen(row)}>{row.label}</button
									>
									{#if row.forms.length > 1 || (row.forms[0] && row.forms[0].form !== row.label)}
										<div class="forms">
											{#each row.forms as f (f.form)}
												<span class="chip">{f.form}</span>
											{/each}
										</div>
									{/if}
									{#if row.similar.length > 0}
										<p class="hint" title={m.dictionary.similarHint}>
											{m.dictionary.similar}
											{row.similar.map(labelOf).join(', ')} ·
											<button class="link" onclick={() => selection.add([row.key, ...row.similar])}
												>{m.dictionary.selectSimilar}</button
											>
										</p>
									{/if}
									{#if row.apart.length > 0}
										<p class="hint">
											{m.dictionary.apart}
											{#each row.apart as other (other)}
												<span class="apart"
													>{labelOf(other)}<button
														class="untie"
														aria-label={m.dictionary.withdraw(labelOf(other))}
														title={m.dictionary.withdraw(labelOf(other))}
														onclick={() =>
															row.entry &&
															action.run(() =>
																api.dictionaryUndoApart(projectId, row.entry ?? '', other)
															)}><Icon name="close" size={11} /></button
													></span
												>
											{/each}
										</p>
									{/if}
								</td>
								<td class="num">{row.documents}</td>
								<td class="num muted" title={m.dictionary.agreementHint}
									>{m.dictionary.agreement(row.listed, row.read)}</td
								>
								<td>
									{#if row.unseen}
										<span class="chip warn" title={m.dictionary.unseenHint}
											>{m.dictionary.unseen}</span
										>
									{:else if row.reviewed}
										<span class="chip ok">{m.dictionary.reviewed}</span>
									{:else}
										<span class="muted">{m.dictionary.toReview}</span>
									{/if}
								</td>
							</tr>
							{#if open === row.key}
								<tr class="details">
									<td></td>
									<td colspan="4">
										<DictionaryDetails
											{projectId}
											{row}
											{evidence}
											act={(work) => action.run(work)}
										/>
									</td>
								</tr>
							{/if}
						{/each}
					</tbody>
				</table>
			{/if}
		{/if}
	{/if}
</div>

<style>
	.dictionary {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-3);
	}
	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	h2 {
		margin: 0;
	}
	.lede,
	.coverage {
		margin: 0;
		max-width: 48rem;
	}
	.coverage {
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
	.grow {
		flex: 1;
	}
	.merge {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.label {
		font-weight: 500;
		color: var(--text-bright);
		text-align: left;
	}
	.forms {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
		margin-top: var(--space-1);
	}
	.hint {
		margin: var(--space-1) 0 0;
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.apart {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		margin-left: var(--space-1);
	}
	.untie {
		display: inline-flex;
		padding: 0 2px;
		border: 0;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
	}
	.untie:hover {
		color: var(--text-bright);
	}
	tr.details td {
		background: var(--surface-2);
	}
</style>
