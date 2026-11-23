<script lang="ts">
	// One line of what to check: the document, place and set-up, each reason in words, and the
	// researcher's decisions on it (reviewed, kept aside with a note, drawn at random). The line
	// opens the review at its page; beside it, the item is kept aside with a note or put back,
	// which changes no answer and resolves nothing.
	import Icon from './Icon.svelte';
	import type { QueueItem } from '#lib/bindings/QueueItem.js';
	import type { Signal } from '#lib/bindings/Signal.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
	import { isKept, placeOf, reasonText } from '#lib/queue.js';

	let {
		item,
		onopen,
		onkeep
	}: {
		item: QueueItem;
		onopen: (item: QueueItem) => void;
		/** Keeps the item aside with a note, or with `null` puts it back. */
		onkeep: (item: QueueItem, note: string | null) => Promise<void>;
	} = $props();
	const m = messages();
	const q = m.queue;

	/** The note being written, while the item is being kept aside. */
	let draft = $state<string | null>(null);
	let busy = $state(false);
	const kept = $derived(isKept(item));

	const tone = (s: Signal) =>
		s.kind === 'failed'
			? 'danger'
			: s.kind === 'not_reviewed' || s.kind === 'not_read'
				? ''
				: s.kind === 'disagrees'
					? 'hand'
					: 'warn';

	async function keep(note: string | null) {
		busy = true;
		try {
			await onkeep(item, note);
			draft = null;
		} finally {
			busy = false;
		}
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		if (draft !== null) void keep(draft);
	}
</script>

<li class="row">
	<button
		class="item"
		class:resolved={!item.open}
		onclick={() => onopen(item)}
		aria-label={q.open(item.document, placeOf(item, m))}
	>
		<span class="body">
			<span class="line">
				<strong>{item.document}</strong>
				<span>{placeOf(item, m)}</span>
				<span class="muted">{item.setup_label} · {item.job_name}</span>
				{#if item.preferred}<span class="chip ok">{q.preferredBadge}</span>{/if}
			</span>
			<span class="line">
				{#if item.sample}<span class="chip sample">{q.sample.badge}</span>{/if}
				{#each item.signals as s, i (i)}
					<span class="chip {tone(s)}">{reasonText(s, m, item.piece === null)}</span>
				{/each}
				{#if item.reviewed}<span class="chip ok">{q.reviewedBadge}</span>{/if}
				{#if item.kept && !item.kept.current}
					<span class="chip muted" title={q.keep.earlierHint(item.kept.note)}>
						{q.keep.earlier}
					</span>
				{/if}
				<!-- When the answer was stored; its fingerprint tells no two rows apart, a piece
				     having one answer at a time. -->
				{#if item.answered_at}
					<span class="revision num muted" title={q.answeredHint}>
						{q.answered(formatDateTime(item.answered_at))}
					</span>
				{/if}
			</span>
			{#if kept && item.kept}
				<span class="line note">
					<span class="chip">{q.keep.badge}</span>
					{#if item.kept.note}<span>{item.kept.note}</span>{/if}
				</span>
			{/if}
		</span>
		<Icon name="chevron" size={14} />
	</button>
	<div class="actions">
		{#if draft !== null}
			<form class="keep" onsubmit={submit}>
				<!-- svelte-ignore a11y_autofocus -->
				<input
					type="text"
					bind:value={draft}
					placeholder={q.keep.notePlaceholder}
					aria-label={q.keep.note}
					maxlength="500"
					autofocus
				/>
				<button class="btn sm" type="submit" disabled={busy}>{q.keep.save}</button>
				<button class="btn ghost sm" type="button" onclick={() => (draft = null)}>
					{q.keep.cancel}
				</button>
			</form>
		{:else if kept}
			<button class="btn ghost sm" disabled={busy} onclick={() => keep(null)}>
				{q.keep.putBack}
			</button>
		{:else}
			<button
				class="btn ghost sm"
				aria-label={q.keep.label(item.document, placeOf(item, m))}
				onclick={() => (draft = item.kept?.note ?? '')}
			>
				{q.keep.action}
			</button>
		{/if}
	</div>
</li>

<style>
	.row {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		border-bottom: 1px solid var(--line);
	}
	.item {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		width: 100%;
		min-height: var(--control-md);
		padding: var(--space-2);
		border: 0;
		background: none;
		color: var(--text-secondary);
		font: inherit;
		font-size: var(--type-sm);
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
		color: var(--text-bright);
	}
	.item.resolved {
		opacity: 0.75;
	}
	.body {
		display: grid;
		gap: var(--space-1);
		min-width: 0;
	}
	.line {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-1) var(--space-2);
		min-width: 0;
	}
	.line strong {
		color: var(--text-bright);
		font-weight: 500;
		overflow-wrap: anywhere;
	}
	.line .chip {
		white-space: normal;
	}
	.note {
		color: var(--text-body);
		overflow-wrap: anywhere;
	}
	.chip.sample {
		border-style: dashed;
	}
	.revision {
		font-size: var(--type-xs);
	}
	.actions {
		padding: 0 var(--space-2);
	}
	.keep {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.keep input {
		width: 16rem;
		max-width: 100%;
	}
	@media (max-width: 40rem) {
		.row {
			grid-template-columns: minmax(0, 1fr);
		}
		.actions {
			padding: 0 var(--space-2) var(--space-2);
		}
	}
</style>
