<script lang="ts">
	// The review of a text: the source beside the text. The first set-up's text (with the
	// researcher's corrections) is the text shown; where the other set-ups, or the model under a
	// correction, read differently, the words are highlighted, and pointing at them shows the
	// readings over the text, any of which can be taken as a correction. Another set-up's whole
	// text can open in a third pane beside it. A search finds words across every page of the text
	// shown, corrections included, and goes to each match (a recording plays from its line).
	import { onMount, tick } from 'svelte';
	import AudioPane from './AudioPane.svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import Icon from './Icon.svelte';
	import MarkedText from './MarkedText.svelte';
	import PageStrip from './PageStrip.svelte';
	import ReadingsPopover from './ReadingsPopover.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import SearchBar, { type ListedMatch } from './SearchBar.svelte';
	import SiglaKey from './SiglaKey.svelte';
	import SpeakerList from './SpeakerList.svelte';
	import VideoPane from './VideoPane.svelte';
	import { apparatus, lines, siglum, witnessParts, type Entry } from '#lib/apparatus.js';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { formatPercent, formatUsd, messages } from '#lib/i18n/index.js';
	import { clock, pieceLabel, pieceNumber } from '#lib/pieces.js';
	import { layout, Review } from '#lib/review.svelte.js';
	import { describeScore } from '#lib/scores.js';
	import { context, search, searchable, type Searched } from '#lib/search.js';
	import { api } from '#lib/tauri.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();
	// svelte-ignore state_referenced_locally
	const review = new Review(jobId, sourceId, { scored: true });

	let index = $state(0);
	let editing = $state<{ setupId: string; text: string } | null>(null);
	let showDiff = $state(true);
	let spans = $state<Record<string, DiffSpan[]>>({});
	/** The difference whose readings are open over the text. Pinned once clicked or stepped to, it
	 * stays until closed; only pointed at, it shows while the pointer is on it or its readings. */
	let selected = $state<number | null>(null);
	let pinned = $state(false);
	/** The difference under the pointer, in either text: highlighted in both. */
	let hovered = $state<number | null>(null);
	let textPane = $state<HTMLElement | null>(null);
	let comparePane = $state<HTMLElement | null>(null);
	let hoverTimer: ReturnType<typeof setTimeout> | undefined;
	/** The witness shown in the compare pane, by its key. */
	let compareKey = $state<string | null>(null);

	const job = $derived(review.job);
	const source = $derived(review.source);
	const views = $derived(review.views);
	const report = $derived(review.report);
	const setups = $derived(review.setups);
	const indices = $derived([...new Set(views.map((v) => v.index))].sort((a, b) => a - b));
	const current = $derived(views.filter((v) => v.index === index));
	/** The set-ups whose answer for this page stopped at the model's output limit. */
	const cutOff = $derived(
		setups.flatMap((s, i) => (current.find((v) => v.setup_id === s.id)?.truncated ? [i] : []))
	);
	/** A single model's error rate on this document, once pages are reviewed. */
	const ownScore = $derived(
		job && report && setups.length === 1 && report.scores[0]
			? describeScore(report.scores[0], job.task, m)
			: null
	);
	/** A YouTube video is transcribed like a recording but watched on YouTube. */
	const online = $derived(source?.kind === 'youtube');
	const recording = $derived(source?.kind === 'audio' || source?.kind === 'video' || online);
	const timed = $derived(current.find((v) => v.segments.length > 0) ?? current[0]);

	function viewFor(setup: Setup | undefined): PieceView | undefined {
		return setup ? current.find((v) => v.setup_id === setup.id) : undefined;
	}

	function shownText(v: PieceView | undefined): string {
		return v?.corrected_text ?? v?.ai_text ?? '';
	}

	const lead = $derived(viewFor(setups[0]));
	const base = $derived(shownText(lead));
	const corrected = $derived(lead?.corrected_text != null);

	/** The witnesses compared with the text shown: each other set-up, and the model itself when
	 * its text was corrected. */
	const witnesses = $derived.by(() => {
		const out: { key: string; siglum: string; text: string }[] = [];
		if (corrected && lead?.ai_text != null) {
			out.push({ key: 'model', siglum: siglum(0), text: lead.ai_text });
		}
		// A correction is compared with the text it corrected, so every change shows.
		if (job?.task === 'text_correction' && lead?.source_text != null) {
			out.push({ key: 'source', siglum: m.review.originalSiglum, text: lead.source_text });
		}
		setups.slice(1).forEach((s, i) => {
			const v = viewFor(s);
			if (v?.ai_text != null) out.push({ key: s.id, siglum: siglum(i + 1), text: shownText(v) });
		});
		return out;
	});

	$effect(() => {
		const text = base;
		const list = witnesses;
		spans = {};
		for (const w of list) {
			api
				.diffTexts(text, w.text)
				.then((d) => {
					if (base === text) spans = { ...spans, [w.key]: d };
				})
				.catch(() => undefined);
		}
	});

	const entries: Entry[] = $derived(
		apparatus(
			base,
			witnesses.filter((w) => spans[w.key]).map((w) => ({ siglum: w.siglum, spans: spans[w.key] }))
		)
	);
	const reading = $derived(lines(base, entries));
	/** An entry whose readings include the model's own text marks a correction. */
	const isCorrection = (e: Entry) => corrected && e.readings.some((r) => r.siglum === siglum(0));
	const open = $derived(entries.find((e) => e.id === selected) ?? null);

	const comparing = $derived(layout.compare && witnesses.length > 0);
	const compared = $derived(
		witnesses.find((w) => w.key === compareKey) ??
			witnesses.find((w) => w.key !== 'model') ??
			witnesses[0]
	);
	const compareParts = $derived(
		compared && spans[compared.key] ? witnessParts(base, spans[compared.key], entries) : null
	);

	/** Who reads what: the first set-up's model output under a correction, or another set-up. */
	function witnessName(s: string): string {
		if (s === siglum(0)) return m.review.modelText(setups[0]?.label ?? '');
		if (s === m.review.originalSiglum) return m.review.original;
		return setups[s.charCodeAt(0) - 97]?.label ?? s;
	}

	/** Shows a difference's readings over the text. */
	function show(id: number, pin: boolean) {
		clearTimeout(hoverTimer);
		selected = id;
		pinned = pin;
	}

	function hide() {
		clearTimeout(hoverTimer);
		selected = null;
		pinned = false;
	}

	/** Pointing at highlighted words shows their readings after a moment, unless some are pinned. */
	function pointAt(e: PointerEvent, id: number) {
		hovered = id;
		if (e.pointerType !== 'mouse' || pinned) return;
		clearTimeout(hoverTimer);
		hoverTimer = setTimeout(() => show(id, false), selected === null ? 300 : 60);
	}

	/** Leaving the words or their readings closes readings that are not pinned. */
	function pointAway() {
		hovered = null;
		if (pinned) return;
		clearTimeout(hoverTimer);
		hoverTimer = setTimeout(() => {
			if (!pinned) hide();
		}, 250);
	}

	/** The words of a difference in a pane. */
	const wordsIn = (pane: HTMLElement | null, id: number) =>
		pane?.querySelector<HTMLElement>(`[data-entry="${id}"]`) ?? null;

	/** Scrolls a pane so that an element in it is in view, a third of the way down. */
	function reveal(pane: HTMLElement | null, el: Element | Range | null) {
		if (!pane || !el) return;
		const box = pane.getBoundingClientRect();
		const r = el.getBoundingClientRect();
		if (r.top >= box.top + 24 && r.bottom <= box.bottom - 24) return;
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		pane.scrollTo({
			top: pane.scrollTop + r.top - box.top - box.height / 3,
			behavior: still ? 'auto' : 'smooth'
		});
	}

	/** Opens a difference and pins it, bringing its words into view in both texts. */
	async function choose(id: number) {
		show(id, true);
		await tick();
		const words = wordsIn(textPane, id);
		reveal(textPane, words);
		reveal(comparePane, wordsIn(comparePane, id));
		words?.focus({ preventScroll: true });
	}

	/** A click anywhere else closes pinned readings. */
	function onPointerDown(e: PointerEvent) {
		if (!pinned || (e.target as HTMLElement).closest('.readings, [data-entry]')) return;
		hide();
	}

	function stepEntry(delta: number) {
		if (entries.length === 0) return;
		const at = entries.findIndex((e) => e.id === selected);
		const next = at < 0 ? (delta > 0 ? 0 : entries.length - 1) : at + delta;
		const target = entries[(next + entries.length) % entries.length];
		void choose(target.id);
	}

	/** Takes a reading into the text as a correction of the first set-up; the model's text stays. */
	async function useReading(e: Entry, text: string) {
		const first = setups[0];
		if (!first || !lead) return;
		// The entry's range may end in punctuation shared by every reading and left out of them.
		const range = base.slice(e.start, e.end);
		const tail = range.startsWith(e.lemma) ? range.slice(e.lemma.length) : '';
		const next = base.slice(0, e.start) + text + tail + base.slice(e.end);
		hide();
		await save(first, next, lead.reviewed);
	}

	const labelOf = (v: PieceView | undefined) => (v ? pieceLabel(v.piece, m, online) : '');

	let searching = $state(false);
	let query = $state('');
	/** The match gone to, among all of them; -1 before any. */
	let hitAt = $state(-1);
	/** Where a recording should play from, after going to a match. */
	let seek = $state<{ ms: number; id: number } | null>(null);
	/** Set when going to a match, until its words are in view. */
	let revealHit = false;

	// Closing the search forgets the words; new words start from the first match.
	$effect(() => {
		if (!searching) query = '';
	});
	$effect(() => {
		void query;
		hitAt = -1;
	});

	/** Each piece as it is searched: the first set-up's text as shown, corrections included. */
	const searched: Searched[] = $derived(
		indices.map((i) => {
			const v = views.find((x) => x.index === i && x.setup_id === setups[0]?.id);
			return {
				index: i,
				text: shownText(v),
				startMs: v?.piece.kind === 'segment' ? Number(v.piece.start_ms) : 0,
				segments: v?.segments ?? []
			};
		})
	);
	const hits = $derived(search(searched, query));
	const listed: ListedMatch[] = $derived(
		hits.map((h) => {
			const piece = searched.find((p) => p.index === h.index);
			return {
				group: labelOf(views.find((v) => v.index === h.index)),
				time: recording && h.lineMs !== null ? clock((piece?.startMs ?? 0) + h.lineMs) : null,
				...context(piece?.text ?? '', h)
			};
		})
	);
	const summary = $derived(
		!searchable(query)
			? ''
			: hits.length === 0
				? m.review.notFound
				: hitAt >= 0
					? m.review.foundAt(hitAt + 1, hits.length)
					: m.review.found(hits.length)
	);
	/** Which match on the page shown is the one gone to. */
	const foundHere = $derived(hits[hitAt]?.index === index ? hits[hitAt].nth : -1);

	function goToHit(i: number) {
		const h = hits[i];
		if (!h) return;
		hitAt = i;
		revealHit = true;
		if (h.index !== index) goTo(h.index);
		if (recording && h.lineMs !== null) seek = { ms: h.lineMs, id: (seek?.id ?? 0) + 1 };
	}

	// The page image follows the page.
	$effect(() => review.showPage(index));

	async function save(setup: Setup, text: string | null, reviewed: boolean) {
		if (await review.save({ setup, index, text, reviewed })) editing = null;
	}

	function toggleReviewed() {
		const first = setups[0];
		if (first && lead && lead.ai_text !== null)
			void save(first, lead.corrected_text, !lead.reviewed);
	}

	function goTo(i: number) {
		index = i;
		editing = null;
		hide();
	}

	function step(delta: number) {
		const next = indices[indices.indexOf(index) + delta];
		if (next !== undefined) goTo(next);
	}

	function onKey(e: KeyboardEvent) {
		if ((e.target as HTMLElement).closest('input, textarea, select')) return;
		if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'f' && !editing) {
			e.preventDefault();
			stepEntry(e.shiftKey ? -1 : 1);
		}
		if (e.key === 'Escape' && selected !== null) {
			const was = selected;
			const focusBack = pinned;
			hide();
			if (focusBack) wordsIn(textPane, was)?.focus();
		}
	}

	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void review.load();
	});
