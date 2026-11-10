<script lang="ts">
	// The review of an analysis: the document beside its answers.
	// The first set-up's answers lead. In a comparison every value carries the sigla of the
	// set-ups that gave it, so agreement reads at a glance; pointing at a name finds it in the
	// document. Corrections are kept apart from the model's answers. A search keeps the fields
	// whose answers or reasons hold its words, marked where they stand, and marks them in a text
	// document too. A name in a list renames or removes it at once, in the corrected answers of
	// every set-up that found it, from its menu: a click, Enter, a right click or the menu key.
	// The names of an answer are one tab stop, ← and → going from one to the next; a name with
	// the focus is found in the document as one pointed at.
	import { onMount, tick } from 'svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import FieldInput from './FieldInput.svelte';
	import Icon from './Icon.svelte';
	import ItemMenu from './ItemMenu.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import SearchBar from './SearchBar.svelte';
	import SiglaKey from './SiglaKey.svelte';
	import { collate, editItem, holdsItem, holdsWords, type Witness } from '#lib/analysis.js';
	import { siglum } from '#lib/apparatus.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { type Draft, fromDraft, toDraft } from '#lib/fields.js';
	import { clearFound, markFound } from '#lib/found.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { Review } from '#lib/review.svelte.js';
	import { find, searchable } from '#lib/search.js';
	import { rovingStop, rovingTarget } from '#lib/keys.js';
	import { guardDraft, toast, type ReviewFind } from '#lib/state.svelte.js';

	let {
		jobId,
		sourceId,
		find: opening
	}: { jobId: string; sourceId: string; find?: ReviewFind } = $props();
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
	const pages = $derived(source?.kind === 'pdf' ? (source.page_count ?? 1) : 1);
	const viewOf = (s: Setup | undefined) => review.views.find((v) => v.setup_id === s?.id);
	const answersOf = (v: PieceView | undefined): FieldAnswer[] =>
		v?.corrected_fields ?? v?.ai_fields ?? [];
	const lead = $derived(viewOf(setups[0]));
	const witnesses: Witness[] = $derived(
		setups.map((s, i) => ({ siglum: siglum(i), answers: answersOf(viewOf(s)) }))
	);
	const comparing = $derived(setups.length > 1);

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

	// A PDF or image is shown page by page; a text document as text.
	$effect(() => review.showPage(page));

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

	/** The name whose menu is open, and where. */
	let itemMenu = $state<{ key: string; value: string; x: number; y: number } | null>(null);

	/** Opens a name's menu: at the pointer for a right click, under the name otherwise. */
	function openItemMenu(e: MouseEvent, f: Field) {
		if (f.kind.type !== 'list' || editing) return;
		const item = (e.target as HTMLElement).closest<HTMLElement>('[data-item]');
		if (!item?.dataset.item) return;
		e.preventDefault();
		const pointer = e.type === 'contextmenu' && (e.clientX !== 0 || e.clientY !== 0);
		const at = item.getBoundingClientRect();
		itemMenu = {
			key: f.key,
			value: item.dataset.item,
			x: pointer ? e.clientX : at.left,
			y: pointer ? e.clientY : at.bottom + 4
		};
	}

	/** The name of each answer that holds its tab stop, once one was reached. */
	let itemStops = $state<Record<string, string>>({});

	function focusItem(key: string, value: string) {
		answersPane
			?.querySelector<HTMLElement>(
				`[data-field="${CSS.escape(key)}"] [data-item="${CSS.escape(value)}"]`
			)
			?.focus();
	}

	function roveItems(e: KeyboardEvent, key: string, values: string[], at: string) {
		if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
		const to = rovingTarget(values, at, e.key);
		if (to === null) return;
		e.preventDefault();
		itemStops[key] = to;
		focusItem(key, to);
	}

	/**
	 * Renames a name, or removes it when `to` is null, in the answers of every set-up that found
	 * it, each kept as reviewed or not as it was.
	 */
	async function changeItem(key: string, from: string, to: string | null) {
		hovered = null;
		const listed = collate(key, witnesses).map((r) => r.value);
		/** Each set-up's corrected answers and review as they stood, to put back. */
		const changed: { setup: Setup; fields: FieldAnswer[] | null; reviewed: boolean }[] = [];
		let failed = false;
		for (const s of setups) {
			const v = viewOf(s);
			if (!v || v.status === 'failed') continue;
			const before = answersOf(v);
			if (!holdsItem(before, key, from)) continue;
			const was = { setup: s, fields: $state.snapshot(v.corrected_fields), reviewed: v.reviewed };
			const saved = await review.save({
				setup: s,
				index: 0,
				fields: editItem(before, key, from, to),
				reviewed: v.reviewed
			});
			if (!saved) {
				failed = true;
				break;
			}
			changed.push(was);
		}
		// A name removed can be put back from the toast, as it was in each set-up's answers.
		if (to === null && changed.length > 0) {
			toast(m.review.itemRemoved(from), {
				action: { label: m.common.undo, run: () => void putBack(key, from, changed) }
			});
		}
		if (failed) return;
		// The focus, lost with the name it was on, goes to the name renamed or to the next one.
		await tick();
		if (document.activeElement && document.activeElement !== document.body) return;
		const now = collate(key, witnesses).map((r) => r.value);
		const next =
			to !== null && now.includes(to)
				? to
				: now[Math.min(Math.max(listed.indexOf(from), 0), now.length - 1)];
		if (next === undefined) return;
		itemStops[key] = next;
		focusItem(key, next);
	}

	/** Puts each set-up's corrected answers and review back as they were, and the focus, lost
	 * with the toast's button, on the name put back. */
	async function putBack(
		key: string,
		name: string,
		changed: { setup: Setup; fields: FieldAnswer[] | null; reviewed: boolean }[]
	) {
		for (const c of changed) {
			if (!(await review.save({ index: 0, ...c }))) return;
		}
		await tick();
		if (document.activeElement && document.activeElement !== document.body) return;
		if (!collate(key, witnesses).some((r) => r.value === name)) return;
		itemStops[key] = name;
		focusItem(key, name);
	}

	function toggleReviewed() {
		const first = setups[0];
		if (first && lead) void save(first, lead.corrected_fields, !lead.reviewed);
	}

	const kindLabel = (f: Field) =>
		f.kind.type === 'entries' ? m.fields.kinds.list : m.fields.kinds[f.kind.type];
	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void review.load();
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
	<span class="mono"
		>{lead?.model_version ?? setups[0]?.model ?? ''}{lead?.cost_usd
			? ` · ${formatUsd(lead.cost_usd)}`
			: ''}</span
	>
{/snippet}

