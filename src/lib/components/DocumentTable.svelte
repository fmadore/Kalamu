<script lang="ts">
	// A project's documents, found and sorted, each with its collections and what has been read
	// from it. Selected documents can be grouped, ungrouped, removed, or sent to New job, whose
	// job is then tied to them (`DocumentSelectionBar`). When an analysis has answered a choice
	// (whether a document speaks of the researcher's question), the table can be filtered by its
	// answer and that analysis exported as one table; while it is, each row shows the answer, its
	// reason and its passages under the title, with Keep and Set aside (`DocumentAnswer`).
	import DocumentAnswer from './DocumentAnswer.svelte';
	import DocumentSelectionBar from './DocumentSelectionBar.svelte';
	import Icon from './Icon.svelte';
	import MoreRows from './MoreRows.svelte';
	import type { ChoiceAnswers } from '#lib/bindings/ChoiceAnswers.js';
	import type { CollectionView } from '#lib/bindings/CollectionView.js';
	import type { DocumentRow } from '#lib/bindings/DocumentRow.js';
	import type { TagView } from '#lib/bindings/TagView.js';
	import { Action } from '#lib/action.svelte.js';
	import { formatDate, formatDateTime, messages } from '#lib/i18n/index.js';
	import { Paging, rowIndex } from '#lib/paging.svelte.js';
	import {
		shownDocuments,
		readDocumentTableView,
		sortDocuments,
		type DocumentSort
	} from '#lib/projects.js';
	import {
		answeredDocuments,
		answerOptions,
		chosenAnswer,
		decisionOf,
		decisionTags,
		rowAnswers,
		tableName
	} from '#lib/screening.js';
	import { Selection } from '#lib/selection.svelte.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { keepFor, recallFor } from '#lib/stored.js';
	import { askForFolders, pickTableSave } from '#lib/dialogs.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		projectName,
		rows,
		collections,
		tags,
		choices = [],
		collectionId,
		onchange,
		onerror
	}: {
		projectId: string;
		projectName: string;
		rows: DocumentRow[];
		collections: CollectionView[];
		tags: TagView[];
		/** The choices the project's analyses answered, to filter by. */
		choices?: ChoiceAnswers[];
		collectionId?: string;
		onchange: () => Promise<void>;
		onerror: (message: string) => void;
	} = $props();
	const m = messages();

	// The sort and the tag filtered on as the project's table was left, remembered on this
	// computer for the projects opened last.
	const VIEW_KEY = 'kalamu.project.table';
	// svelte-ignore state_referenced_locally
	const left = recallFor(VIEW_KEY, projectId, readDocumentTableView);
	let query = $state('');
	let sort = $state<DocumentSort>(left?.sort ?? 'title');
	const selection = new Selection();
	/** A change to the documents chosen: the choice is cleared and the project read again. */
	const action = new Action({
		after: async () => {
			selection.clear();
			await onchange();
		},
		report: (message) => onerror(message)
	});
	/** A decision given from a row: the project read again, the choice of documents kept. */
	const deciding = new Action({ report: (message) => onerror(message) });
	let tag = $state(left?.tag ?? '');
	/** The answer filtered on, as `answerKey` names it; empty for none. */
	let answer = $state('');
	$effect(() => keepFor(VIEW_KEY, projectId, { sort, tag }, 100));
	/** The new name typed for the tag filtered on, while renaming it. */
	let renaming = $state<string | null>(null);

	// A tag no document carries any longer stops filtering; so does an answer no longer given.
	const filterTag = $derived(tags.some((t) => t.name === tag) ? tag : '');
	const answered = $derived(answeredDocuments(choices, answer));
	const filterAnswer = $derived(answered ? answer : '');
	const answerGroups = $derived(answerOptions(choices, m));
	const rowAnswered = $derived(rowAnswers(choices, filterAnswer));
	const shown = $derived(
		sortDocuments(
			shownDocuments(rows, collectionId, query, filterTag).filter(
				(r) => !answered || answered.has(r.document.id)
			),
			sort
		)
	);
	// A long list is drawn a page at a time, from its first page again when filtered or sorted.
	const paging = new Paging();
	const drawn = $derived(paging.of(shown));
	let table = $state<HTMLTableElement | null>(null);
	$effect(() => {
		void [query, filterTag, filterAnswer, sort, collectionId];
		paging.reset();
	});
	const chosen = $derived(selection.of(shown, (r) => r.document.id));
	const allChosen = $derived(shown.length > 0 && chosen.length === shown.length);
	const names = $derived(new Map(collections.map((c) => [c.collection.id, c.collection.name])));

	/** Renames the tag filtered on across the project, and keeps filtering on it. */
	async function renameTag(e: SubmitEvent) {
		e.preventDefault();
		const from = filterTag;
		const to = renaming?.trim() ?? '';
		renaming = null;
		if (!from || !to || to === from) return;
		await action.run(
			() => api.renameTag(projectId, from, to),
			async (renamed) => {
				tag = renamed.name;
				await onchange();
				toast(m.projects.tagRenamed(renamed.changed));
			}
		);
	}

	/** The analysis filtered on, one row per document with the researcher's tags. */
	async function exportAnswers() {
		const found = chosenAnswer(choices, filterAnswer);
		if (!found) return;
		const { group } = found;
		const path = await pickTableSave(tableName(projectName, group));
		if (!path) return;
		await action.run(async () => {
			// The CSV goes beside the workbook: in the Mac App Store's build, its folder is asked for.
			if (!(await askForFolders([path]))) return;
			await api.exportAnswers(projectId, group.job_id, group.setup_id, path);
			toast(m.screening.exported);
		}, null);
	}

	/** Keep or Set aside one document from its row; the decision it already has, taken off. */
	async function decide(id: string, tags: string[], keep: boolean) {
		const { add, remove } = decisionTags(keep, m);
		const again = decisionOf(tags, m) === (keep ? 'keep' : 'aside');
		await deciding.run(
			async () => {
				if (again) return api.untagDocuments(projectId, [id], add);
				await api.untagDocuments(projectId, [id], remove);
				await api.tagDocuments(projectId, [id], add);
			},
			async () => {
				await onchange();
				toast(again ? m.screening.decisionTakenOff(add) : m.screening.decided(add, 1));
			}
		);
	}

	function open(id: string) {
		go({ name: 'projects', projectId, collectionId, documentId: id });
	}
