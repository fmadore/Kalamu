<script lang="ts">
	// The review of an analysis: the document beside its answers.
	// The first set-up's answers lead. In a comparison every value carries the sigla of the
	// set-ups that gave it, so agreement reads at a glance; pointing at a name finds it in the
	// document. Corrections are kept apart from the model's answers. A search keeps the fields
	// whose answers or reasons hold its words, marked where they stand, and marks them in a text
	// document too. A name in a list renames or removes it at once, in the corrected answers of
	// every set-up that found it, from its menu: a click, Enter, a right click or the menu key.
	// The names of an answer are one tab stop, ← and → going from one to the next; a name with
	// the focus is found in the document as one pointed at. What the models missed is added at
	// the end of a list (`AddItems`), in every set-up's corrected answers; a name no model listed
	// is the researcher's, underlined in their blue and said to be (`ListAnswer` draws a list's
	// names, `ListedNames` changes them). An analysis whose fields are
	// one repeating group gives rows, page by page, reviewed in a table of its own
	// (`RecordsReview`), which adds no names this way. An answer that quotes the text (an
	// interview's themes) is a card per entry (`CitedAnswer`, changed by `CitedAnswers`).
	import { onMount, tick } from 'svelte';
	import CitedAnswer from './CitedAnswer.svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import FieldInput from './FieldInput.svelte';
	import Icon from './Icon.svelte';
	import ItemMenu from './ItemMenu.svelte';
	import ListAnswer from './ListAnswer.svelte';
	import RecordsReview from './RecordsReview.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import SearchBar from './SearchBar.svelte';
	import SiglaKey from './SiglaKey.svelte';
	import { collate, holdsWords, type Witness } from '#lib/analysis.js';
	import { siglum } from '#lib/apparatus.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { CitedAnswers } from '#lib/citedAnswers.svelte.js';
	import { type Draft, fromDraft, toDraft } from '#lib/fields.js';
	import { clearFound, markFound } from '#lib/found.js';
	import { messages } from '#lib/i18n/index.js';
	import { ListedNames } from '#lib/listedNames.svelte.js';
	import { pieceCost } from '#lib/pieces.js';
	import { groupOf } from '#lib/records.js';
	import { ownRegions } from '#lib/regions.svelte.js';
	import { Review } from '#lib/review.svelte.js';
	import { find, searchable } from '#lib/search.js';
	import { guardDraft, type ReviewFind } from '#lib/state.svelte.js';

	let {
		jobId,
		sourceId,
		find: opening,
		start
	}: {
		jobId: string;
		sourceId: string;
		find?: ReviewFind;
		/** The piece to open on, from 0: a page of records chosen in the list of what to check. */
		start?: number;
	} = $props();
	const m = messages();
	// svelte-ignore state_referenced_locally
	const review = new Review(jobId, sourceId);

	/** The page of a scan shown. */
	let page = $state(0);
	let hovered = $state<string | null>(null);
	/** The answers being corrected, and the draft they were opened on, to tell whether they
	 * changed. */
	let editing = $state<{ setupId: string; draft: Draft; from: string } | null>(null);

	const source = $derived(review.source);
	const setups = $derived(review.setups);
	const fields = $derived(setups[0]?.prompt.schema?.fields ?? []);
	/** Records: rows page by page, reviewed in their own table. */
	const records = $derived(groupOf(setups[0]?.prompt.schema) !== null);
	const pages = $derived(source?.kind === 'pdf' ? (source.page_count ?? 1) : 1);
	const viewOf = (s: Setup | undefined) => review.views.find((v) => v.setup_id === s?.id);
	const answersOf = (v: PieceView | undefined): FieldAnswer[] =>
		v?.corrected_fields ?? v?.ai_fields ?? [];
	const lead = $derived(viewOf(setups[0]));
	const witnesses: Witness[] = $derived(
		setups.map((s, i) => ({ siglum: siglum(i), answers: answersOf(viewOf(s)) }))
	);
	const comparing = $derived(setups.length > 1);
	/** Each set-up's own answers, from the model: a name none of them lists is the researcher's. */
	const models = $derived(setups.map((s) => viewOf(s)?.ai_fields ?? []));

	let searching = $state(false);
	let query = $state('');
	$effect(() => {
		if (!searching) query = '';
	});
	const filtering = $derived(searching && searchable(query));
	/** The fields listed: those whose answers hold the words searched for, or all of them. */
	const shown = $derived(
		filtering ? fields.filter((f) => holdsWords(f.key, witnesses, query)) : fields
	);
	/** How many times the words are found in a text document. */
	const inDocument = $derived(
		filtering && source?.kind === 'text' && review.text ? find(review.text, query).length : 0
	);
	let answersPane = $state<HTMLElement | null>(null);

	// The words searched for, marked in the answers listed.
	$effect(() => {
		void shown;
		if (!answersPane || !filtering || editing) return;
		markFound(answersPane, '.answer p', query);
		return clearFound;
	});

	// Opened from a project's search on an answer: that field brought into view, once.
	let opened = false;
	$effect(() => {
		const key = opening?.field;
		if (opened || !key || !answersPane || !shown.some((f) => f.key === key)) return;
		opened = true;
		void tick().then(() =>
			answersPane
				?.querySelector(`[data-field="${CSS.escape(key)}"]`)
				?.scrollIntoView({ block: 'start' })
		);
	});

	/** The researcher's regions on the scan page shown (#84). */
	// svelte-ignore state_referenced_locally
	const regions = ownRegions(jobId, sourceId, () =>
		review.paged && !records ? { page: page + 1, half: null } : null
	);

	// A PDF or image is shown page by page; a text document as text. Records show their own.
	$effect(() => {
		if (!records) review.showPage(page);
	});

	function step(delta: number) {
		page = Math.min(Math.max(page + delta, 0), pages - 1);
	}

	function startEditing(setup: Setup) {
		const draft = toDraft(fields, answersOf(viewOf(setup)));
		editing = { setupId: setup.id, draft, from: JSON.stringify(draft) };
	}

	async function save(setup: Setup, answers: FieldAnswer[] | null, reviewed: boolean) {
		if (await review.save({ setup, index: 0, fields: answers, reviewed })) editing = null;
	}

	/** The names of the list answers: their menu, their tab stops, and what changes them. */
	const names = new ListedNames({
		review,
		witnesses: () => witnesses,
		pane: () => answersPane,
		editing: () => editing !== null,
		unhover: () => (hovered = null)
	});

	/** The answers that quote the text: the text read, where each quotation plays, and what
	 * changes them. */
	const cited = new CitedAnswers(review, () => answersPane);

	function toggleReviewed() {
		const first = setups[0];
		if (first && lead) void save(first, lead.corrected_fields, !lead.reviewed);
	}

	const kindLabel = (f: Field) =>
		f.kind.type === 'entries' ? m.fields.kinds.list : m.fields.kinds[f.kind.type];
	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void review.load().then(() => cited.load());
		const release = guardDraft({
			dirty: () => editing !== null && JSON.stringify(editing.draft) !== editing.from,
			discard: () => (editing = null)
		});
		// Opened from a project's search: its words, searched here too.
		if (opening) {
			searching = true;
			query = opening.query;
		}
		return release;
	});