{#if itemMenu}
	{@const at = itemMenu}
	<ItemMenu
		value={at.value}
		x={at.x}
		y={at.y}
		onrename={(to) => void changeItem(at.key, at.value, to)}
		onremove={() => void changeItem(at.key, at.value, null)}
		onclose={() => (itemMenu = null)}
	/>
{/if}

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
	text={review.text}
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
						<FieldInput field={f} bind:value={draft[f.key]} />
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
				{#if readings.length === 0}
					<p class="muted none">{m.review.noneFound}</p>
				{:else if f.kind.type === 'list' || f.kind.type === 'choices'}
					{@const named = f.kind.type === 'list'}
					{@const values = readings.map((r) => r.value)}
					{@const stop = rovingStop(values, itemStops[f.key])}
					<p class="items" oncontextmenu={(e) => openItemMenu(e, f)}>
						{#each readings as r, k (r.value)}{#if k > 0}<span class="sep">{'; '}</span>{/if}<button
								type="button"
								class="item"
								class:named
								class:partial={comparing && r.sigla.length < setups.length}
								data-item={r.value}
								tabindex={stop === r.value ? 0 : -1}
								aria-haspopup={named ? 'menu' : undefined}
								aria-describedby={named ? 'item-keys' : 'item-locates'}
								onclick={(e) => openItemMenu(e, f)}
								onkeydown={(e) => roveItems(e, f.key, values, r.value)}
								onfocus={() => {
									hovered = r.value;
									itemStops[f.key] = r.value;
								}}
								onblur={() => (hovered = null)}
								onpointerenter={() => (hovered = r.value)}
								onpointerleave={() => (hovered = null)}
								>{r.value}{#if comparing && r.sigla.length < setups.length}<sup class="found"
										>{r.sigla.join(' ')}</sup
									>{/if}</button
							>{/each}
					</p>
				{:else}
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
		<span class="sr-only" id="item-locates">{m.review.itemLocates}</span>
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
	/* A name reads as the text around it; pointed at or focused, it takes the highlighter, as
	   where it is found in the document. */
	.items .item {
		display: inline;
		padding: 0;
		border: 0;
		border-radius: 2px;
		background: none;
		color: inherit;
		font: inherit;
		line-height: inherit;
		text-align: inherit;
		cursor: default;
		transition: background-color 120ms ease-out;
	}
	.items .item.named {
		cursor: pointer;
	}
	.items .item:hover,
	.items .item:focus-visible {
		background: var(--marker);
		color: var(--on-marker);
		box-shadow: 0 0 0 2px var(--marker);
	}
	.items .item:focus-visible {
		outline-offset: 3px;
	}
	/* The sigla of the set-ups that found a name, when not all of them did. */
	.found {
		font-family: var(--font-mono);
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: uppercase;
		color: var(--text-muted);
		margin-left: 0.15em;
	}
	.sep {
		color: var(--text-faint);
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
