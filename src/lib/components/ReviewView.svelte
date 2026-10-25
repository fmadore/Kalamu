<script lang="ts">
	// The review: the source on its lightbox beside the text. The first set-up's text (with the
	// researcher's corrections) is the text shown; where the other set-ups, or the model under a
	// correction, read differently, the words are highlighted, and choosing one opens the readings
	// beside it, any of which can be taken as a correction.
	import { onMount, tick } from 'svelte';
	import AudioPane from './AudioPane.svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
	import SpeakerList from './SpeakerList.svelte';
	import { apparatus, lines, numbered, siglum, type Entry } from '#lib/apparatus.js';
	import type { ComparisonReport } from '#lib/bindings/ComparisonReport.js';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { describeError } from '#lib/errors.js';
	import { formatPercent, formatUsd, messages } from '#lib/i18n/index.js';
	import { api } from '#lib/tauri.js';
	import { go, toast } from '#lib/state.svelte.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let views = $state<PieceView[]>([]);
	let index = $state(0);
	let preview = $state<PagePreview | null>(null);
	let zoom = $state(1);
	let editing = $state<{ setupId: string; text: string } | null>(null);
	let showDiff = $state(true);
	let spans = $state<Record<string, DiffSpan[]>>({});
	let report = $state<ComparisonReport | null>(null);
	let error = $state<string | null>(null);
	/** The difference whose readings are open beside the text. */
	let selected = $state<number | null>(null);
	let ownReading = $state('');
	let textPane = $state<HTMLElement | null>(null);

	const source = $derived(job?.sources.find((s) => s.id === sourceId));
	const indices = $derived([...new Set(views.map((v) => v.index))].sort((a, b) => a - b));
	const current = $derived(views.filter((v) => v.index === index));
	const setups = $derived(job?.setups ?? []);
	const recording = $derived(source?.kind === 'audio' || source?.kind === 'video');
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

	/** Who reads what: the first set-up's model output under a correction, or another set-up. */
	function witnessName(s: string): string {
		if (s === siglum(0)) return m.review.modelText(setups[0]?.label ?? '');
		return setups[s.charCodeAt(0) - 97]?.label ?? s;
	}

	/** Opens a difference, scrolling its words into view. */
	async function choose(id: number | null) {
		selected = id;
		ownReading = '';
		if (id === null) return;
		await tick();
		const lemma = textPane?.querySelector<HTMLElement>(`.lemma[data-entry="${id}"]`);
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		lemma?.scrollIntoView({ block: 'nearest', behavior: still ? 'auto' : 'smooth' });
		lemma?.focus({ preventScroll: true });
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
		selected = null;
		await save(first, next, lead.reviewed);
	}

	function pieceLabel(v: PieceView | undefined): string {
		if (!v) return '';
		if (v.piece.kind === 'page') {
			const half = v.piece.half ? ` (${v.piece.half})` : '';
			return m.review.page(`${v.piece.number}${half}`);
		}
		if (v.piece.kind === 'whole') return m.review.document;
		const s = Math.floor(v.piece.start_ms / 1000);
		const clock = [s / 3600, (s / 60) % 60, s % 60]
			.map((n) => String(Math.floor(n)).padStart(2, '0'))
			.join(':');
		return m.review.segment(clock);
	}

	/** The short name of a piece in the strip of pages: its page number, or its place. */
	function pieceNumber(i: number): string {
		const v = views.find((x) => x.index === i);
		if (v?.piece.kind === 'page') return `${v.piece.number}${v.piece.half ? v.piece.half[0] : ''}`;
		return String(indices.indexOf(i) + 1);
	}

	async function load() {
		try {
			job = await api.getJob(jobId);
			views = await api.sourcePieces(jobId, sourceId);
			if (setups.length > 1) report = await api.compareSource(jobId, sourceId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	// The page image follows the page.
	$effect(() => {
		const i = index;
		preview = null;
		if (source?.kind === 'pdf' || source?.kind === 'image') {
			api
				.piecePreview(jobId, sourceId, i)
				.then((p) => {
					if (index === i) preview = p;
				})
				.catch(() => (preview = { mime: 'none', data: '' }));
		}
	});

	async function save(setup: Setup, text: string | null, reviewed: boolean) {
		error = null;
		try {
			await api.saveCorrection({
				job_id: jobId,
				setup_id: setup.id,
				source_id: sourceId,
				index,
				text,
				reviewed,
				updated_at: 0
			});
			editing = null;
			views = await api.sourcePieces(jobId, sourceId);
			if (setups.length > 1) report = await api.compareSource(jobId, sourceId);
			toast(m.review.saved);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	function toggleReviewed() {
		const first = setups[0];
		if (first && lead && lead.ai_text !== null)
			void save(first, lead.corrected_text, !lead.reviewed);
	}

	function goTo(i: number) {
		index = i;
		editing = null;
		selected = null;
	}

	function step(delta: number) {
		const next = indices[indices.indexOf(index) + delta];
		if (next !== undefined) goTo(next);
	}

	function onKey(e: KeyboardEvent) {
		const typing = ['TEXTAREA', 'INPUT', 'SELECT'].includes((e.target as HTMLElement).tagName);
		if (e.ctrlKey && e.key === 'Enter' && !editing) {
			e.preventDefault();
			toggleReviewed();
			return;
		}
		if (typing) return;
		if (e.key === 'PageDown' || (e.altKey && e.key === 'ArrowRight')) step(1);
		if (e.key === 'PageUp' || (e.altKey && e.key === 'ArrowLeft')) step(-1);
		if (!e.ctrlKey && !e.altKey && e.key.toLowerCase() === 'f' && !editing) {
			e.preventDefault();
			stepEntry(e.shiftKey ? -1 : 1);
		}
		if (e.key === 'Escape' && selected !== null) void choose(null);
	}

	const reviewedCount = $derived(
		indices.filter((i) => views.some((v) => v.index === i && v.reviewed)).length
	);
	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void load();
	});
</script>

<svelte:window onkeydown={onKey} />

<div class="review">
	<header class="masthead">
		<div class="crumbs">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })} title={m.review.back}
				>{job?.name ?? m.review.title}</button
			>
			<span class="slash" aria-hidden="true">/</span>
			<h1 class="file mono">{source?.file_name ?? ''}</h1>
		</div>
		<div class="pager">
			<button
				class="btn sm icon ghost"
				aria-label={m.review.previous}
				title={m.review.previous}
				disabled={indices.indexOf(index) <= 0}
				onclick={() => step(-1)}><Icon name="previous" /></button
			>
			<select
				aria-label={m.review.title}
				value={index}
				onchange={(e) => goTo(Number(e.currentTarget.value))}
			>
				{#each indices as i (i)}
					{@const v = views.find((x) => x.index === i)}
					<option value={i}
						>{views.some((x) => x.index === i && x.reviewed)
							? m.review.pieceReviewed(pieceLabel(v))
							: pieceLabel(v)}</option
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
		</div>
		<div class="actions">
			{#if lead && lead.ai_text !== null && setups[0] && !editing}
				{@const first = setups[0]}
				<button class="btn" onclick={() => (editing = { setupId: first.id, text: base })}
					><Icon name="edit" size={14} />{m.review.edit}</button
				>
				{#if lead.reviewed}
					<button
						class="btn"
						aria-pressed="true"
						title={m.review.markNotReviewed}
						onclick={toggleReviewed}
						><Icon name="check" size={14} />{m.review.reviewed}<span class="kbd"
							>{m.review.reviewedShortcut}</span
						></button
					>
				{:else}
					<button class="btn primary" aria-pressed="false" onclick={toggleReviewed}
						><Icon name="check" size={14} />{m.review.markReviewed}<span class="kbd"
							>{m.review.reviewedShortcut}</span
						></button
					>
				{/if}
			{/if}
		</div>
	</header>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="work">
		<section class="pane source" class:stage={!recording} aria-label={m.review.source}>
			<div class="bar">
				<span class="bar-title mono">{m.review.source} · {pieceLabel(lead ?? current[0])}</span>
			</div>
			<div class="desk">
				{#if recording}
					<AudioPane
						{jobId}
						{sourceId}
						{index}
						offsetMs={timed?.piece.kind === 'segment' ? Number(timed.piece.start_ms) : 0}
						segments={timed?.segments ?? []}
					/>
				{:else}
					<PageImage {preview} {zoom} />
				{/if}
			</div>
			{#if !recording}
				<div class="zoom">
					<button
						class="zoom-btn"
						aria-label={m.review.zoomOut}
						title={m.review.zoomOut}
						onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}><Icon name="zoom-out" /></button
					>
					<span class="zoom-value mono">{Math.round(zoom * 100)}%</span>
					<button
						class="zoom-btn"
						aria-label={m.review.zoomIn}
						title={m.review.zoomIn}
						onclick={() => (zoom = Math.min(3, zoom + 0.2))}><Icon name="zoom-in" /></button
					>
					<span class="zoom-rule" aria-hidden="true"></span>
					<button class="zoom-btn wide" onclick={() => (zoom = 1)}>{m.review.fitWidth}</button>
				</div>
			{/if}
		</section>

		<section class="pane edition" aria-label={m.review.readingText}>
			<div class="bar">
				<span class="witness-head">
					{#if setups.length > 1}<i class="siglum">{siglum(0)}</i>{/if}
					<span class="mono">{setups[0]?.label ?? ''}</span>
					{#if corrected}<span class="chip hand">{m.review.corrected}</span>{/if}
				</span>
				{#if witnesses.length > 0 && !editing}
					<span class="diffs">
						{#if entries.length > 0}
							<span class="count"
								><span class="swatch" aria-hidden="true"></span>{m.review.differences(
									entries.length
								)}</span
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
							<span class="count muted">{m.review.noDifferences}</span>
						{/if}
						<label class="switch"
							><input type="checkbox" bind:checked={showDiff} /> {m.review.showDiff}</label
						>
					</span>
				{/if}
			</div>

			<div class="page" bind:this={textPane}>
				{#if editing && editedSetup}
					{@const target = editedSetup}
					<p class="editing-of">
						{m.review.correcting}
						{#if setups.length > 1}<i class="siglum">{siglum(setups.indexOf(target))}</i>{/if}
						<span class="mono">{target.label}</span>
					</p>
					<textarea
						class="editor"
						bind:value={editing.text}
						aria-label="{m.review.edit} {target.label}"></textarea>
					<div class="edit-actions">
						<button class="btn primary" onclick={() => editing && save(target, editing.text, true)}
							>{m.review.save}</button
						>
						<button class="btn ghost" onclick={() => (editing = null)}>{m.review.discard}</button>
						{#if viewFor(target)?.corrected_text != null}
							<button
								class="btn ghost"
								onclick={() => save(target, null, viewFor(target)?.reviewed ?? false)}
								>{m.review.revert}</button
							>
						{/if}
					</div>
				{:else if !lead || (lead.ai_text === null && lead.status !== 'failed')}
					<p class="muted">{m.review.notDone}</p>
				{:else if lead.status === 'failed'}
					<p class="error">{lead.error}</p>
				{:else}
					<div class="text" lang="">
						{#each reading as line, li (li)}
							<div class="line" class:blank={line.number === 0}>
								{#if numbered(line.number, entries)}<span class="n num" aria-hidden="true"
										>{line.number}</span
									>{/if}
								<!-- prettier-ignore -->
								<span class="t">{#each line.parts as part, k (k)}{#if part.entry !== null}{@const id = part.entry}{@const e = entries.find((x) => x.id === id)}<button class="lemma" data-entry={id} class:marked={showDiff} class:corr={e && isCorrection(e)} class:on={selected === id} aria-expanded={selected === id} onclick={() => choose(selected === id ? null : id)}>{part.text}</button>{:else}{part.text}{/if}{/each}</span>
							</div>
							{#if open && line.number === open.line}
								{@const e = open}
								<div class="readings" role="group" aria-label={m.review.disagree}>
									<div class="readings-head">
										<span>{isCorrection(e) ? m.review.corrected_here : m.review.disagree}</span>
										<span class="mono"
											>{m.review.differenceOf(
												entries.findIndex((x) => x.id === e.id) + 1,
												entries.length
											)}</span
										>
										<button
											class="btn sm icon ghost"
											aria-label={m.common.close}
											onclick={() => choose(null)}><Icon name="close" size={14} /></button
										>
									</div>
									<ul class="options">
										<li class="option">
											<span class="option-text in-text">{e.lemma || m.review.omitted}</span>
											<span class="option-who mono"
												>{corrected ? m.review.yourCorrections : (setups[0]?.label ?? '')}</span
											>
											<span class="chip">{m.review.inUse}</span>
										</li>
										{#each e.readings as r (r.siglum)}
											<li class="option">
												<span class="option-text">{r.text || m.review.omitted}</span>
												<span class="option-who mono">{witnessName(r.siglum)}</span>
												<button class="btn sm" onclick={() => useReading(e, r.text)}
													>{m.review.useReading}</button
												>
											</li>
										{/each}
										<li class="option own">
											<input
												type="text"
												bind:value={ownReading}
												aria-label={m.review.ownReading}
												placeholder={m.review.ownReadingPlaceholder}
												onkeydown={(ev) => {
													if (ev.key === 'Enter' && ownReading.trim()) {
														ev.preventDefault();
														void useReading(e, ownReading.trim());
													}
												}}
											/>
											<button
												class="btn sm primary"
												disabled={!ownReading.trim()}
												onclick={() => useReading(e, ownReading.trim())}>{m.review.apply}</button
											>
										</li>
									</ul>
									<p class="readings-note">{m.review.readingKept}</p>
								</div>
							{/if}
						{/each}
					</div>
				{/if}

				{#if !editing && recording}
					<SpeakerList
						{jobId}
						{sourceId}
						{setups}
						views={current}
						manyPieces={indices.length > 1}
						onsaved={async () => {
							views = await api.sourcePieces(jobId, sourceId);
						}}
					/>
				{/if}

				{#if !editing && setups.length > 1}
					<section class="models" aria-labelledby="models-head">
						<h2 id="models-head">{m.review.models}</h2>
						{#each setups as s, i (s.id)}
							{@const v = viewFor(s)}
							<div class="witness">
								<i class="siglum">{siglum(i)}</i>
								<span class="w-name mono">{s.label}</span>
								<span class="w-meta muted mono">
									{#if v?.status === 'failed'}<span class="error-text">{m.status.failed}</span>
									{:else}{v?.model_version ?? s.model}{v?.cost_usd
											? ` · ${formatUsd(v.cost_usd)}`
											: ''}{/if}
								</span>
								{#if i > 0 && v && v.ai_text !== null}
									<span class="tools">
										{#if v.corrected_text != null}<span class="chip hand">{m.review.corrected}</span
											>{/if}
										<button
											class="btn sm"
											onclick={() => (editing = { setupId: s.id, text: shownText(v) })}
											>{m.review.edit}</button
										>
										<label class="inline">
											<input
												type="checkbox"
												checked={v.reviewed}
												onchange={(e) => save(s, v.corrected_text, e.currentTarget.checked)}
											/>
											{m.review.markReviewed}
										</label>
									</span>
								{/if}
							</div>
						{/each}
					</section>
				{/if}
			</div>

			<div class="foot">
				<span class="mono">
					{#if report}
						{m.review.cer}:
						{#each report.scores as s, i (s.setup_id)}
							<span class="score"
								>{siglum(i).toUpperCase()}
								{s.cer !== null ? formatPercent(s.cer) : '–'}</span
							>
						{/each}
						{#if report.scores.some((s) => s.cer !== null)}<span class="hint"
								>· {m.review.scoredPages(report.scores[0]?.scored_pages ?? 0)}</span
							>{:else}<span class="hint">· {m.review.noReference}</span>{/if}
					{:else}
						{lead?.model_version ?? setups[0]?.model ?? ''}{lead?.cost_usd
							? ` · ${formatUsd(lead.cost_usd)}`
							: ''}
					{/if}
				</span>
				{#if witnesses.length > 0 || corrected}
					<span class="legend">
						{#if witnesses.length > 0}<span
								><i class="key marker" aria-hidden="true"></i>{m.review.legendDiffer}</span
							>{/if}
						<span><i class="key hand" aria-hidden="true"></i>{m.review.legendCorrected}</span>
					</span>
				{/if}
			</div>
		</section>
	</div>

	{#if indices.length > 1}
		<footer class="pages" aria-label={m.review.pages}>
			<ol class="strip">
				{#each indices as i (i)}
					{@const done = views.some((v) => v.index === i && v.reviewed)}
					<li>
						<button
							class="piece"
							class:current={i === index}
							class:done
							aria-current={i === index ? 'page' : undefined}
							aria-label={done
								? m.review.pieceReviewed(pieceLabel(views.find((v) => v.index === i)))
								: pieceLabel(views.find((v) => v.index === i))}
							onclick={() => goTo(i)}
						>
							{#if done}<Icon name="check" size={12} />{/if}<span class="mono"
								>{pieceNumber(i)}</span
							>
						</button>
					</li>
				{/each}
			</ol>
			<div class="tally">
				<span>{m.review.reviewedOf(reviewedCount, indices.length)}</span>
				<span class="tally-bar" aria-hidden="true"
					><i style:width="{(100 * reviewedCount) / Math.max(indices.length, 1)}%"></i></span
				>
			</div>
		</footer>
	{/if}
</div>

<style>
	.review {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.masthead .crumbs {
		flex: 1 1 16rem;
		align-items: baseline;
		flex-wrap: nowrap;
		gap: var(--space-2);
		min-width: 0;
	}
	.crumbs .back {
		font-size: var(--type-md);
		font-weight: 600;
		color: var(--text-bright);
		white-space: nowrap;
	}
	.crumbs .back:hover {
		text-decoration: underline;
		text-underline-offset: 0.2em;
	}
	.slash {
		color: var(--text-faint);
	}
	.file {
		font-size: var(--type-sm);
		font-weight: 400;
		letter-spacing: 0;
		color: var(--text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pager {
		display: flex;
		gap: var(--space-1);
		align-items: center;
		flex: none;
	}
	.pager select {
		min-height: var(--control-sm);
		max-width: 14rem;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
	}
	.actions {
		flex: 1 1 16rem;
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.error {
		color: var(--danger-text);
		padding: var(--space-2) var(--space-5);
	}
	.work {
		flex: 1;
		display: grid;
		grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.05fr);
		min-height: 0;
	}
	.pane {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-height: 0;
		position: relative;
	}
	.bar {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		min-height: 2.75rem;
		padding: var(--space-1) var(--space-4);
		border-bottom: 1px solid var(--line-soft);
	}
	.bar-title {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	/* The lightbox: dark in both themes, so the page is the brightest thing on screen. */
	.source {
		border-right: 1px solid var(--line);
		background: var(--surface-1);
	}
	.stage {
		background: var(--stage);
		border-right-color: var(--stage-line);
	}
	.stage .bar {
		border-bottom-color: var(--stage-line);
	}
	.stage .bar-title {
		color: var(--on-stage-muted);
	}
	.desk {
		overflow: auto;
		min-height: 0;
		padding: var(--space-4);
	}
	.stage .desk {
		padding: var(--space-5) var(--space-5) 4.5rem;
	}
	.stage .desk :global(.muted) {
		color: var(--on-stage-muted);
	}
	.zoom {
		position: absolute;
		left: 50%;
		bottom: var(--space-4);
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: 2px;
		padding: var(--space-1);
		border-radius: 9px;
		background: var(--stage-raised);
		box-shadow:
			0 0 0 1px var(--stage-line),
			0 8px 24px rgba(0, 0, 0, 0.35);
		color: var(--on-stage);
	}
	.zoom-btn {
		display: grid;
		place-items: center;
		min-width: var(--control-sm);
		height: var(--control-sm);
		padding: 0;
		border: 0;
		border-radius: 6px;
		background: transparent;
		color: inherit;
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.zoom-btn.wide {
		padding: 0 0.625rem;
	}
	.zoom-btn:hover {
		background: rgba(255, 255, 255, 0.08);
	}
	.zoom-btn:focus-visible {
		outline-color: var(--on-stage);
	}
	.zoom-value {
		min-width: 3rem;
		text-align: center;
		font-size: var(--type-xs);
	}
	.zoom-rule {
		width: 1px;
		height: 1.125rem;
		margin: 0 var(--space-1);
		background: var(--stage-line);
	}
	.edition {
		grid-template-rows: auto minmax(0, 1fr) auto;
		background: var(--paper);
	}
	.witness-head {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.diffs {
		margin-left: auto;
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-xs);
		color: var(--text-secondary);
	}
	.count {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		margin-right: var(--space-1);
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
	}
	/* The text being checked. */
	.page {
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-6) var(--space-7) var(--space-4);
	}
	.text {
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.7;
		color: var(--text-body);
		max-width: 42rem;
	}
	/* Each line sits in a grid with its number, printed every fifth line and on every line that
	   differs, on the line's first baseline. */
	.line {
		display: grid;
		grid-template-columns: 2.25rem minmax(0, 1fr);
		column-gap: var(--space-3);
		align-items: baseline;
		min-height: 1.7em;
	}
	.line.blank {
		min-height: 0.7em;
	}
	.t {
		grid-column: 2;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.n {
		grid-column: 1;
		justify-self: end;
		font-family: var(--font-mono);
		font-size: 0.6875rem;
		color: var(--text-faint);
	}
	.lemma {
		display: inline;
		padding: 0;
		border: 0;
		border-radius: 2px;
		background: none;
		color: inherit;
		font: inherit;
		line-height: inherit;
		text-align: inherit;
		white-space: pre-wrap;
		cursor: pointer;
		transition:
			background-color 120ms ease-out,
			box-shadow 120ms ease-out;
	}
	.lemma.marked {
		background: var(--marker);
		color: var(--on-marker);
		box-shadow: 0 0 0 2px var(--marker);
	}
	.lemma.corr {
		background: none;
		color: inherit;
		box-shadow: none;
		text-decoration: underline;
		text-decoration-color: var(--hand);
		text-decoration-thickness: 2px;
		text-underline-offset: 0.25em;
	}
	.lemma.on,
	.lemma:hover {
		box-shadow:
			0 0 0 2px var(--marker),
			0 0 0 3.5px var(--text-bright);
	}
	.lemma.corr.on,
	.lemma.corr:hover {
		box-shadow: none;
		background: var(--hand-wash);
	}
	/* The readings of one difference, opened under its line. */
	.readings {
		margin: var(--space-2) 0 var(--space-3) calc(2.25rem + var(--space-3));
		max-width: 30rem;
		padding: var(--space-3);
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		line-height: 1.45;
		color: var(--text-bright);
		display: grid;
		gap: var(--space-3);
	}
	.readings-head {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.readings-head > :first-child {
		flex: 1;
	}
	.options {
		list-style: none;
		margin: 0;
		padding: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-control);
	}
	.option {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-3);
		border-top: 1px solid var(--line-soft);
	}
	.option:first-child {
		border-top: 0;
	}
	.option-text {
		font-family: var(--font-text);
		font-size: var(--type-lg);
		overflow-wrap: anywhere;
	}
	.option-text.in-text {
		font-weight: 600;
	}
	.option-who {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 12rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.option.own {
		grid-template-columns: minmax(0, 1fr) auto;
	}
	.own input {
		min-height: var(--control-sm);
		font-family: var(--font-text);
		font-size: var(--type-md);
	}
	.readings-note {
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.models {
		margin-top: var(--space-6);
		padding-top: var(--space-4);
		border-top: 1px solid var(--line);
		max-width: 44rem;
		display: grid;
		gap: var(--space-1);
	}
	.models h2 {
		font-size: var(--type-sm);
		margin-bottom: var(--space-1);
	}
	.witness {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		min-height: 2.5rem;
	}
	.w-name {
		font-size: var(--type-sm);
		color: var(--text-bright);
	}
	.w-meta {
		font-size: var(--type-xs);
	}
	.error-text {
		color: var(--danger-text);
	}
	.tools {
		margin-left: auto;
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
	.foot {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2) var(--space-4);
		min-height: 2.25rem;
		padding: var(--space-1) var(--space-4);
		border-top: 1px solid var(--line-soft);
		font-size: var(--type-xs);
		color: var(--text-muted);
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
	.editing-of {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-secondary);
		font-size: var(--type-sm);
		margin-bottom: var(--space-3);
	}
	.editor {
		width: 100%;
		min-height: 24rem;
		max-width: 44rem;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.65;
	}
	.edit-actions {
		display: flex;
		gap: var(--space-2);
		margin-top: var(--space-3);
	}
	/* The pages of the source, each with its review state. */
	.pages {
		display: flex;
		align-items: center;
		gap: var(--space-5);
		padding: var(--space-2) var(--space-4);
		border-top: 1px solid var(--line);
		background: var(--surface-1);
	}
	.strip {
		flex: 1;
		min-width: 0;
		list-style: none;
		margin: 0;
		padding: 2px;
		display: flex;
		gap: var(--space-1);
		overflow-x: auto;
	}
	.piece {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 3px;
		min-width: 2.25rem;
		height: 2rem;
		padding: 0 var(--space-2);
		border: 1px solid var(--line);
		border-radius: 6px;
		background: var(--surface-2);
		color: var(--text-secondary);
		font-size: var(--type-xs);
		cursor: pointer;
	}
	.piece:hover {
		border-color: var(--line-hover);
	}
	.piece.done {
		color: var(--ok-text);
	}
	.piece.current {
		border-color: var(--text-bright);
		box-shadow: inset 0 0 0 1px var(--text-bright);
		color: var(--text-bright);
	}
	.tally {
		flex: none;
		display: grid;
		gap: var(--space-1);
		min-width: 12rem;
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-bright);
	}
	.tally-bar {
		display: block;
		height: 4px;
		border-radius: 2px;
		background: var(--line);
		overflow: hidden;
	}
	.tally-bar i {
		display: block;
		height: 100%;
		background: var(--ok);
	}
	@media (max-width: 1000px) {
		.work {
			grid-template-columns: 1fr;
			grid-template-rows: minmax(16rem, 42vh) minmax(0, 1fr);
		}
		.source {
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
	}
	@media (max-width: 820px) {
		.page {
			padding: var(--space-4) var(--space-4) var(--space-6) var(--space-2);
		}
		.readings {
			margin-left: 0;
		}
		.tally {
			display: none;
		}
	}
</style>
