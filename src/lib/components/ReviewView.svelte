<script lang="ts">
	import { onMount } from 'svelte';
	import PageImage from './PageImage.svelte';
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
	let editing = $state<Record<string, string>>({});
	let showDiff = $state(true);
	let diffs = $state<Record<string, DiffSpan[]>>({});
	let report = $state<ComparisonReport | null>(null);
	let error = $state<string | null>(null);

	const source = $derived(job?.sources.find((s) => s.id === sourceId));
	const indices = $derived([...new Set(views.map((v) => v.index))].sort((a, b) => a - b));
	const current = $derived(views.filter((v) => v.index === index));
	const comparison = $derived((job?.setups.length ?? 0) > 1);

	function viewFor(setup: Setup): PieceView | undefined {
		return current.find((v) => v.setup_id === setup.id);
	}

	function shownText(v: PieceView | undefined): string {
		return v?.corrected_text ?? v?.ai_text ?? '';
	}

	function pieceLabel(v: PieceView | undefined): string {
		if (!v) return '';
		if (v.piece.kind === 'page') {
			const half = v.piece.half ? ` (${v.piece.half})` : '';
			return m.review.page(`${v.piece.number}${half}`);
		}
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
			if (comparison) report = await api.compareSource(jobId, sourceId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	// Load the page image whenever the page changes.
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

	// Differences between the first set-up and each other one, on the shown texts.
	$effect(() => {
		if (!job || !comparison || !showDiff) {
			diffs = {};
			return;
		}
		const base = shownText(viewFor(job.setups[0]));
		for (const setup of job.setups.slice(1)) {
			const other = shownText(viewFor(setup));
			api
				.diffTexts(base, other)
				.then((spans) => (diffs = { ...diffs, [setup.id]: spans }))
				.catch(() => undefined);
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
			const { [setup.id]: _, ...rest } = editing;
			editing = rest;
			views = await api.sourcePieces(jobId, sourceId);
			if (comparison) report = await api.compareSource(jobId, sourceId);
			toast(m.review.saved);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	function step(delta: number) {
		const at = indices.indexOf(index);
		const next = indices[at + delta];
		if (next !== undefined) index = next;
	}

	function onKey(e: KeyboardEvent) {
		if ((e.target as HTMLElement).tagName === 'TEXTAREA') return;
		if (e.key === 'PageDown' || (e.altKey && e.key === 'ArrowRight')) step(1);
		if (e.key === 'PageUp' || (e.altKey && e.key === 'ArrowLeft')) step(-1);
	}

	const reviewedCount = $derived(
		job ? indices.filter((i) => views.some((v) => v.index === i && v.reviewed)).length : 0
	);

	onMount(() => {
		void load();
	});
</script>

<svelte:window onkeydown={onKey} />

<div class="review">
	<header>
		<div class="title">
			<button class="btn ghost sm" onclick={() => go({ name: 'jobs', jobId })}
				>← {m.review.back}</button
			>
			<h1>{source?.file_name ?? m.review.title}</h1>
			<span class="muted small">{m.review.reviewed}: {reviewedCount}/{indices.length}</span>
		</div>
		<div class="nav">
			<button class="btn sm" disabled={indices.indexOf(index) <= 0} onclick={() => step(-1)}
				>{m.review.previous}</button
			>
			<select aria-label={m.review.title} bind:value={index}>
				{#each indices as i (i)}
					{@const v = views.find((x) => x.index === i)}
					<option value={i}
						>{pieceLabel(v)}{views.some((x) => x.index === i && x.reviewed) ? ' ✓' : ''}</option
					>
				{/each}
			</select>
			<button
				class="btn sm"
				disabled={indices.indexOf(index) >= indices.length - 1}
				onclick={() => step(1)}>{m.review.next}</button
			>
		</div>
	</header>

	{#if error}<p class="chip danger">{error}</p>{/if}

	{#if comparison && report}
		<div class="scores card">
			<strong>{m.review.scores}</strong>
			{#each report.scores as s (s.setup_id)}
				<span class="score">
					<span class="mono">{s.label}</span>
					{#if s.cer !== null}
						{m.review.cer}
						{formatPercent(s.cer)} · {m.review.wer}
						{formatPercent(s.wer ?? 0)} ({m.review.scoredPages(s.scored_pages)})
					{/if}
					{#if s.cost_usd !== null}<span class="muted">{formatUsd(s.cost_usd)}</span>{/if}
				</span>
			{/each}
			{#if report.scores.every((s) => s.cer === null)}
				<span class="muted small">{m.review.noReference}</span>
			{/if}
			<label class="inline small"
				><input type="checkbox" bind:checked={showDiff} /> {m.review.showDiff}</label
			>
		</div>
	{/if}

	<div
		class="panes"
		style:grid-template-columns="minmax(0, 1.1fr) repeat({job?.setups.length ?? 1}, minmax(0, 1fr))"
	>
		<section class="pane source" aria-label={m.review.source}>
			<div class="pane-head">
				<span class="label">{m.review.source}</span>
				<div>
					<button
						class="btn ghost sm"
						aria-label={m.review.zoomOut}
						onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}>−</button
					>
					<button
						class="btn ghost sm"
						aria-label={m.review.zoomIn}
						onclick={() => (zoom = Math.min(3, zoom + 0.2))}>+</button
					>
				</div>
			</div>
			<PageImage {preview} {zoom} />
		</section>

		{#each job?.setups ?? [] as setup, col (setup.id)}
			{@const v = viewFor(setup)}
			<section class="pane" aria-label={setup.label}>
				<div class="pane-head">
					<span>
						<strong class="mono small">{comparison ? setup.label : m.review.aiText}</strong>
						{#if v?.corrected_text !== null && v?.corrected_text !== undefined}<span class="chip ok"
								>{m.review.corrected}</span
							>{/if}
						{#if v?.reviewed}<span class="chip ok">{m.review.reviewed}</span>{/if}
						{#if v?.status === 'failed'}<span class="chip danger">{m.status.failed}</span>{/if}
					</span>
					{#if v && v.status !== 'failed' && v.ai_text !== null}
						<span class="tools">
							{#if editing[setup.id] === undefined}
								<button
									class="btn sm"
									onclick={() => (editing = { ...editing, [setup.id]: shownText(v) })}
									>{m.review.edit}</button
								>
								<label class="inline small">
									<input
										type="checkbox"
										checked={v.reviewed}
										onchange={(e) => save(setup, v.corrected_text, e.currentTarget.checked)}
									/>
									{m.review.markReviewed}
								</label>
							{:else}
								<button class="btn sm primary" onclick={() => save(setup, editing[setup.id], true)}
									>{m.review.save}</button
								>
								<button
									class="btn sm ghost"
									onclick={() => {
										const { [setup.id]: _, ...rest } = editing;
										editing = rest;
									}}>{m.review.discard}</button
								>
							{/if}
						</span>
					{/if}
				</div>
				{#if !v || (v.ai_text === null && v.status !== 'failed')}
					<p class="muted pad">{m.review.notDone}</p>
				{:else if v.status === 'failed'}
					<p class="err pad">{v.error}</p>
				{:else if editing[setup.id] !== undefined}
					<textarea
						class="text"
						bind:value={editing[setup.id]}
						aria-label="{m.review.edit} {setup.label}"></textarea>
					{#if v.corrected_text !== null}
						<button class="btn sm ghost revert" onclick={() => save(setup, null, v.reviewed)}
							>{m.review.revert}</button
						>
					{/if}
				{:else if col > 0 && showDiff && diffs[setup.id]}
					<div class="text" lang="">
						{#each diffs[setup.id] as span, k (k)}
							{#if span.kind === 'same'}{span.text}{:else if span.kind === 'added'}<ins
									>{span.text}</ins
								>{:else}<del>{span.text}</del>{/if}
						{/each}
					</div>
				{:else}
					<div class="text">{shownText(v)}</div>
				{/if}
				{#if v?.model_version}<p class="muted tiny">
						{v.model_version}{v.cost_usd ? ` · ${formatUsd(v.cost_usd)}` : ''}
					</p>{/if}
			</section>
		{/each}
	</div>
</div>

<style>
	.review {
		display: grid;
		grid-template-rows: auto auto 1fr;
		gap: var(--space-3);
		height: calc(100vh - 2 * var(--space-5));
	}
	header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-4);
		flex-wrap: wrap;
	}
	.title,
	.nav {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.title h1 {
		font-size: var(--type-lg);
		overflow-wrap: anywhere;
	}
	.small {
		font-size: var(--type-sm);
	}
	.tiny {
		font-size: var(--type-xs);
		padding: 0 var(--space-3) var(--space-2);
	}
	.scores {
		display: flex;
		gap: var(--space-4);
		align-items: center;
		flex-wrap: wrap;
		padding: var(--space-2) var(--space-4);
		font-size: var(--type-sm);
	}
	.score {
		display: inline-flex;
		gap: var(--space-2);
		align-items: baseline;
	}
	.panes {
		display: grid;
		gap: var(--space-3);
		min-height: 0;
	}
	.pane {
		display: flex;
		flex-direction: column;
		min-height: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-1);
		overflow: hidden;
	}
	.pane-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-bottom: 1px solid var(--line);
		flex-wrap: wrap;
	}
	.pane-head > span {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	.tools {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-1);
		align-items: center;
	}
	.text {
		flex: 1;
		overflow: auto;
		padding: var(--space-4);
		background: var(--paper);
		font-family: var(--font-text);
		font-size: 1rem;
		line-height: 1.65;
		white-space: pre-wrap;
		unicode-bidi: plaintext;
		border: 0;
		border-radius: 0;
		min-height: 0;
	}
	textarea.text {
		resize: none;
		outline-offset: -2px;
	}
	ins {
		background: var(--diff-added);
		text-decoration: none;
		border-radius: 2px;
	}
	del {
		background: var(--diff-removed);
		border-radius: 2px;
	}
	.pad {
		padding: var(--space-4);
	}
	.err {
		color: var(--danger);
	}
	.revert {
		align-self: flex-start;
		margin: var(--space-2) var(--space-3);
	}
</style>
