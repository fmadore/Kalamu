<script lang="ts">
	// The speakers of a recording, as an edition lists who speaks: each model label with the name
	// the researcher gives it. Names are a layer over the model's output, applied here and in
	// every file; the output keeps its own labels.
	import type { PieceView } from '#lib/bindings/PieceView.js';
	import type { Setup } from '#lib/bindings/Setup.js';
	import { siglum } from '#lib/apparatus.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		sourceId,
		setups,
		views,
		manyPieces,
		onsaved
	}: {
		jobId: string;
		sourceId: string;
		setups: Setup[];
		/** The piece under review, one view per set-up. */
		views: PieceView[];
		/** A recording sent in several pieces, where a label may change person between pieces. */
		manyPieces: boolean;
		onsaved: () => void | Promise<void>;
	} = $props();

	const m = messages();
	let drafts = $state<Record<string, Record<string, string>>>({});
	let error = $state<string | null>(null);
	let saving = $state<string | null>(null);

	/** The set-ups that labelled speakers in this piece, with their siglum. */
	const named = $derived(
		setups.flatMap((s, i) => {
			const v = views.find((x) => x.setup_id === s.id);
			const labels = v ? [...new Set([...v.speakers, ...Object.keys(v.speaker_names)])] : [];
			return v && labels.length > 0 ? [{ setup: s, siglum: siglum(i), view: v, labels }] : [];
		})
	);

	// Start each set-up's fields from the names already saved.
	$effect(() => {
		for (const { setup, view } of named) {
			drafts[setup.id] ??= { ...view.speaker_names };
		}
	});

	async function save(setup: Setup, view: PieceView) {
		error = null;
		saving = setup.id;
		try {
			const names = { ...view.speaker_names, ...drafts[setup.id] };
			await api.saveSpeakerNames(jobId, { setup_id: setup.id, source_id: sourceId, names });
			await onsaved();
			toast(m.review.namesSaved);
		} catch (e) {
			error = describeError(e, m);
		} finally {
			saving = null;
		}
	}
</script>

{#if named.length > 0}
	<section class="speakers" aria-labelledby="speakers-title">
		<h2 id="speakers-title" class="smcp">{m.review.speakers}</h2>
		<p class="muted hint">{m.review.speakersHint}</p>
		{#each named as { setup, siglum: s, view, labels } (setup.id)}
			<form
				class="witness"
				onsubmit={(e) => {
					e.preventDefault();
					void save(setup, view);
				}}
			>
				{#if named.length > 1}<i class="siglum">{s}</i>{/if}
				{#each labels as label (label)}
					<label class="row">
						<span class="label-name">{label}</span>
						<input
							type="text"
							placeholder={m.review.speakerPlaceholder}
							aria-label={m.review.speakerName(label)}
							value={drafts[setup.id]?.[label] ?? ''}
							oninput={(e) => {
								drafts[setup.id] = { ...drafts[setup.id], [label]: e.currentTarget.value };
							}}
						/>
					</label>
				{/each}
				<div class="actions">
					<button class="btn sm" type="submit" disabled={saving === setup.id}
						>{m.review.saveNames}</button
					>
				</div>
			</form>
		{/each}
		{#if manyPieces}<p class="muted hint">{m.review.speakersPieces}</p>{/if}
		{#if error}<p class="error" role="alert">{error}</p>{/if}
	</section>
{/if}

<style>
	.speakers {
		margin-top: var(--space-6);
		padding-top: var(--space-3);
		border-top: 1px solid var(--line);
		max-width: 44rem;
	}
	.speakers h2 {
		font-size: 1.0625rem;
		color: var(--text-muted);
		margin-bottom: var(--space-1);
	}
	.hint {
		font-size: var(--type-sm);
		margin-bottom: var(--space-3);
	}
	.witness {
		position: relative;
		display: grid;
		gap: var(--space-2);
		margin-bottom: var(--space-4);
	}
	.witness .siglum {
		position: absolute;
		left: -3rem;
		width: 2.5rem;
		text-align: right;
	}
	.row {
		display: grid;
		grid-template-columns: minmax(7rem, 12rem) minmax(0, 18rem);
		gap: var(--space-3);
		align-items: center;
	}
	.label-name {
		font-weight: 600;
		color: var(--text-secondary);
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
	}
	.error {
		color: var(--danger);
	}
	@media (max-width: 820px) {
		.row {
			grid-template-columns: 1fr;
			gap: var(--space-1);
		}
	}
</style>