</script>

{#snippet actions()}
	{#if lead && setups[0] && lead.status !== 'failed' && !editing}
		{@const first = setups[0]}
		<button class="btn" onclick={() => startEditing(first)}
			><Icon name="edit" size={14} />{m.review.edit}</button
		>
	{/if}
{/snippet}

{#snippet tools()}
	{setups[0]?.prompt.preset_name ?? ''}
{/snippet}

{#snippet searchBar()}
	<SearchBar
		bind:query
		placeholder={source?.kind === 'text' ? m.review.searchAnswersAndText : m.review.searchAnswers}
		summary={!filtering
			? ''
			: shown.length === 0 && inDocument === 0
				? m.review.notFound
				: source?.kind === 'text'
					? `${m.review.answersFound(shown.length, fields.length)} · ${m.review.inDocument(inDocument)}`
					: m.review.answersFound(shown.length, fields.length)}
		onclose={() => (searching = false)}
	/>
{/snippet}

{#snippet foot()}
	<span class="mono">{lead?.model_version ?? setups[0]?.model ?? ''}{pieceCost(lead, m)}</span>
{/snippet}

{#if records}
	<RecordsReview {review} {start} />
{:else}
	<!-- Each opening is a menu of its own. A menu says it has closed a moment after it hides, so
	     one opened again on the same name in that moment would otherwise be taken for the one
	     that closed, and never show. -->
	{#key names.menu}
		{#if names.menu}
			{@const at = names.menu}
			<ItemMenu
				value={at.value}
				renamable={at.renamable}
				x={at.x}
				y={at.y}
				onrename={(to) => void names.change(at.key, at.value, to)}
				onremove={() => void names.change(at.key, at.value, null)}
				onclose={() => {
					if (names.menu === at) names.menu = null;
				}}
			/>
		{/if}
	{/key}

	<ReviewFrame
		{jobId}
		{sourceId}
		jobName={review.job?.name ?? null}
		fileName={source?.file_name ?? ''}
		error={review.error}
		{actions}
		search={searchBar}
		bind:searching
		bind:textPane={answersPane}
		reviewed={lead && lead.status !== 'failed' && !editing ? lead.reviewed : null}
		onreviewed={toggleReviewed}
		onstep={pages > 1 ? step : undefined}
		source={source?.kind === 'text' ? 'text' : 'scan'}
		piece={pages > 1 ? m.review.page(String(page + 1)) : null}
		paging={{ at: page, count: pages }}
		preview={review.preview}
		regions={regions.place ? regions : undefined}
		text={cited.source?.text ?? review.text}
		span={cited.span}
		find={hovered ?? (filtering ? query : '')}
		found={hovered === null && filtering}
		witness={{
			label: setups[0]?.label ?? '',
			siglum: comparing ? siglum(0) : null,
			corrected: lead?.corrected_fields != null
		}}
		label={m.review.answers}
		{tools}
		{foot}
	>
		{#if editing && editedSetup}
			{@const target = editedSetup}
			{@const draft = editing.draft}
			{@const kept = viewOf(target)}
			<CorrectionEditor
				label={target.label}
				siglum={comparing ? siglum(setups.indexOf(target)) : null}
				onsave={() => save(target, fromDraft(fields, draft, answersOf(kept)), true)}
				ondiscard={() => (editing = null)}
				onrevert={kept?.corrected_fields ? () => save(target, null, kept.reviewed) : undefined}
			>
				<div>
					{#each fields as f (f.key)}
						<div class="answer">
							<h2>{f.label} <span class="kind">{kindLabel(f)}</span></h2>
							{#if f.kind.type === 'cited'}
								<p class="muted none">{m.cited.inCards}</p>
							{:else}
								<FieldInput field={f} bind:value={draft[f.key]} />
							{/if}
						</div>
					{/each}
				</div>
			</CorrectionEditor>
		{:else if !lead || (lead.ai_fields.length === 0 && lead.status !== 'failed')}
			<p class="muted">{m.review.notDone}</p>
		{:else if lead.status === 'failed'}
			<p class="error">{lead.error}</p>
		{:else}
			{#if shown.length === 0}<p class="muted">{m.review.noAnswerFound}</p>{/if}
			{#each shown as f (f.key)}
				{@const readings = collate(f.key, witnesses)}
				<div class="answer" data-field={f.key} class:opened={opening?.field === f.key}>
					<h2>{f.label}</h2>
					{#if f.kind.type === 'cited'}
						<CitedAnswer
							field={f}
							{setups}
							answersOf={(s) => {
								const v = viewOf(s);
								return v && v.status !== 'failed' ? answersOf(v) : null;
							}}
							{comparing}
							{cited}
						/>
					{:else if readings.length === 0}
						<p class="muted none">{m.review.noneFound}</p>
					{/if}
					{#if f.kind.type === 'list' || f.kind.type === 'choices'}
						<ListAnswer
							field={f}
							{readings}
							{names}
							{models}
							{comparing}
							count={setups.length}
							onhover={(value) => {
								hovered = value;
								if (value !== null) cited.span = null;
							}}
						/>
					{:else if readings.length > 0 && f.kind.type !== 'cited'}
						{#each readings as r (r.value)}
							<p class:prose={f.kind.type === 'text'} class:choice={f.kind.type === 'choice'}>
								{#if comparing}<i class="siglum">{r.sigla.join(' ')}</i>{/if}
								{r.value}
							</p>
						{/each}
					{/if}
					{#if f.reason}
						{#each witnesses as w (w.siglum)}
							{@const reason = w.answers.find((a) => a.key === f.key)?.reason}
							{#if reason}
								<p class="reason">
									{#if comparing}<i class="siglum">{w.siglum}</i>{/if}
									{reason}
								</p>
							{/if}
						{/each}
					{/if}
				</div>
			{/each}
		{/if}

		{#if !editing}
			<span class="sr-only" id="item-keys">{m.review.itemKeys}</span>
			<span class="sr-only" id="choice-keys">{m.review.choiceKeys}</span>
		{/if}

		{#if !editing && comparing}
			<SiglaKey
				{setups}
				viewOf={(s) => viewOf(s)}
				answered={(v) => v.status !== 'failed'}
				corrected={(v) => v.corrected_fields != null}
				onedit={(s) => startEditing(s)}
				onreviewed={(s, v, on) => save(s, v.corrected_fields, on)}
			/>
		{/if}
	</ReviewFrame>
{/if}

<style>
	.error {
		color: var(--danger-text);
	}
	/* One answer: a sans label over the value in the reading face. */
	.answer {
		padding: var(--space-4) 0;
		border-top: 1px solid var(--line-soft);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.6;
		color: var(--text-body);
	}
	.answer:first-child {
		border-top: 0;
	}
	/* The answer a project's search opened on. */
	.answer.opened {
		scroll-margin-top: var(--space-3);
		box-shadow: inset 3px 0 0 var(--text-muted);
		padding-left: var(--space-3);
	}
	.answer h2 {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
		margin-bottom: var(--space-2);
	}
	.kind {
		font-weight: 400;
		margin-left: var(--space-2);
		color: var(--text-muted);
	}
	.answer p + p {
		margin-top: var(--space-2);
	}
	.answer .siglum {
		margin-right: 0.35em;
	}
	.choice {
		font-weight: 600;
	}
	.reason {
		font-size: var(--type-md);
		font-style: italic;
		color: var(--text-secondary);
	}
	.none {
		font-family: var(--font-ui);
		font-size: var(--type-sm);
	}
</style>