</script>

<svelte:window onkeydown={onKey} onpointerdown={onPointerDown} />

{#snippet pager()}
	<button
		class="btn sm icon ghost"
		aria-label={m.review.previous}
		title={m.review.previous}
		disabled={indices.indexOf(index) <= 0}
		onclick={() => step(-1)}><Icon name="previous" /></button
	>
	<select
		class="pages"
		aria-label={m.review.title}
		value={index}
		onchange={(e) => goTo(Number(e.currentTarget.value))}
	>
		{#each indices as i (i)}
			{@const v = views.find((x) => x.index === i)}
			<option value={i}
				>{views.some((x) => x.index === i && x.reviewed)
					? m.review.pieceReviewed(labelOf(v))
					: labelOf(v)}</option
			>
		{/each}
	</select>
	<button
		class="btn sm icon ghost"
		aria-label={m.review.next}
		title={m.review.next}
		disabled={indices.indexOf(index) >= indices.length - 1}
		onclick={() => step(1)}><Icon name="chevron" /></button
	>
{/snippet}

{#snippet actions()}
	{#if witnesses.length > 0}
		<button class="btn toggle" aria-pressed={comparing} onclick={() => layout.toggleCompare()}
			><Icon name="compare" size={14} />{m.review.compare}</button
		>
	{/if}
	{#if lead && lead.ai_text !== null && setups[0] && !editing}
		{@const first = setups[0]}
		<button class="btn" onclick={() => (editing = { setupId: first.id, text: base })}
			><Icon name="edit" size={14} />{m.review.edit}</button
		>
	{/if}
{/snippet}

{#snippet player()}
	{#if online && source}
		<VideoPane
			url={source.path}
			title={source.title ?? null}
			segments={timed?.segments ?? []}
			{seek}
		/>
	{:else}
		<AudioPane
			{jobId}
			{sourceId}
			{index}
			offsetMs={timed?.piece.kind === 'segment' ? Number(timed.piece.start_ms) : 0}
			segments={timed?.segments ?? []}
			{seek}
		/>
	{/if}
{/snippet}

{#snippet searchBar()}
	<SearchBar
		bind:query
		placeholder={m.review.searchText}
		{summary}
		matches={listed}
		current={hitAt}
		onpick={goToHit}
		onclose={() => (searching = false)}
	/>
{/snippet}

{#snippet tools()}
	{#if witnesses.length > 0 && !editing}
		{#if entries.length > 0}
			<span class="count"
				><span class="swatch" aria-hidden="true"></span>{m.review.differences(entries.length)}</span
			>
			<button
				class="btn sm icon"
				aria-label={m.review.previousDifference}
				title="{m.review.previousDifference} (Shift F)"
				onclick={() => stepEntry(-1)}><Icon name="up" size={14} /></button
			>
			<button
				class="btn sm icon"
				aria-label={m.review.nextDifference}
				title="{m.review.nextDifference} (F)"
				onclick={() => stepEntry(1)}><Icon name="down" size={14} /></button
			>
		{:else if Object.keys(spans).length === witnesses.length}
			<span class="count">{m.review.noDifferences}</span>
		{/if}
		<label class="switch"
			><input type="checkbox" bind:checked={showDiff} /> {m.review.showDiff}</label
		>
	{/if}
{/snippet}

{#snippet foot()}
	<span class="mono">
		{#if report && setups.length > 1}
			{m.review.cer}:
			{#each report.scores as s, i (s.setup_id)}
				<span
					class="score"
					title={s.wer !== null ? `${m.review.wer}: ${formatPercent(s.wer)}` : undefined}
					>{siglum(i).toUpperCase()}
					{s.cer !== null ? formatPercent(s.cer) : '–'}</span
				>
			{/each}
			{#if report.scores.some((s) => s.cer !== null)}<span class="hint"
					>· {m.review.scoredPages(report.scores[0]?.scored_pages ?? 0)}</span
				>{:else}<span class="hint">· {m.review.noReference}</span>{/if}
		{:else if ownScore}
			<span title={m.scores.note}>{ownScore}</span>
		{:else}
			{lead?.model_version ?? setups[0]?.model ?? ''}{lead?.cost_usd
				? ` · ${formatUsd(lead.cost_usd)}`
				: ''}
		{/if}
	</span>
	{#if witnesses.length > 0 || corrected}
		<span class="legend">
			{#if witnesses.length > 0}<span
					><i class="key marker" aria-hidden="true"></i>{job?.task === 'text_correction'
						? m.review.legendChanged
						: m.review.legendDiffer}</span
				>{/if}
			<span><i class="key hand" aria-hidden="true"></i>{m.review.legendCorrected}</span>
		</span>
	{/if}
{/snippet}

{#snippet compareBar()}
	{#if compared}
		<span class="compared">
			<i class="siglum">{compared.siglum}</i>
			{#if witnesses.length > 1}
				<select
					class="mono"
					aria-label={m.review.compareWith}
					value={compared.key}
					onchange={(e) => (compareKey = e.currentTarget.value)}
				>
					{#each witnesses as w (w.key)}
						<option value={w.key}>{witnessName(w.siglum)}</option>
					{/each}
				</select>
			{:else}
				<span class="mono">{witnessName(compared.siglum)}</span>
			{/if}
		</span>
	{/if}
	<button
		class="btn sm icon ghost close-compare"
		aria-label={m.review.closeCompare}
		title={m.review.closeCompare}
		onclick={() => layout.toggleCompare()}><Icon name="close" size={14} /></button
	>
{/snippet}

{#snippet compareText()}
	{#if compareParts}
		<MarkedText
			parts={compareParts}
			{entries}
			marked={showDiff}
			{selected}
			{hovered}
			emptyLabel={m.review.leftOutHere}
			onpick={choose}
			onpoint={(_, id) => (hovered = id)}
			onaway={() => (hovered = null)}
		/>
	{:else}
		<p class="muted">{m.common.loading}</p>
	{/if}
{/snippet}

{#snippet strip()}
	{#if indices.length > 1}
		<PageStrip
			{jobId}
			{sourceId}
			paged={review.paged}
			tally
			current={index}
			items={indices.map((i, place) => {
				const v = views.find((x) => x.index === i);
				return {
					index: i,
					label: labelOf(v),
					number: v ? pieceNumber(v.piece, place, m) : String(place + 1),
					done: views.some((x) => x.index === i && x.reviewed)
				};
			})}
			onselect={goTo}
		/>
	{/if}
{/snippet}

<ReviewFrame
	{jobId}
	jobName={job?.name ?? null}
	fileName={source?.file_name ?? ''}
	error={review.error}
	{pager}
	{actions}
	search={searchBar}
	bind:searching
	reviewed={lead && lead.ai_text !== null && !editing ? lead.reviewed : null}
	onreviewed={toggleReviewed}
	onstep={step}
	source={recording ? 'player' : source?.kind === 'text' ? 'text' : 'scan'}
	piece={labelOf(lead ?? current[0])}
	preview={review.preview}
	text={lead?.source_text ?? review.text}
	{player}
	witness={{
		label: setups[0]?.label ?? '',
		siglum: setups.length > 1 ? siglum(0) : null,
		corrected
	}}
	label={m.review.readingText}
	{tools}
	{foot}
	bind:textPane
	third={comparing && compared ? compareText : undefined}
	thirdLabel={m.review.compareWith}
	thirdBar={compareBar}
	bind:thirdPane={comparePane}
	{strip}
>
	{#if editing && editedSetup}
		{@const target = editedSetup}
		{@const kept = viewFor(target)}
		<CorrectionEditor
			label={target.label}
			siglum={setups.length > 1 ? siglum(setups.indexOf(target)) : null}
			onsave={() => editing && save(target, editing.text, true)}
			ondiscard={() => (editing = null)}
			onrevert={kept?.corrected_text != null
				? () => save(target, null, kept?.reviewed ?? false)
				: undefined}
		>
			<textarea class="editor" bind:value={editing.text} aria-label="{m.review.edit} {target.label}"
			></textarea>
		</CorrectionEditor>
	{:else if !lead || (lead.ai_text === null && lead.status !== 'failed')}
		<p class="muted">{m.review.notDone}</p>
	{:else if lead.status === 'failed'}
		<p class="error">{lead.error}</p>
	{:else}
		{#if cutOff.length > 0}
			<p class="cut-off" role="note">
				{#if setups.length > 1}{#each cutOff as i (i)}<i class="siglum">{siglum(i)}</i>{/each}{/if}
				{m.review.truncated}
			</p>
		{/if}
		<MarkedText
			lines={reading}
			{entries}
			marked={showDiff}
			{selected}
			{hovered}
			corrected={(id) => {
				const e = entries.find((x) => x.id === id);
				return e !== undefined && isCorrection(e);
			}}
			emptyLabel={m.review.addedHere}
			controls="readings"
			find={searching ? query : ''}
			current={foundHere}
			onfound={(at) => {
				if (!revealHit) return;
				revealHit = false;
				reveal(textPane, at);
			}}
			onpick={(id) => (selected === id && pinned ? hide() : show(id, true))}
			onpoint={pointAt}
			onaway={pointAway}
		/>
		{#if open && textPane}
			{@const e = open}
			{@const pane = textPane}
			{#key e.id}
				<ReadingsPopover
					id="readings"
					entry={e}
					place={{ n: entries.findIndex((x) => x.id === e.id) + 1, total: entries.length }}
					heading={isCorrection(e)
						? m.review.corrected_here
						: job?.task === 'text_correction'
							? m.review.changedHere
							: m.review.disagree}
					inUse={corrected ? m.review.yourCorrections : (setups[0]?.label ?? '')}
					{witnessName}
					{pane}
					onuse={(text) => useReading(e, text)}
					onclose={hide}
					onenter={() => clearTimeout(hoverTimer)}
					onleave={pointAway}
					onpin={() => (pinned = true)}
				/>
			{/key}
		{/if}
	{/if}

	{#if !editing && recording}
		<SpeakerList
			{jobId}
			{sourceId}
			{setups}
			views={current}
			manyPieces={indices.length > 1}
			onsaved={() => review.refresh()}
		/>
	{/if}

	{#if !editing && setups.length > 1}
		<SiglaKey
			{setups}
			viewOf={viewFor}
			answered={(v) => v.ai_text !== null}
			corrected={(v) => v.corrected_text != null}
			onedit={(s, v) => (editing = { setupId: s.id, text: shownText(v) })}
			onreviewed={(s, v, on) => save(s, v.corrected_text, on)}
		/>
	{/if}
</ReviewFrame>

<style>
	.pages {
		min-height: var(--control-sm);
		max-width: 14rem;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
	}
	/* A view that is on reads as pressed in, not as an action. */
	.toggle[aria-pressed='true'] {
		background: var(--wash);
		border-color: var(--line-strong);
	}
	.count {
		white-space: nowrap;
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		margin-right: var(--space-1);
		color: var(--text-secondary);
	}
	.swatch {
		width: 0.75rem;
		height: 0.75rem;
		border-radius: 2px;
		background: var(--marker);
	}
	.switch {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		margin-left: var(--space-2);
		white-space: nowrap;
		color: var(--text-secondary);
	}
	.error {
		color: var(--danger-text);
	}
	/* The model stopped at its length limit: the end of the page may be missing. */
	.cut-off {
		display: flex;
		align-items: baseline;
		gap: var(--space-2);
		margin: 0 0 var(--space-4);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--warn-border);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
		font-size: var(--type-sm);
	}
	.editor {
		width: 100%;
		min-height: 24rem;
		max-width: 44rem;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.65;
	}
	.compared {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.compared select {
		min-height: var(--control-sm);
		max-width: 16rem;
		font-size: var(--type-xs);
	}
	.close-compare {
		margin-left: auto;
	}
	.score {
		margin-left: var(--space-2);
		color: var(--text-bright);
	}
	.hint {
		font-family: var(--font-ui);
		margin-left: var(--space-1);
	}
	.legend {
		display: inline-flex;
		gap: var(--space-4);
	}
	.legend > span {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.key {
		display: inline-block;
		width: 0.875rem;
	}
	.key.marker {
		height: 0.625rem;
		border-radius: 2px;
		background: var(--marker);
	}
	.key.hand {
		height: 2px;
		background: var(--hand);
	}
</style>
