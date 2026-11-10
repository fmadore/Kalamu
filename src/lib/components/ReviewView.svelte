<script lang="ts">
	// The review of a text: the source beside the text. The first set-up's text (with the
	// researcher's corrections) is the text shown; where the other set-ups, or the model under a
	// correction, read differently, the words are highlighted, and pointing at them shows the
	// readings over the text, any of which can be taken as a correction. Another set-up's whole
	// text can open in a third pane beside it. A search finds words across every page of the text
	// shown, corrections included, and goes to each match (a recording plays from its line).
	// A recording's transcript is shown once, as this text: the player's pane holds the turns to
	// listen from, and the passage being heard is marked here and followed while it plays.
	import { onMount, tick, untrack } from 'svelte';
	import AudioPane from './AudioPane.svelte';
	import CompareBar from './CompareBar.svelte';
	import ComparePane from './ComparePane.svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import Icon from './Icon.svelte';
	import MarkedText from './MarkedText.svelte';
	import PageStrip from './PageStrip.svelte';
	import ReadingsPopover from './ReadingsPopover.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import ScoreFoot from './ScoreFoot.svelte';
	import SearchBar from './SearchBar.svelte';
	import SiglaKey from './SiglaKey.svelte';
	import SpeakerList from './SpeakerList.svelte';
	import VideoPane from './VideoPane.svelte';
	import { apparatus, lines, siglum, type Entry } from '#lib/apparatus.js';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { caretTop, growWithText, level, placeInText, reveal } from '#lib/caret.js';
	import { Comparison, shownText, witnessesOf, witnessName } from '#lib/compare.svelte.js';
	import { clearHeard, markHeard, placeLines } from '#lib/heard.js';
	import { messages } from '#lib/i18n/index.js';
	import { shifted } from '#lib/platform.js';
	import { pieceLabel, pieceNumber } from '#lib/pieces.js';
	import { layout, Review } from '#lib/review.svelte.js';
	import { ReviewSearch } from '#lib/reviewSearch.svelte.js';
	import { FIELDS } from '#lib/keys.js';
	import { describeScore } from '#lib/scores.js';
	import { guardDraft, leaveThen, shortcutsHeld, type ReviewFind } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		sourceId,
		find: opening,
		start
	}: {
		jobId: string;
		sourceId: string;
		/** A match chosen in a project's search, to go to. */
		find?: ReviewFind;
		/** The piece to open on, from 0: the one chosen in the list of what to check. */
		start?: number;
	} = $props();
	const m = messages();
	// svelte-ignore state_referenced_locally
	const review = new Review(jobId, sourceId, { scored: true });

	// svelte-ignore state_referenced_locally
	let index = $state(start ?? 0);
	// Stepping through the list of what to check turns to the next piece of the same source.
	$effect(() => {
		const at = start;
		if (at !== undefined) untrack(() => goTo(at));
	});
	/** The text being corrected, and the text it was opened on, to tell whether it changed. */
	let editing = $state<{ setupId: string; text: string; from: string } | null>(null);
	let showDiff = $state(true);
	let spans = $state<Record<string, DiffSpan[]>>({});
	/** The difference whose readings are open over the text. Pinned once clicked or stepped to, it
	 * stays until closed; only pointed at, it shows while the pointer is on it or its readings. */
	let selected = $state<number | null>(null);
	let pinned = $state(false);
	/** The difference last opened by a click or a step, or reached with the arrow keys, kept
	 * once its readings close, so the next step goes on from it rather than from the first. */
	let stepped = $state<number | null>(null);
	/** The text the pinned readings were opened from, where the focus goes back when they close. */
	let opener: HTMLElement | null = null;
	/** The difference under the pointer, in either text: highlighted in both. */
	let hovered = $state<number | null>(null);
	let textPane = $state<HTMLElement | null>(null);
	let comparePane = $state<HTMLElement | null>(null);
	let hoverTimer: ReturnType<typeof setTimeout> | undefined;

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
	/** The turn the player is at (from 0 in `timed`'s lines, -1 for none) and whether it plays. */
	let heard = $state({ line: -1, playing: false });

	function viewFor(setup: Setup | undefined): PieceView | undefined {
		return setup ? current.find((v) => v.setup_id === setup.id) : undefined;
	}

	const lead = $derived(viewFor(setups[0]));
	const base = $derived(shownText(lead));
	const corrected = $derived(lead?.corrected_text != null);

	/** The witnesses compared with the text shown: each other set-up, and the model itself when
	 * its text was corrected. */
	const witnesses = $derived(witnessesOf(setups, viewFor, job?.task, m));
	const comparison = new Comparison(() => witnesses);

	/** How many times the differences were worked out afresh. */
	let rounds = $state(0);
	$effect(() => {
		const text = base;
		const list = witnesses;
		spans = {};
		untrack(() => rounds++);
		for (const w of list) {
			api
				.diffTexts(text, w.text)
				.then((d) => {
					if (base === text) spans = { ...spans, [w.key]: d };
				})
				.catch(() => undefined);
		}
	});

	/** Where each of the player's turns stands in the text shown, corrections included. */
	const said = $derived((timed?.segments ?? []).map((s) => s.text));
	const placed = $derived(recording ? placeLines(base, said) : []);
	/** When the researcher last scrolled the text; following the player waits a moment after. */
	let scrolledAt = 0;
	$effect(() => {
		const pane = textPane;
		if (!pane) return;
		const scrolled = () => (scrolledAt = Date.now());
		const events = ['wheel', 'touchmove', 'keydown'] as const;
		for (const name of events) pane.addEventListener(name, scrolled, { passive: true });
		return () => events.forEach((name) => pane.removeEventListener(name, scrolled));
	});

	const entries: Entry[] = $derived(
		apparatus(
			base,
			witnesses.filter((w) => spans[w.key]).map((w) => ({ siglum: w.siglum, spans: spans[w.key] }))
		)
	);
	const reading = $derived(lines(base, entries));

	// The passage the player is at is marked in the text, drawn again with the text, and kept in
	// view while it plays, unless the researcher has just scrolled the text away.
	$effect(() => {
		const span = editing ? null : (placed[heard.line] ?? null);
		const playing = heard.playing;
		const pane = textPane;
		void [reading, showDiff, selected];
		if (!pane || !recording) return;
		let gone = false;
		void tick().then(() => {
			if (gone) return;
			const at = markHeard(pane, '.t', span);
			if (at && playing && Date.now() - scrolledAt > 4_000) reveal(pane, at);
		});
		return () => {
			gone = true;
			clearHeard();
		};
	});

	// Once a reading used is in the text and its differences are worked out again, the focus
	// goes back to that place, or to the next one if it no longer differs.
	$effect(() => {
		const want = refocus;
		if (!want || rounds <= want.round || Object.keys(spans).length < witnesses.length) return;
		const target = entries.find((x) => x.start >= want.at) ?? entries.at(-1);
		untrack(() => {
			refocus = null;
			if (target) stepped = target.id;
		});
		void tick().then(() => {
			const words = target ? wordsIn(textPane, target.id) : null;
			if (words) words.focus({ preventScroll: true });
			else if (textPane?.hasAttribute('tabindex')) textPane.focus({ preventScroll: true });
		});
	});
	/** An entry whose readings include the model's own text marks a correction. */
	const isCorrection = (e: Entry) => corrected && e.readings.some((r) => r.siglum === siglum(0));
	const open = $derived(entries.find((e) => e.id === selected) ?? null);
	/** The difference that holds the texts' one tab stop: the one open, else the last reached. */
	const stop = $derived((pinned ? selected : null) ?? stepped);

	/** Who reads what: the first set-up's model output under a correction, or another set-up. */
	const nameOf = (s: string) => witnessName(s, setups, m);

	/** Shows a difference's readings over the text. */
	function show(id: number, pin: boolean) {
		clearTimeout(hoverTimer);
		selected = id;
		pinned = pin;
		if (pin) stepped = id;
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

	/** Opens the main text for correction where the researcher is reading, the caret there and
	 * that line where it stood on screen. */
	async function correctHere(setupId: string) {
		const place = placeInText(textPane, base);
		editing = { setupId, text: base, from: base };
		if (!place) return;
		await tick();
		const pane = textPane;
		const area = pane?.querySelector<HTMLTextAreaElement>('textarea.editor');
		if (!pane || !area) return;
		area.focus({ preventScroll: true });
		area.setSelectionRange(place.start, place.end);
		const areaTop =
			area.getBoundingClientRect().top - pane.getBoundingClientRect().top + pane.scrollTop;
		pane.scrollTop = areaTop + caretTop(area, place.start) - place.below;
	}

	/** Opens a difference and pins it, bringing its words into view in both texts, whichever
	 * text it was chosen in. Chosen from the keyboard, the focus goes into its readings. */
	async function choose(id: number, from?: HTMLElement | null, keyboard = false) {
		show(id, true);
		opener = from ?? textPane;
		await tick();
		const words = wordsIn(textPane, id);
		const other = wordsIn(comparePane, id);
		const clicked = from ? wordsIn(from, id) : null;
		if (from && clicked) {
			// Chosen in one text: the other comes level with it, so both read the same place.
			const height = clicked.getBoundingClientRect().top - from.getBoundingClientRect().top;
			if (from === textPane) level(comparePane, other, height);
			else level(textPane, words, height);
		} else {
			reveal(textPane, words);
			reveal(comparePane, other);
		}
		words?.focus({ preventScroll: true });
		if (keyboard) await focusReadings();
	}

	/** Takes the focus to the first reading to choose, once the readings are placed. */
	async function focusReadings() {
		await tick();
		await new Promise((done) => requestAnimationFrame(done));
		const box = textPane?.querySelector<HTMLElement>('.readings');
		(box?.querySelector<HTMLElement>('.pick') ?? box?.querySelector<HTMLElement>('input'))?.focus({
			preventScroll: true
		});
	}

	/** Closes pinned readings, the focus back on the words they were opened from. */
	async function closeReadings() {
		const was = selected;
		const from = opener;
		hide();
		if (was === null) return;
		await tick();
		(wordsIn(from, was) ?? wordsIn(textPane, was))?.focus({ preventScroll: true });
	}

	/** A click anywhere else closes pinned readings. */
	function onPointerDown(e: PointerEvent) {
		if (!pinned || (e.target as HTMLElement).closest('.readings, [data-entry], [data-step]'))
			return;
		hide();
	}

	function stepEntry(delta: number, keyboard = false) {
		if (entries.length === 0) return;
		const at = entries.findIndex((e) => e.id === (selected ?? stepped));
		const next = at < 0 ? (delta > 0 ? 0 : entries.length - 1) : at + delta;
		const target = entries[(next + entries.length) % entries.length];
		void choose(target.id, undefined, keyboard);
	}

	/** Where the focus goes once a reading used from the readings is in the text and the
	 * differences are worked out again: the place it was, or the next one. */
	let refocus = $state<{ at: number; round: number } | null>(null);

	/** Takes a reading into the text as a correction of the first set-up; the model's text stays. */
	async function useReading(e: Entry, text: string) {
		const first = setups[0];
		if (!first || !lead) return;
		// The entry's range may end in punctuation shared by every reading and left out of them.
		const range = base.slice(e.start, e.end);
		const tail = range.startsWith(e.lemma) ? range.slice(e.lemma.length) : '';
		const next = base.slice(0, e.start) + text + tail + base.slice(e.end);
		const focused = textPane?.querySelector('.readings')?.contains(document.activeElement);
		const from = opener;
		hide();
		if (focused) {
			refocus = { at: e.start, round: rounds };
			(wordsIn(from, e.id) ?? wordsIn(textPane, e.id))?.focus({ preventScroll: true });
		}
		if (!(await save(first, next, lead.reviewed))) refocus = null;
	}

	const labelOf = (v: PieceView | undefined) => (v ? pieceLabel(v.piece, m, online) : '');

	// svelte-ignore state_referenced_locally
	const finder = new ReviewSearch(
		{
			views: () => views,
			lead: () => setups[0]?.id,
			indices: () => indices,
			index: () => index,
			recording: () => recording,
			labelOf,
			goTo,
			compare: (setupId) => comparison.show(setupId)
		},
		opening
	);

	// The page image follows the page.
	$effect(() => review.showPage(index));

	async function save(setup: Setup, text: string | null, reviewed: boolean): Promise<boolean> {
		const saved = await review.save({ setup, index, text, reviewed });
		if (saved) editing = null;
		return saved;
	}

	function toggleReviewed() {
		const first = setups[0];
		if (first && lead && lead.ai_text !== null)
			void save(first, lead.corrected_text, !lead.reviewed);
	}

	/** Turns to another piece, asking first when a correction holds changes. */
	function goTo(i: number) {
		leaveThen(() => {
			index = i;
			editing = null;
			hide();
			stepped = null;
			refocus = null;
		});
	}

	function step(delta: number) {
		const next = indices[indices.indexOf(index) + delta];
		if (next !== undefined) goTo(next);
	}

	function onKey(e: KeyboardEvent) {
		if (shortcutsHeld()) return;
		const target = e.target as HTMLElement;
		const inField = target.closest(FIELDS) !== null;
		// Escape closes the readings from anywhere but another field, their own included.
		if (e.key === 'Escape' && selected !== null && (!inField || target.closest('.readings'))) {
			if (pinned) void closeReadings();
			else hide();
			return;
		}
		if (inField) return;
		if (!e.ctrlKey && !e.metaKey && !e.altKey && e.key.toLowerCase() === 'f' && !editing) {
			e.preventDefault();
			stepEntry(e.shiftKey ? -1 : 1, true);
		}
	}

	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void review.load();
		return guardDraft({
			dirty: () => editing !== null && editing.text !== editing.from,
			discard: () => (editing = null)
		});
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
		onchange={(e) => {
			const list = e.currentTarget;
			goTo(Number(list.value));
			// Kept on the piece shown while the researcher is asked about a correction.
			list.value = String(index);
		}}
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
		<button class="btn toggle" aria-pressed={comparison.open} onclick={() => layout.toggleCompare()}
			><Icon name="compare" size={14} />{m.review.compare}</button
		>
	{/if}
	{#if lead && lead.ai_text !== null && setups[0] && !editing}
		{@const first = setups[0]}
		<button class="btn" onclick={() => correctHere(first.id)}
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
			seek={finder.seek}
			onheard={(line, playing) => (heard = { line, playing })}
		/>
	{:else}
		<AudioPane
			{jobId}
			{sourceId}
			{index}
			offsetMs={timed?.piece.kind === 'segment' ? Number(timed.piece.start_ms) : 0}
			segments={timed?.segments ?? []}
			seek={finder.seek}
			onheard={(line, playing) => (heard = { line, playing })}
		/>
	{/if}
{/snippet}

{#snippet searchBar()}
	<SearchBar
		bind:query={finder.query}
		placeholder={m.review.searchText}
		summary={finder.summary}
		matches={finder.listed}
		current={finder.at}
		onpick={finder.goToHit}
		onclose={() => (finder.searching = false)}
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
				title="{m.review.previousDifference} ({shifted('F')})"
				data-step
				onclick={() => stepEntry(-1)}><Icon name="up" size={14} /></button
			>
			<button
				class="btn sm icon"
				aria-label={m.review.nextDifference}
				title="{m.review.nextDifference} (F)"
				data-step
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
	<ScoreFoot
		{report}
		{setups}
		{ownScore}
		{lead}
		compared={witnesses.length > 0}
		correction={job?.task === 'text_correction'}
		{corrected}
	/>
{/snippet}

{#snippet compareBar()}
	<CompareBar
		compared={comparison.compared}
		{witnesses}
		{nameOf}
		onchoose={(key) => (comparison.key = key)}
		onclose={() => layout.toggleCompare()}
	/>
{/snippet}

{#snippet compareText()}
	<ComparePane
		compared={comparison.compared}
		{base}
		{spans}
		{entries}
		marked={showDiff}
		{selected}
		{hovered}
		{stop}
		onpick={(id, keyboard) => void choose(id, comparePane, keyboard)}
		onmove={(id) => (stepped = id)}
		onhover={(id) => (hovered = id)}
	/>
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
	{sourceId}
	jobName={job?.name ?? null}
	fileName={source?.file_name ?? ''}
	error={review.error}
	{pager}
	{actions}
	search={searchBar}
	bind:searching={finder.searching}
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
	third={comparison.open && comparison.compared ? compareText : undefined}
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
			<textarea
				class="editor"
				bind:value={editing.text}
				aria-label="{m.review.edit} {target.label}"
				{@attach growWithText}></textarea>
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
			find={finder.searching ? finder.query : ''}
			current={finder.here}
			onfound={(at) => {
				if (!finder.revealing) return;
				finder.revealing = false;
				reveal(textPane, at);
			}}
			{stop}
			onpick={(id, keyboard) =>
				selected === id && pinned ? hide() : void choose(id, textPane, keyboard)}
			onmove={(id) => (stepped = id)}
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
					witnessName={nameOf}
					{pane}
					onuse={(text) => useReading(e, text)}
					onclose={() => void closeReadings()}
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
			onedit={(s, v) => {
				const text = shownText(v);
				editing = { setupId: s.id, text, from: text };
			}}
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
		border-color: var(--control-line-hover);
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
	/* As tall as its text, so the pane scrolls it as it scrolls the text read; where the webview
	   cannot size it so, `growWithText` does. */
	.editor {
		width: 100%;
		min-height: 24rem;
		field-sizing: content;
		resize: none;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.65;
	}
</style>
