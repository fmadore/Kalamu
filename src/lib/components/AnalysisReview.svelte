<script lang="ts">
	// The review of an analysis: the document beside its answers, set like an edition's index.
	// The first set-up's answers lead. In a comparison every value carries the sigla of the
	// set-ups that gave it, so agreement reads at a glance; pointing at a name finds it in the
	// document. Corrections are kept apart from the model's answers.
	import { onMount } from 'svelte';
	import Icon from './Icon.svelte';
	import PageImage from './PageImage.svelte';
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

	const kindLabel = (f: Field) => m.fields.kinds[f.kind.type];
	const editedSetup = $derived(setups.find((s) => s.id === editing?.setupId));

	onMount(() => {
		void load();
	});
</script>

<div class="review">
	<header class="masthead">
		<div class="title">
			<button class="back" onclick={() => go({ name: 'jobs', jobId })}
				><Icon name="back" size={14} />{m.review.back}</button
			>
			<h1>
				{source?.file_name ?? m.review.title}
				<span class="lede">{lead?.reviewed ? m.review.reviewed : m.review.notReviewed}</span>
			</h1>
		</div>
	</header>

	<div class="strip">
		<span class="witnesses">
			{#each setups as s, i (s.id)}
				<span><i class="siglum">{siglum(i)}</i> {s.label}</span>
			{/each}
		</span>
		<span class="muted">{setups[0]?.prompt.preset_name ?? ''}</span>
	</div>

	{#if error}<p class="error" role="alert">{error}</p>{/if}

	<div class="work">
		<section class="pane source" aria-label={m.review.source}>
			<div class="bar">
				<span class="bar-title">{m.review.source}</span>
				{#if source?.kind === 'pdf' || source?.kind === 'image'}
					<div class="zoom">
						{#if pages > 1}
							<button class="btn sm" disabled={page === 0} onclick={() => (page -= 1)}
								>{m.review.previous}</button
							>
							<span class="num muted">{m.review.page(String(page + 1))}</span>
							<button class="btn sm" disabled={page >= pages - 1} onclick={() => (page += 1)}
								>{m.review.next}</button
							>
						{/if}
						<button
							class="btn sm icon"
							aria-label={m.review.zoomOut}
							title={m.review.zoomOut}
							onclick={() => (zoom = Math.max(0.4, zoom - 0.2))}><Icon name="zoom-out" /></button
						>
						<button
							class="btn sm icon"
							aria-label={m.review.zoomIn}
							title={m.review.zoomIn}
							onclick={() => (zoom = Math.min(3, zoom + 0.2))}><Icon name="zoom-in" /></button
						>
					</div>
				{/if}
			</div>
			{#if text !== null}
				<div class="document" lang="">
					{#each findTerm(text, hovered ?? '') as part, k (k)}{#if part.hit}<mark>{part.text}</mark
							>{:else}{part.text}{/if}{/each}
				</div>
			{:else}
				<div class="desk"><PageImage {preview} {zoom} /></div>
			{/if}
		</section>

		<section class="pane edition" aria-label={m.review.answers}>
			<div class="bar">
				<i class="siglum">{siglum(0)}</i>
				<span class="bar-title">{setups[0]?.label ?? ''}</span>
				{#if lead?.corrected_fields}<span class="chip corr">{m.review.corrected}</span>{/if}
				{#if lead && setups[0] && lead.status !== 'failed' && !editing}
					{@const first = setups[0]}
					<span class="tools">
						<button class="btn sm primary" onclick={() => startEditing(first)}
							>{m.review.edit}</button
						>
						<label class="inline">
							<input
								type="checkbox"
								checked={lead.reviewed}
								onchange={(e) => save(first, lead.corrected_fields, e.currentTarget.checked)}
							/>
							{m.review.markReviewed}
						</label>
					</span>
				{/if}
			</div>

			<div class="page">
				<div class="running-head">
					<span class="smcp">{source?.file_name ?? ''}</span>
					<span class="smcp muted">{m.review.answersOf} <i class="siglum">{siglum(0)}</i></span>
				</div>

				{#if editing && editedSetup}
					{@const target = editedSetup}
					{@const draft = editing.draft}
					<p class="editing-of">
						{m.review.correcting}
						<i class="siglum">{siglum(setups.indexOf(target))}</i>
						{target.label}
					</p>
					{#each fields as f (f.key)}
						<div class="answer">
							<h2 class="smcp">{f.label} <span class="kind">{kindLabel(f)}</span></h2>
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
							<h2 class="smcp">{f.label}</h2>
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

				{#if !editing}
					<div class="sigla">
						<h2 class="smcp">{m.review.sigla}</h2>
						{#each setups as s, i (s.id)}
							{@const v = viewOf(s)}
							<div class="witness">
								<i class="siglum">{siglum(i)}</i>
								<span class="w-name">{s.label}</span>
								<span class="w-meta muted num">
									{#if v?.status === 'failed'}<span class="error">{m.status.failed}</span>
									{:else}{v?.model_version ?? s.model}{v?.cost_usd
											? ` · ${formatUsd(v.cost_usd)}`
											: ''}{/if}
								</span>
								{#if i > 0 && v && v.status !== 'failed'}
									<span class="tools">
										{#if v.corrected_fields}<span class="chip corr">{m.review.corrected}</span>{/if}
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
						{#if lead?.corrected_fields}
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
	.back {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
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
	.witnesses {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-4);
	}
	.error {
		color: var(--danger);
		padding: var(--space-2) var(--space-6);
	}
	.work {
		flex: 1;
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
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
	.tools,
	.zoom {
		margin-left: auto;
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.btn.icon {
		width: var(--control-sm);
		padding: 0;
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
	/* A text document, read as an edition reads: Garamond on the page, a found name in tint. */
	.document {
		overflow: auto;
		min-height: 0;
		padding: var(--space-5) var(--space-6);
		background: var(--paper);
		font-family: var(--font-text);
		font-size: 1.1875rem;
		line-height: 1.6;
		color: var(--text-bright);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	mark {
		background: var(--tint);
		color: inherit;
		border-radius: 2px;
		box-shadow: 0 0 0 2px var(--tint);
	}
	.chip.corr {
		border-color: var(--rubric);
		color: var(--rubric);
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
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
		margin-bottom: var(--space-4);
		font-size: 1.125rem;
	}
	.answer {
		max-width: 44rem;
		padding: var(--space-3) 0 var(--space-4);
		border-top: 1px solid var(--line);
		font-family: var(--font-text);
		font-size: 1.1875rem;
		line-height: 1.55;
		color: var(--text-bright);
	}
	.answer:first-of-type {
		border-top: 1px solid var(--rule);
	}
	.answer h2 {
		font-size: 1rem;
		color: var(--text-muted);
		margin-bottom: var(--space-1);
	}
	.kind {
		font-family: var(--font-ui);
		font-variant-caps: normal;
		letter-spacing: 0;
		font-size: var(--type-xs);
		margin-left: var(--space-2);
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
		background: var(--tint);
		box-shadow: 0 0 0 2px var(--tint);
	}
	/* The sigla of the set-ups that found a name, when not all of them did. */
	.found {
		font-family: var(--font-text);
		font-weight: 600;
		font-variant-caps: all-small-caps;
		color: var(--rubric);
		margin-left: 0.1em;
	}
	.sep {
		color: var(--text-muted);
	}
	.choice {
		font-weight: 500;
	}
	.reason {
		font-size: 1.0625rem;
		font-style: italic;
		color: var(--text-secondary);
	}
	.none {
		font-family: var(--font-ui);
		font-size: var(--type-sm);
	}
	.small {
		font-size: var(--type-sm);
		font-family: var(--font-ui);
	}
	.checks {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-4);
		font-family: var(--font-ui);
		font-size: var(--type-md);
	}
	.editing-of {
		color: var(--text-secondary);
		margin-bottom: var(--space-2);
	}
	.editor {
		width: 100%;
		min-height: 8rem;
		font-family: var(--font-text);
		font-size: 1.0625rem;
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
		h1 {
			white-space: normal;
		}
	}
</style>
