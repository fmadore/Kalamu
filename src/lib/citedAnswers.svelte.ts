// The answers of an analysis that quotes its text, as its review changes them and plays them:
// the text the analysis read, for showing a quotation where it stands; where a quotation's
// recording is (the transcript the text was frozen from, played at the quotation's turn); and
// the researcher's changes (an entry's fields typed anew, an entry or one quotation removed),
// each one command kept in the corrected answers, and each taken back from its toast by putting
// the set-up's correction back as it stood, which the engine accepts since the quotations it
// holds are the model's. The focus, lost with what was removed, goes where the researcher can
// carry on, as it does for the names of a list (`listedNames.svelte.ts`).

import { tick } from 'svelte';
import type { CitedEdit } from './bindings/CitedEdit.js';
import type { CitedEvidence } from './bindings/CitedEvidence.js';
import type { CitedSource } from './bindings/CitedSource.js';
import type { Correction } from './bindings/Correction.js';
import type { DerivedState } from './bindings/DerivedState.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Setup } from './bindings/Setup.js';
import { utf16Span } from './cited.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import type { Review } from './review.svelte.js';
import { app, go, toast, type View } from './state.svelte.js';
import { api } from './tauri.js';
import { citedApi } from './tauriCited.js';

/** Whether any field of these set-ups quotes the text. */
export function quotesText(setups: Setup[]): boolean {
	return setups.some((s) => s.prompt.schema?.fields.some((f) => f.kind.type === 'cited'));
}

/** A piece's correction as it stands, to put back: its corrected answers (none for the model's),
 * its text and whether it is reviewed. */
export function standing(review: Review, v: PieceView): Correction {
	return {
		job_id: review.jobId,
		setup_id: v.setup_id,
		source_id: review.sourceId,
		index: v.index,
		text: v.corrected_text,
		fields: v.corrected_fields ?? undefined,
		reviewed: v.reviewed,
		updated_at: 0
	};
}

/** The review a quotation's recording plays in: the transcript the analysed text was frozen
 * from, at the quotation's turn. None when the quotation has no time, the text was not frozen
 * from a transcription, or that transcript is gone. */
export function recordingOf(
	e: CitedEvidence,
	review: Pick<Review, 'source'>,
	derived: DerivedState[],
	taskOf: (jobId: string) => string | undefined
): View | null {
	const from = review.source?.derived_from;
	if (e.start_ms === null || !from) return null;
	if (derived.find((d) => d.source_id === review.source?.id)?.freshness === 'gone') return null;
	if (taskOf(from.job_id) !== 'audio_transcription') return null;
	return { name: 'review', jobId: from.job_id, sourceId: from.source_id, at: e.start_ms };
}

/** Where the focus goes once the answers are read again: an entry's quotation, an entry, or the
 * field itself. */
type FocusAt = { entry: number; quote?: number } | null;

/** An entry's fields being typed anew. */
export interface EntryEdit {
	setupId: string;
	key: string;
	entry: number;
	draft: Record<string, string>;
	before: Record<string, string>;
}

export class CitedAnswers {
	readonly #review: Review;
	readonly #pane: () => HTMLElement | null;
	/** The text the analysis read, with its turns; none until read, or for an analysis that
	 * quotes nothing. */
	source = $state<CitedSource | null>(null);
	/** How the frozen text stands against the transcript it came from. */
	derived = $state<DerivedState[]>([]);
	/** A quotation shown where it stands in the text, in UTF-16 units. */
	span = $state<{ start: number; end: number } | null>(null);
	/** The entry whose fields are being typed anew. */
	editing = $state<EntryEdit | null>(null);
	busy = $state(false);

	constructor(review: Review, pane: () => HTMLElement | null) {
		this.#review = review;
		this.#pane = pane;
	}

	/** Reads the text and where it came from, once the job is read; nothing for an analysis
	 * that quotes nothing. */
	async load(): Promise<void> {
		const review = this.#review;
		if (!quotesText(review.setups)) return;
		[this.source, this.derived] = await Promise.all([
			citedApi.citedSource(review.jobId, review.sourceId).catch(() => null),
			api.derivedState(review.jobId).catch(() => [])
		]);
	}

	/** Where the quotation's recording plays, if it can. */
	recording(e: CitedEvidence): View | null {
		return recordingOf(
			e,
			this.#review,
			this.derived,
			(id) => app.jobs.find((j) => j.id === id)?.task
		);
	}

	/** Plays the quotation in its transcript's review, or, without a recording, shows it where
	 * it stands in the text beside the answers. */
	open(e: CitedEvidence): void {
		const to = this.recording(e);
		if (to) go(to);
		else if (this.source) this.span = utf16Span(this.source.text, e.anchor);
	}

	/**
	 * Makes one change, or several to one entry, to field `key` of a set-up's answers, and says
	 * so with a way to take it back; the focus then goes to `after`, and to `back` once taken
	 * back. False when the engine refused, the reason shown.
	 */
	async change(
		setup: Setup,
		key: string,
		changes: CitedEdit[],
		said: string,
		after: FocusAt,
		back: FocusAt
	): Promise<boolean> {
		const review = this.#review;
		const view = review.views.find((v) => v.setup_id === setup.id && v.index === 0);
		if (!view || changes.length === 0 || this.busy) return false;
		const before = standing(review, view);
		// The focus was on what changes (a button of the card, say, which the card's next
		// quotation or entry may now hold): it moves on in any case, as if it had been lost.
		const within = this.#field(setup.id, key)?.contains(document.activeElement) ?? false;
		this.busy = true;
		review.error = null;
		try {
			for (const change of changes) {
				await citedApi.editCited(review.jobId, setup.id, review.sourceId, 0, key, change);
			}
		} catch (e) {
			review.error = describeError(e, messages());
			return false;
		} finally {
			this.busy = false;
			await review.refresh().catch(() => undefined);
		}
		this.span = null;
		toast(said, {
			action: { label: messages().common.undo, run: () => void this.#putBack(before, key, back) }
		});
		await this.focus(setup.id, key, after, within);
		return true;
	}

	/** Puts the set-up's correction back as it stood, and the focus on what came back. */
	async #putBack(before: Correction, key: string, at: FocusAt) {
		if (!(await this.#review.putBack([before]))) return;
		await this.focus(before.setup_id, key, at, true);
	}

	/** Takes the focus to an entry's quotation, an entry, or the field, when the focus was lost
	 * with what it was on (or always, after a toast's button). */
	async focus(setupId: string, key: string, at: FocusAt, always = false): Promise<void> {
		await tick();
		const lost = !document.activeElement || document.activeElement === document.body;
		if (!always && !lost) return;
		const field = this.#field(setupId, key);
		const entry = at && field?.querySelector<HTMLElement>(`[data-entry="${at.entry}"]`);
		const target =
			(entry &&
				at?.quote !== undefined &&
				entry.querySelector<HTMLElement>(`[data-quote="${at.quote}"] [data-act="open"]`)) ||
			entry?.querySelector<HTMLElement>('[data-act="edit"]') ||
			field;
		target?.focus();
	}

	/** One set-up's cards for field `key`. */
	#field(setupId: string, key: string): HTMLElement | null {
		return (
			this.#pane()?.querySelector<HTMLElement>(
				`[data-cited="${CSS.escape(`${setupId}/${key}`)}"]`
			) ?? null
		);
	}
}