</script>

{#if rows.length === 0}
	<p class="muted">{m.projects.noDocuments}</p>
{:else}
	<div class="table-filters">
		<label class="table-search">
			<Icon name="search" size={14} />
			<input
				type="text"
				bind:value={query}
				data-find
				placeholder={m.projects.search}
				aria-label={m.projects.search}
			/>
		</label>
		{#if tags.length > 0}
			<select bind:value={() => filterTag, (v) => (tag = v)} aria-label={m.projects.tagFilter}>
				<option value="">{m.projects.anyTag}</option>
				{#each tags as t (t.name)}
					<option value={t.name}>{m.projects.tagCount(t.name, t.documents)}</option>
				{/each}
			</select>
			{#if filterTag && renaming !== null}
				<form class="rename-tag" onsubmit={renameTag}>
					<!-- svelte-ignore a11y_autofocus -->
					<input
						type="text"
						bind:value={renaming}
						aria-label={m.projects.renameTagTo(filterTag)}
						placeholder={m.projects.renameTagTo(filterTag)}
						autofocus
						onkeydown={(e) => e.key === 'Escape' && (renaming = null)}
					/>
					<button class="btn sm" disabled={!renaming.trim()}>{m.projects.renameTag}</button>
				</form>
			{:else if filterTag}
				<button class="btn ghost sm" onclick={() => (renaming = filterTag)}
					><Icon name="edit" size={14} />{m.projects.renameTag}</button
				>
			{/if}
		{/if}
		{#if answerGroups.length > 0}
			<select bind:value={() => filterAnswer, (v) => (answer = v)} aria-label={m.screening.filter}>
				<option value="">{m.screening.anyAnswer}</option>
				{#each answerGroups as g, i (i)}
					<optgroup label={g.title}>
						{#each g.options as o (o.key)}
							<option value={o.key}>{o.label}</option>
						{/each}
					</optgroup>
				{/each}
			</select>
			{#if filterAnswer}
				<button class="btn ghost sm" disabled={action.busy} onclick={exportAnswers}
					><Icon name="download" size={14} />{m.screening.exportTable}</button
				>
			{/if}
		{/if}
		<label class="sort">
			<span>{m.projects.sortBy}</span>
			<select bind:value={sort}>
				<option value="title">{m.projects.sortTitle}</option>
				<option value="added">{m.projects.sortAdded}</option>
				<option value="run">{m.projects.sortRun}</option>
			</select>
		</label>
	</div>

	{#if chosen.length > 0}
		<DocumentSelectionBar
			{projectId}
			{projectName}
			{collectionId}
			{collections}
			{tags}
			{chosen}
			{action}
			deciding={!!filterAnswer}
			onclear={() => selection.clear()}
			{onchange}
		/>
	{/if}

	{#if shown.length === 0}
		<p class="muted">{m.projects.noMatch}</p>
	{:else}
		<table class="data-table" bind:this={table} aria-rowcount={shown.length + 1}>
			<thead>
				<tr aria-rowindex={1}>
					<th class="check">
						<input
							type="checkbox"
							checked={allChosen}
							aria-label={m.projects.selectAll}
							onchange={() => selection.toggleAll(shown.map((r) => r.document.id))}
						/>
					</th>
					<th>{m.projects.columnTitle}</th>
					<th>{m.projects.columnKind}</th>
					<th>{m.projects.columnCollections}</th>
					<th>{m.projects.columnResults}</th>
					<th>{m.projects.columnDate}</th>
				</tr>
			</thead>
			<tbody>
				{#each drawn as row, i (row.document.id)}
					{@const doc = row.document}
					{@const said = rowAnswered?.get(doc.id)}
					<tr data-row={i} aria-rowindex={rowIndex(i)} class:chosen={selection.has(doc.id)}>
						<td class="check">
							<input
								type="checkbox"
								checked={selection.has(doc.id)}
								aria-label={m.projects.select(doc.title)}
								onchange={() => selection.toggle(doc.id)}
							/>
						</td>
						<td
							><div class="title">
								<button class="link" onclick={() => open(doc.id)}>{doc.title}</button>
								<span class="file mono" title={doc.source.path}>{doc.source.file_name}</span>
								{#if row.missing}
									<span class="chip danger" title={m.projects.missingHint}
										>{m.projects.missing}</span
									>
								{/if}
								{#if row.same_content > 0}
									<span class="chip warn">{m.projects.sameContent(row.same_content)}</span>
								{/if}
								{#if said}
									<DocumentAnswer
										title={doc.title}
										answer={said}
										decision={decisionOf(doc.tags, m)}
										busy={deciding.busy}
										ondecide={(keep) => decide(doc.id, doc.tags, keep)}
									/>
								{/if}
							</div></td
						>
						<td class="muted">{m.projects.kind[doc.source.kind]}</td>
						<td>
							<div class="chips">
								{#each row.collections as id (id)}
									<span class="chip">{names.get(id)}</span>
								{/each}
								{#each doc.tags as name (name)}
									<button
										class="chip tag"
										class:on={name === filterTag}
										aria-pressed={name === filterTag}
										onclick={() => (tag = name === filterTag ? '' : name)}
										><Icon name="tag" size={11} />{name}</button
									>
								{/each}
							</div>
						</td>
						<td class="results">
							{#if row.tasks.length === 0}
								<span class="muted">{m.projects.noResults}</span>
							{:else}
								{#each row.tasks as t (t.task)}
									{@const said = m.projects.taskResults(
										m.task[t.task],
										t.results,
										t.reviewed,
										t.preferred
									)}
									<!-- Each mark says what it counts on pointing. The whole is a button that opens
									     the document: named by the sentence, which it also shows when the keyboard
									     reaches it. -->
									<button
										class="result"
										class:reviewed={t.reviewed > 0}
										title={said}
										aria-label={said}
										data-said={said}
										onclick={() => open(doc.id)}
									>
										<span class="mark" aria-hidden="true" title={m.task[t.task]}
											><Icon name={t.task} size={13} /></span
										>
										<span class="num" aria-hidden="true" title={m.projects.resultCount(t.results)}
											>{t.results}</span
										>
										{#if t.reviewed > 0}<span
												class="mark"
												aria-hidden="true"
												title={m.projects.reviewedMark(t.reviewed)}
												><Icon name="check" size={12} /></span
											>{/if}
										{#if t.preferred}<span
												class="mark"
												aria-hidden="true"
												title={m.projects.preferredMark}><Icon name="star" size={12} filled /></span
											>{/if}
									</button>
								{/each}
							{/if}
						</td>
						<td class="num muted date" title={formatDateTime(doc.created_at)}
							>{formatDate(doc.created_at)}</td
						>
					</tr>
				{/each}
			</tbody>
		</table>
		<MoreRows {paging} total={shown.length} {table} />
	{/if}
{/if}

<style>
	.rename-tag {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	.rename-tag input {
		width: 14rem;
	}
	.sort {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	/* Collections and tags in one row that wraps, centred on each other: in the line of text a
	   chip that starts with an icon sat lower than one without. */
	.chips {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1);
	}
	.chip.tag {
		border: 0;
		cursor: pointer;
		gap: 0.25rem;
		font-family: inherit;
	}
	.chip.tag:hover,
	.chip.tag.on {
		background: var(--wash);
		color: var(--text-bright);
	}
	.title {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-1) var(--space-2);
	}
	.link {
		padding: 0;
		border: 0;
		background: none;
		color: var(--text-bright);
		font-weight: 500;
		text-align: left;
		cursor: pointer;
	}
	.link:hover {
		text-decoration: underline;
		text-underline-offset: 0.18em;
	}
	.file {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.results {
		white-space: nowrap;
	}
	.result {
		position: relative;
		display: inline-flex;
		align-items: center;
		gap: 2px;
		margin-right: var(--space-2);
		padding: 1px 3px;
		border: 0;
		border-radius: var(--radius-tag);
		background: none;
		color: var(--text-secondary);
		font: inherit;
		cursor: pointer;
	}
	.result:hover {
		background: var(--wash);
	}
	/* Reached from the keyboard, the marks say in words what they count, under them. */
	.result:focus-visible::after {
		content: attr(data-said);
		position: absolute;
		top: calc(100% + 4px);
		right: 0;
		z-index: 2;
		width: max-content;
		max-width: 20rem;
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-tag);
		background: var(--surface-2);
		box-shadow: var(--shadow-flyout);
		color: var(--text-bright);
		font-size: var(--type-xs);
		white-space: normal;
		text-align: left;
	}
	.result.reviewed {
		color: var(--ok-text);
	}
	.result .mark {
		display: inline-grid;
	}
	.date {
		white-space: nowrap;
	}
	@media (max-width: 760px) {
		th:nth-child(3),
		td:nth-child(3),
		th:nth-child(6),
		td:nth-child(6) {
			display: none;
		}
		/* The table fits the window: results wrap, each count kept with its mark, and long
		   names break. */
		.results {
			white-space: normal;
		}
		td {
			overflow-wrap: anywhere;
		}
	}
</style>
