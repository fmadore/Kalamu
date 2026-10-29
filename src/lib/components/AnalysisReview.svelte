<script lang="ts">
	// The review of an analysis: the document beside its answers.
	// The first set-up's answers lead. In a comparison every value carries the sigla of the
	// set-ups that gave it, so agreement reads at a glance; pointing at a name finds it in the
	// document. Corrections are kept apart from the model's answers.
	import { onMount } from 'svelte';
	import CorrectionEditor from './CorrectionEditor.svelte';
	import FieldInput from './FieldInput.svelte';
	import Icon from './Icon.svelte';
	import ReviewFrame from './ReviewFrame.svelte';
	import SiglaKey from './SiglaKey.svelte';
	import { collate, type Witness } from '#lib/analysis.js';
	import { siglum } from '#lib/apparatus.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldAnswer } from '#lib/bindings/FieldAnswer.js';
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { type Draft, fromDraft, toDraft } from '#lib/fields.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { Review } from '#lib/review.svelte.js';

	let { jobId, sourceId }: { jobId: string; sourceId: string } = $props();
	const m = messages();
	// svelte-ignore state_referenced_locally
	const review = new Review(jobId, sourceId);

	/** The page of a scan shown. */
	let page = $state(0);
	let hovered = $state<string | null>(null);
	/** The answers being corrected. */
	let editing = $state<{ setupId: string; draft: Draft } | null>(null);

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

	// A PDF or image is shown page by page; a text document as text.
	$effect(() => review.showPage(page));

	function step(delta: number) {
		page = Math.min(Math.max(page + delta, 0), pages - 1);
	}

	function startEditing(setup: Setup) {
		editing = { setupId: setup.id, draft: toDraft(fields, answersOf(viewOf(setup))) };
	}

	async function save(setup: Setup, answers: FieldAnswer[] | null, reviewed: boolean) {
		if (await review.save({ setup, index: 0, fields: answers, reviewed })) editing = null;
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

{#snippet foot()}
	<span class="mono"
		>{lead?.model_version ?? setups[0]?.model ?? ''}{lead?.cost_usd
			? ` · ${formatUsd(lead.cost_usd)}`
			: ''}</span
	>
{/snippet}

<ReviewFrame
	{jobId}
	jobName={review.job?.name ?? null}
	fileName={source?.file_name ?? ''}
	error={review.error}
	{actions}
	reviewed={lead && lead.status !== 'failed' && !editing ? lead.reviewed : null}
	onreviewed={toggleReviewed}
	onstep={pages > 1 ? step : undefined}
	source={source?.kind === 'text' ? 'text' : 'scan'}
	piece={pages > 1 ? m.review.page(String(page + 1)) : null}
	paging={{ at: page, count: pages }}
	preview={review.preview}
	text={review.text}
	find={hovered ?? ''}
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
		{#each fields as f (f.key)}
			{@const readings = collate(f.key, witnesses)}
			<div class="answer">
				<h2>{f.label}</h2>
				{#if readings.length === 0}
					<p class="muted none">{m.review.noneFound}</p>
				{:else if f.kind.type === 'list' || f.kind.type === 'choices'}
					<p class="items">
						{#each readings as r, k (r.value)}{#if k > 0}<span class="sep">{'; '}</span>{/if}<span
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
</style>
