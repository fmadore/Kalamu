<script lang="ts">
	// The review of an analysis: the document beside its answers.
	// The first set-up's answers lead. In a comparison every value carries the sigla of the
	// set-ups that gave it, so agreement reads at a glance; pointing at a name finds it in the
	// document. Corrections are kept apart from the model's answers.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
	import Splitter from './Splitter.svelte';
	import { collate, findTerm, type Witness } from '#lib/analysis.js';
	import { siglum } from '#lib/apparatus.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { describeError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { go, toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let views = $state<PieceView[]>([]);
	let text = $state<string | null>(null);
	let preview = $state<PagePreview | null>(null);
	let page = $state(0);
	let zoom = $state(1);
	let hovered = $state<string | null>(null);
	let error = $state<string | null>(null);
	/** The answers being corrected: text per field, lists one item per line. */
	let editing = $state<{ setupId: string; draft: Record<string, string | string[]> } | null>(null);

	const source = $derived(job?.sources.find((s) => s.id === sourceId));
	const setups = $derived(job?.setups ?? []);
	const fields = $derived(setups[0]?.prompt.schema?.fields ?? []);
	const pages = $derived(source?.kind === 'pdf' ? (source.page_count ?? 1) : 1);
	const viewOf = (s: Setup | undefined) => views.find((v) => v.setup_id === s?.id);
	const answersOf = (v: PieceView | undefined): FieldAnswer[] =>
		v?.corrected_fields ?? v?.ai_fields ?? [];
	const lead = $derived(viewOf(setups[0]));
	const witnesses: Witness[] = $derived(
		setups.map((s, i) => ({ siglum: siglum(i), answers: answersOf(viewOf(s)) }))
	);
	const comparing = $derived(setups.length > 1);

	async function load() {
		try {
			job = await api.getJob(jobId);
			views = await api.sourcePieces(jobId, sourceId);
			if (source?.kind === 'text') text = await api.sourceText(jobId, sourceId);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	// A PDF or image is shown page by page; a text document as text.
	$effect(() => {
		const p = page;
		preview = null;
		if (source?.kind === 'pdf' || source?.kind === 'image') {
			api
				.piecePreview(jobId, sourceId, p)
				.then((r) => {
					if (page === p) preview = r;
				})
				.catch(() => (preview = { mime: 'none', data: '' }));
		}
	});

	function startEditing(setup: Setup) {
		const answers = answersOf(viewOf(setup));
		const draft: Record<string, string | string[]> = {};
		for (const f of fields) {
			const value = answers.find((a) => a.key === f.key)?.value ?? '';
			const items = Array.isArray(value) ? value : value ? [value] : [];
			draft[f.key] =
				f.kind.type === 'list'
					? items.join('\n')
					: f.kind.type === 'choices'
						? items
						: (items[0] ?? '');
		}
		editing = { setupId: setup.id, draft };
	}

	/** The draft as answers, keeping the model's reasons. */
	function answersFrom(setup: Setup, draft: Record<string, string | string[]>): FieldAnswer[] {
		const before = answersOf(viewOf(setup));
		return fields.map((f) => {
			const raw = draft[f.key] ?? '';
			const value =
				f.kind.type === 'list'
					? String(raw)
							.split('\n')
							.map((l) => l.trim())
							.filter(Boolean)
					: f.kind.type === 'choices'
						? (raw as string[])
						: String(raw).trim();
			const reason = before.find((a) => a.key === f.key)?.reason;
			return reason ? { key: f.key, value, reason } : { key: f.key, value };
		});
	}

	async function save(setup: Setup, answers: FieldAnswer[] | null, reviewed: boolean) {
		error = null;
		try {
			await api.saveCorrection({
				job_id: jobId,
				setup_id: setup.id,
				source_id: sourceId,
				index: 0,
				text: null,
				fields: answers ?? undefined,
				reviewed,
				updated_at: 0
			});
			editing = null;
			views = await api.sourcePieces(jobId, sourceId);
			toast(m.review.saved);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	// Where the boundary between the document and the answers sits, remembered on this computer.
	const SPLIT_KEY = 'kalamu.analysis.split';
	const SPLIT = 0.5;
	function storedSplit(): number {
		try {
			const v = Number(localStorage.getItem(SPLIT_KEY));
			if (v > 0 && v < 1) return v;
		} catch {
			// No storage: the default split.
		}
		return SPLIT;
	}
	let split = $state(storedSplit());
	let workWidth = $state(0);
	$effect(() => {
		const v = String(split);
		try {
			localStorage.setItem(SPLIT_KEY, v);
		} catch {
			// The split is only a convenience.
		}
	});
	function setSplit(at: number) {
		const min = workWidth > 0 ? Math.min(0.4, 280 / workWidth) : 0.25;
		split = Math.min(Math.max(at, min), 1 - min);
	}

	const kindLabel = (f: Field) => m.fields.kinds[f.kind.type];
	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void load();
	});
</script>

<div class="review">
	<header class="masthead">
		<div class="crumbs">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })} title={m.review.back}
				>{job?.name ?? m.review.title}</button
			>
			<span class="slash" aria-hidden="true">/</span>
			<h1 class="file mono">{source?.file_name ?? ''}</h1>
		</div>
		<div class="actions">
			{#if lead && setups[0] && lead.status !== 'failed' && !editing}
				{@const first = setups[0]}
				<button class="btn" onclick={() => startEditing(first)}
					><Icon name="edit" size={14} />{m.review.edit}</button
				>
				{#if lead.reviewed}
					<button
						class="btn"
						aria-pressed="true"
						title={m.review.markNotReviewed}
						onclick={() => save(first, lead.corrected_fields, false)}
						><Icon name="check" size={14} />{m.review.reviewed}</button
					>
				{:else}
					<button
						class="btn primary"
						aria-pressed="false"
						onclick={() => save(first, lead.corrected_fields, true)}
						><Icon name="check" size={14} />{m.review.markReviewed}</button
					>
				{/if}
			{/if}
		</div>
	</header>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div
		class="work"
		style:--columns="minmax(0, {split}fr) minmax(0, {1 - split}fr)"
		bind:clientWidth={workWidth}
	>
		<section
			class="pane source"
			class:stage={source?.kind === 'pdf' || source?.kind === 'image'}
			aria-label={m.review.source}
		>
			<div class="bar">
				<span class="bar-title mono"
					>{m.review.source}{#if source?.kind === 'pdf' && pages > 1}
						· {m.review.page(String(page + 1))}{/if}</span
				>
				{#if source?.kind === 'pdf' && pages > 1}
					<span class="paging">
						<button
							class="zoom-btn"
							aria-label={m.review.previous}
							disabled={page === 0}
							onclick={() => (page -= 1)}><Icon name="previous" /></button
						>
						<button
							class="zoom-btn"
							aria-label={m.review.next}
							disabled={page >= pages - 1}
							onclick={() => (page += 1)}><Icon name="chevron" /></button
						>
					</span>
				{/if}
			</div>
			{#if text !== null}
				<div class="document" lang="">
					{#each findTerm(text, hovered ?? '') as part, k (k)}{#if part.hit}<mark>{part.text}</mark
							>{:else}{part.text}{/if}{/each}
				</div>
			{:else}
				<div class="desk"><PageImage {preview} {zoom} /></div>
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
				</div>
			{/if}
		</section>

		<section class="pane edition" aria-label={m.review.answers}>
			<div class="bar">
				<span class="witness-head">
					{#if comparing}<i class="siglum">{siglum(0)}</i>{/if}
					<span class="mono">{setups[0]?.label ?? ''}</span>
					{#if lead?.corrected_fields}<span class="chip hand">{m.review.corrected}</span>{/if}
				</span>
				<span class="preset">{setups[0]?.prompt.preset_name ?? ''}</span>
			</div>

			<div class="page">
				{#if editing && editedSetup}
					{@const target = editedSetup}
					{@const draft = editing.draft}
					<p class="editing-of">
						{m.review.correcting}
						{#if comparing}<i class="siglum">{siglum(setups.indexOf(target))}</i>{/if}
						<span class="mono">{target.label}</span>
					</p>
					{#each fields as f (f.key)}
						<div class="answer">
							<h2>{f.label} <span class="kind">{kindLabel(f)}</span></h2>
							{#if f.kind.type === 'choice'}
								<select bind:value={draft[f.key]} aria-label={f.label}>
									{#each f.kind.values as v (v)}<option value={v}>{v}</option>{/each}
								</select>
							{:else if f.kind.type === 'choices'}
								{@const chosen = draft[f.key] as string[]}
								<div class="checks">
									{#each f.kind.values as v (v)}
										<label class="inline">
											<input
												type="checkbox"
												checked={chosen.includes(v)}
												onchange={(e) =>
													(draft[f.key] = e.currentTarget.checked
														? [...chosen, v]
														: chosen.filter((x) => x !== v))}
											/>
											{v}
										</label>
									{/each}
								</div>
							{:else}
								<textarea
									class="editor"
									class:short={f.kind.type === 'list'}
									aria-label={f.label}
									bind:value={draft[f.key]}></textarea>
								{#if f.kind.type === 'list'}<p class="muted small">{m.review.onePerLine}</p>{/if}
							{/if}
						</div>
					{/each}
					<div class="edit-actions">
						<button
							class="btn primary"
							onclick={() => save(target, answersFrom(target, draft), true)}>{m.review.save}</button
						>
						<button class="btn ghost" onclick={() => (editing = null)}>{m.review.discard}</button>
						{#if viewOf(target)?.corrected_fields}
							<button
								class="btn ghost"
								onclick={() => save(target, null, viewOf(target)?.reviewed ?? false)}
								>{m.review.revert}</button
							>
						{/if}
					</div>
				{:else if !lead || (lead.ai_fields.length === 0 && lead.status !== 'failed')}
					<p class="muted">{m.review.notDone}</p>
				{:else if lead.status === 'failed'}
					<p class="error">{lead.error}</p>
				{:else}
					{#each fields as f (f.key)}
						{@const readings = collate(f.key, witnesses)}
						<div class="answer">
							<h2>{f.label}</h2>
							{#if readings.length === 0}
								<p class="muted none">{m.review.noneFound}</p>
							{:else if f.kind.type === 'list' || f.kind.type === 'choices'}
								<p class="items">
									{#each readings as r, k (r.value)}{#if k > 0}<span class="sep">{'; '}</span
											>{/if}<span
											class="item"
											class:partial={comparing && r.sigla.length < setups.length}
											role="mark"
											onpointerenter={() => (hovered = r.value)}
											onpointerleave={() => (hovered = null)}
											>{r.value}{#if comparing && r.sigla.length < setups.length}<sup class="found"
													>{r.sigla.join(' ')}</sup
												>{/if}</span
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

				{#if !editing && comparing}
					<section class="models" aria-labelledby="models-head">
						<h2 id="models-head">{m.review.models}</h2>
						{#each setups as s, i (s.id)}
							{@const v = viewOf(s)}
							<div class="witness">
								<i class="siglum">{siglum(i)}</i>
								<span class="w-name mono">{s.label}</span>
								<span class="w-meta muted mono">
									{#if v?.status === 'failed'}<span class="error-text">{m.status.failed}</span>
									{:else}{v?.model_version ?? s.model}{v?.cost_usd
											? ` · ${formatUsd(v.cost_usd)}`
											: ''}{/if}
								</span>
								{#if i > 0 && v && v.status !== 'failed'}
									<span class="tools">
										{#if v.corrected_fields}<span class="chip hand">{m.review.corrected}</span>{/if}
										<button class="btn sm" onclick={() => startEditing(s)}>{m.review.edit}</button>
										<label class="inline">
											<input
												type="checkbox"
												checked={v.reviewed}
												onchange={(e) => save(s, v.corrected_fields, e.currentTarget.checked)}
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

			<div class="foot mono">
				{lead?.model_version ?? setups[0]?.model ?? ''}{lead?.cost_usd
					? ` · ${formatUsd(lead.cost_usd)}`
					: ''}
			</div>
		</section>
		<Splitter
			label={m.review.resizeSource}
			position={split}
			onset={setSplit}
			onreset={() => (split = SPLIT)}
		/>
	</div>
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
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.error {
		color: var(--danger-text);
		padding: var(--space-2) var(--space-5);
	}
	.work {
		--columns: minmax(0, 1fr) minmax(0, 1fr);
		flex: 1;
		display: grid;
		grid-template-columns: var(--columns);
		min-height: 0;
		position: relative;
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
	.source {
		border-right: 1px solid var(--line);
		background: var(--surface-1);
	}
	/* The lightbox under a scan, as in the text review. */
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
	.paging {
		margin-left: auto;
		display: flex;
		gap: 2px;
		color: var(--on-stage);
	}
	.desk {
		overflow: auto;
		min-height: 0;
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
		cursor: pointer;
	}
	.zoom-btn:hover:not(:disabled) {
		background: rgba(255, 255, 255, 0.08);
	}
	.zoom-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.zoom-btn:focus-visible {
		outline-color: var(--on-stage);
	}
	.zoom-value {
		min-width: 3rem;
		text-align: center;
		font-size: var(--type-xs);
	}
	/* A text document, set in the reading face; a name pointed at is found in it. */
	.document {
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-6);
		background: var(--paper);
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.65;
		color: var(--text-body);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	mark {
		background: var(--marker);
		color: var(--on-marker);
		border-radius: 2px;
		box-shadow: 0 0 0 2px var(--marker);
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
	.preset {
		margin-left: auto;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
		font-size: var(--type-sm);
	}
	.page {
		overflow: auto;
		min-height: 0;
		padding: var(--space-4) var(--space-6) var(--space-7);
	}
	/* One answer: a sans label over the value in the reading face. */
	.answer {
		max-width: 44rem;
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
	.answer h2 {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
		margin-bottom: var(--space-2);
	}
	.kind {
		font-weight: 400;
		margin-left: var(--space-2);
		color: var(--text-faint);
	}
	.answer p + p {
		margin-top: var(--space-2);
	}
	.answer .siglum {
		margin-right: 0.35em;
	}
	.items .item {
		border-radius: 2px;
		cursor: default;
		transition: background-color 120ms ease-out;
	}
	.items .item:hover {
		background: var(--marker);
		color: var(--on-marker);
		box-shadow: 0 0 0 2px var(--marker);
	}
	/* The sigla of the set-ups that found a name, when not all of them did. */
	.found {
		font-family: var(--font-mono);
		font-size: 0.6875rem;
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
	.small {
		font-size: var(--type-xs);
		font-family: var(--font-ui);
	}
	.checks {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-4);
		font-family: var(--font-ui);
	}
	.editing-of {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-secondary);
		font-size: var(--type-sm);
		margin-bottom: var(--space-2);
	}
	.editor {
		width: 100%;
		min-height: 8rem;
		font-family: var(--font-text);
		font-size: var(--type-md);
		line-height: 1.5;
	}
	.editor.short {
		min-height: 6rem;
	}
	.edit-actions {
		display: flex;
		gap: var(--space-2);
		margin-top: var(--space-3);
	}
	.models {
		margin-top: var(--space-5);
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
	.foot {
		display: flex;
		align-items: center;
		min-height: 2.25rem;
		padding: var(--space-1) var(--space-4);
		border-top: 1px solid var(--line-soft);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	@media (max-width: 1000px) {
		.work {
			grid-template-columns: 1fr;
			grid-template-rows: minmax(16rem, 42vh) minmax(0, 1fr);
		}
		.work :global(.splitter) {
			display: none;
		}
		.source {
			border-right: 0;
			border-bottom: 1px solid var(--line);
		}
	}
	@media (max-width: 820px) {
		.page {
			padding: var(--space-4);
		}
	}
</style>
