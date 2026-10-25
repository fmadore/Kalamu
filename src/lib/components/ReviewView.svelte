<script lang="ts">
	// The review: the source beside the reading text, set as an edition. The first set-up's text
	// (with the researcher's corrections) is the reading text; where the other set-ups, or the
	// model under a correction, read differently, the apparatus below the text says so, by line.
	import { onMount } from 'svelte';
	import type { Attachment } from 'svelte/attachments';
	import AudioPane from './AudioPane.svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
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
	let hovered = $state<number | null>(null);

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

	/** The witnesses compared with the reading text: each other set-up, and the model itself
	 * when its text was corrected. */
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
	/** The line of the entry under the pointer or the keyboard, whose number lights up with it. */
	const hoveredLine = $derived(entries.find((e) => e.id === hovered)?.line ?? null);

	/** The way back from the text: pointing at a lemma lights up its entry and line number. */
	const followLemmata: Attachment<HTMLElement> = (text) => {
		const over = (e: PointerEvent) => {
			const lemma = (e.target as Element).closest<HTMLElement>('.lemma');
			hovered = lemma ? Number(lemma.dataset.entry) : null;
		};
		const leave = () => (hovered = null);
		text.addEventListener('pointerover', over);
		text.addEventListener('pointerleave', leave);
		return () => {
			text.removeEventListener('pointerover', over);
			text.removeEventListener('pointerleave', leave);
		};
	};

	/** Scrolls the edition page so an entry's lemma is in view, and the entry too when both fit. */
	function reveal(id: number, entry: HTMLElement) {
		const page = entry.closest<HTMLElement>('.page');
		const lemma = page?.querySelector<HTMLElement>(`.lemma[data-entry="${id}"]`);
		if (!page || !lemma) return;
		const box = page.getBoundingClientRect();
		const margin = 48;
		const top = lemma.getBoundingClientRect().top - box.top;
		const bottom = entry.getBoundingClientRect().bottom - box.top;
		if (top >= margin && bottom <= box.height - margin) return;
		const delta =
			bottom - top + 2 * margin <= box.height ? (top + bottom - box.height) / 2 : top - margin;
		const still = matchMedia('(prefers-reduced-motion: reduce)').matches;
		page.scrollBy({ top: delta, behavior: still ? 'auto' : 'smooth' });
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

	function step(delta: number) {
		const at = indices.indexOf(index);
		const next = indices[at + delta];
		if (next !== undefined) {
			index = next;
			editing = null;
		}
	}

	function onKey(e: KeyboardEvent) {
		if ((e.target as HTMLElement).tagName === 'TEXTAREA') return;
		if (e.key === 'PageDown' || (e.altKey && e.key === 'ArrowRight')) step(1);
		if (e.key === 'PageUp' || (e.altKey && e.key === 'ArrowLeft')) step(-1);
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
		<div class="title">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })}
				><Icon name="back" size={14} />{m.review.back}</button
			>
			<h1>
				{source?.file_name ?? m.review.title}
				<span class="lede num">{m.review.reviewedOf(reviewedCount, indices.length)}</span>
			</h1>
		</div>
		<div class="pager">
			<button class="btn" disabled={indices.indexOf(index) <= 0} onclick={() => step(-1)}
				>{m.review.previous}</button
			>
			<select
				aria-label={m.review.title}
				value={index}
				onchange={(e) => {
					index = Number(e.currentTarget.value);
					editing = null;
				}}
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
				class="btn"
				disabled={indices.indexOf(index) >= indices.length - 1}
				onclick={() => step(1)}>{m.review.next}</button
			>
		</div>
	</header>

	<div class="strip">
		{#if report}
			<span class="scores">
				{m.review.cer}:
				{#each report.scores as s, i (s.setup_id)}
					<span class="score">
						<i class="siglum">{siglum(i)}</i>
						{s.label}
						{#if s.cer !== null}<b class="num">{formatPercent(s.cer)}</b>{:else}<span class="muted"
								>–</span
							>{/if}
					</span>
				{/each}
				{#if report.scores.some((s) => s.cer !== null)}
					<span class="muted">({m.review.scoredPages(report.scores[0]?.scored_pages ?? 0)})</span>
				{:else}
					<span class="muted">{m.review.noReference}</span>
				{/if}
			</span>
		{:else}
			<span class="muted">{lead?.model_version ?? setups[0]?.model ?? ''}</span>
		{/if}
		{#if witnesses.length > 0}
			<label class="inline"
				><input type="checkbox" bind:checked={showDiff} /> {m.review.showDiff}</label
			>
		{/if}
	</div>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="work">
		<section class="pane source" aria-label={m.review.source}>
			<div class="bar">
				<span class="bar-title">{m.review.source}</span>
				{#if !recording}
					<div class="zoom">
						<button
							class="btn sm icon"
							aria-label={m.review.zoomOut}
							title={m.review.zoomOut}
							onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}><Icon name="zoom-out" /></button
						>
						<span class="num muted">{Math.round(zoom * 100)}%</span>
						<button
							class="btn sm icon"
							aria-label={m.review.zoomIn}
							title={m.review.zoomIn}
							onclick={() => (zoom = Math.min(3, zoom + 0.2))}><Icon name="zoom-in" /></button
						>
					</div>
				{/if}
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
		</section>

		<section class="pane edition" aria-label={m.review.readingText}>
			<div class="bar">
				<i class="siglum">{siglum(0)}</i>
				<span class="bar-title">{setups[0]?.label ?? ''}</span>
				{#if corrected}<span class="chip corr">{m.review.corrected}</span>{/if}
				{#if lead?.reviewed}<span class="chip">{m.review.reviewed}</span>{/if}
				{#if lead && lead.ai_text !== null && setups[0] && !editing}
					{@const first = setups[0]}
					<span class="tools">
						<button
							class="btn sm primary"
							onclick={() => (editing = { setupId: first.id, text: base })}>{m.review.edit}</button
						>
						<label class="inline">
							<input
								type="checkbox"
								checked={lead.reviewed}
								onchange={(e) => save(first, lead.corrected_text, e.currentTarget.checked)}
							/>
							{m.review.markReviewed}
						</label>
					</span>
				{/if}
			</div>

			<div class="page">
				<div class="running-head">
					<span class="smcp">{source?.file_name ?? ''} · {pieceLabel(lead ?? current[0])}</span>
					{#if !editing}<span class="smcp muted"
							>{m.review.readingTextOf} <i class="siglum">{siglum(0)}</i></span
						>{/if}
				</div>

				{#if editing && editedSetup}
					{@const target = editedSetup}
					<p class="editing-of">
						{m.review.correcting}
						<i class="siglum">{siglum(setups.indexOf(target))}</i>
						{target.label}
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
					<div class="text" lang="" {@attach followLemmata}>
						{#each reading as line, li (li)}
							<div class="line" class:blank={line.number === 0}>
								{#if numbered(line.number, entries)}<span
										class="n num"
										class:on={hoveredLine === line.number}
										aria-hidden="true">{line.number}</span
									>{/if}
								<!-- prettier-ignore -->
								<span class="t">{#each line.parts as part, k (k)}{#if part.entry !== null}{@const id = part.entry}<span class="lemma" data-entry={id} class:marked={showDiff} class:corr={isCorrection(entries.find((x) => x.id === id) ?? entries[0])} class:on={hovered === id}>{part.text}</span>{:else}{part.text}{/if}{/each}</span>
							</div>
						{/each}
					</div>
					{#if lead.keywords.length > 0}
						<p class="keywords" aria-label={m.review.keywords}>
							{#each lead.keywords as k (k)}<span class="chip">{k}</span>{/each}
						</p>
					{/if}
				{/if}

				{#if entries.length > 0 && !editing}
					<ol class="apparatus" aria-label={m.review.apparatus}>
						{#each entries as e (e.id)}
							<li class:on={hovered === e.id}>
								<button
									class="entry"
									title={m.review.showInText}
									onmouseenter={() => (hovered = e.id)}
									onmouseleave={() => (hovered = null)}
									onfocus={(ev) => {
										hovered = e.id;
										const el = ev.currentTarget;
										requestAnimationFrame(() => reveal(e.id, el));
									}}
									onblur={() => (hovered = null)}
									onclick={(ev) => reveal(e.id, ev.currentTarget)}
								>
									<span class="ln num">{e.line}</span>
									<span class:corr={isCorrection(e)}>{e.lemma || m.review.added}</span>
									{#if isCorrection(e)}<i class="siglum">{m.review.corrSiglum}</i>{/if}
									<span class="bracket">]</span>
									{#each e.readings as r, i (r.siglum)}
										{#if i > 0}<span class="sep">:</span>{/if}
										{r.text || m.review.omitted}
										<i class="siglum">{r.siglum}</i>
									{/each}
								</button>
							</li>
						{/each}
					</ol>
				{/if}

				{#if !editing}
					<div class="sigla">
						<h2 class="smcp">{m.review.sigla}</h2>
						{#each setups as s, i (s.id)}
							{@const v = viewFor(s)}
							<div class="witness">
								<i class="siglum">{siglum(i)}</i>
								<span class="w-name">{s.label}</span>
								<span class="w-meta muted num">
									{#if v?.status === 'failed'}<span class="error">{m.status.failed}</span>
									{:else}{v?.model_version ?? s.model}{v?.cost_usd
											? ` · ${formatUsd(v.cost_usd)}`
											: ''}{/if}
								</span>
								{#if i > 0 && v && v.ai_text !== null}
									<span class="tools">
										{#if v.corrected_text != null}<span class="chip corr">{m.review.corrected}</span
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
						{#if corrected}
							<div class="witness">
								<i class="siglum">{m.review.corrSiglum}</i>
								<span class="w-name">{m.review.yourCorrections}</span>
								<span class="w-meta muted">{m.review.correctionsKept}</span>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</section>
	</div>
</div>

<style>
	.review {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.title {
		display: grid;
		justify-items: start;
		gap: var(--space-1);
		min-width: 0;
	}
	h1 {
		display: flex;
		align-items: baseline;
		gap: var(--space-4);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	h1 .lede {
		font-family: var(--font-ui);
		font-size: var(--type-md);
	}
	.pager {
		display: flex;
		gap: var(--space-2);
		align-items: center;
		flex: none;
	}
	.strip {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-3) var(--space-6);
		border-bottom: 1px solid var(--line);
		color: var(--text-secondary);
	}
	.scores {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-1) var(--space-4);
	}
	.score b {
		color: var(--text-bright);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
	.error {
		color: var(--danger);
		padding: var(--space-2) var(--space-6);
	}
	.work {
		flex: 1;
		display: grid;
		grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
		min-height: 0;
	}
	.pane {
		display: grid;
		grid-template-rows: auto 1fr;
		min-height: 0;
	}
	.bar {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-height: 3.25rem;
		padding: 0 var(--space-5);
		border-bottom: 1px solid var(--line);
		background: var(--surface-0);
	}
	.bar-title {
		font-weight: 600;
		color: var(--text-bright);
	}
	.bar .siglum {
		font-size: 1.6rem;
	}
	.tools {
		margin-left: auto;
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}
	.zoom {
		margin-left: auto;
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.zoom .num {
		width: 3rem;
		text-align: center;
	}
	.btn.icon {
		width: var(--control-sm);
		padding: 0;
	}
	.back {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
	}
	.source {
		border-right: 1px solid var(--line);
	}
	.desk {
		background: var(--desk);
		overflow: auto;
		padding: var(--space-5);
		min-height: 0;
	}
	.chip.corr {
		border-color: var(--rubric);
		color: var(--rubric);
	}
	/* The edition page. */
	.page {
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-6) var(--space-7) 5rem;
		background: var(--paper);
	}
	.running-head {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: var(--space-4);
		margin-bottom: var(--space-5);
		font-size: 1.125rem;
	}
	.text {
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.65;
		color: var(--text-bright);
		font-variant-numeric: oldstyle-nums;
		max-width: 44rem;
	}
	/* Each line sits in the margin's grid: its number, when printed, shares the first baseline. */
	.line {
		display: grid;
		grid-template-columns: 2.5rem minmax(0, 1fr);
		column-gap: 1rem;
		align-items: baseline;
		margin-left: -3.5rem;
		min-height: 1.65em;
	}
	.line.blank {
		min-height: 0.6em;
	}
	.t {
		grid-column: 2;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.n {
		grid-column: 1;
		justify-self: end;
		padding: 0 0.2em;
		border-radius: 2px;
		font-size: 0.8em;
		font-weight: 600;
		font-variant-numeric: lining-nums tabular-nums;
		color: var(--rubric);
		transition: background-color 120ms ease-out;
	}
	.n.on {
		background: var(--tint);
	}
	.lemma {
		border-radius: 2px;
		transition: background-color 120ms ease-out;
	}
	.lemma.marked {
		background: var(--tint);
		box-shadow: 0 0 0 2px var(--tint);
	}
	.lemma.corr {
		color: var(--rubric);
	}
	.lemma.on {
		background: var(--accent-chip);
		box-shadow: 0 0 0 2px var(--accent-chip);
		outline: 1px solid var(--accent-border);
	}
	.keywords {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
		margin-top: var(--space-4);
	}
	.apparatus {
		list-style: none;
		margin: var(--space-5) 0 0;
		padding: var(--space-3) 0 0;
		border-top: 1px solid var(--rule);
		font-family: var(--font-text);
		font-size: 1.0625rem;
		line-height: 1.6;
		max-width: 44rem;
		font-variant-numeric: oldstyle-nums;
	}
	.apparatus li {
		position: relative;
		border-radius: 3px;
		padding: 0 var(--space-2);
		margin: 0 calc(-1 * var(--space-2));
		transition: background-color 120ms ease-out;
	}
	.apparatus li.on {
		background: var(--accent-wash);
	}
	.entry {
		display: block;
		width: 100%;
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.entry:focus-visible {
		outline-offset: 2px;
	}
	.ln {
		position: absolute;
		left: -3rem;
		width: 2.5rem;
		text-align: right;
		font-weight: 600;
		font-variant-numeric: lining-nums tabular-nums;
	}
	.apparatus .corr {
		color: var(--rubric);
	}
	.bracket,
	.sep {
		margin: 0 0.3em 0 0.15em;
	}
	.apparatus .siglum {
		margin-left: 0.15em;
	}
	.sigla {
		margin-top: var(--space-6);
		padding-top: var(--space-3);
		border-top: 1px solid var(--line);
		max-width: 44rem;
	}
	.sigla h2 {
		font-size: 1.0625rem;
		color: var(--text-muted);
		margin-bottom: var(--space-1);
	}
	.witness {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-height: 2.5rem;
		position: relative;
	}
	.witness .siglum {
		position: absolute;
		left: -3rem;
		width: 2.5rem;
		text-align: right;
	}
	.w-name {
		font-weight: 600;
		color: var(--text-bright);
	}
	.w-meta {
		font-size: var(--type-sm);
	}
	.editing-of {
		color: var(--text-secondary);
		margin-bottom: var(--space-2);
	}
	.editor {
		width: 100%;
		min-height: 22rem;
		max-width: 44rem;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.6;
	}
	.edit-actions {
		display: flex;
		gap: var(--space-2);
		margin-top: var(--space-3);
	}
	@media (max-width: 960px) {
		.work {
			grid-template-columns: 1fr;
			grid-template-rows: minmax(16rem, 40vh) 1fr;
		}
		.source {
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
	}
	@media (max-width: 820px) {
		.strip {
			flex-wrap: wrap;
			padding: var(--space-3) var(--space-4);
		}
		.bar {
			flex-wrap: wrap;
			padding: var(--space-2) var(--space-4);
		}
		.page {
			padding: var(--space-4) var(--space-4) var(--space-6) 3.5rem;
		}
		.desk {
			padding: var(--space-3);
		}
		h1 {
			white-space: normal;
		}
	}
</style>
