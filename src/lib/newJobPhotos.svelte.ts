// New job's photographs checked against each other (#216): the same page photographed twice,
// a blurred photograph. The engine does the work, off the interface's thread; this keeps what it
// found, asks again only when photographs are added (removing one or moving a page needs no new
// check: its warnings simply go), and remembers what the researcher chose to keep.

import type { InputWarning } from './bindings/InputWarning.js';
import type { PhotoChecks } from './bindings/PhotoChecks.js';
import type { PhotoView } from './bindings/PhotoView.js';
import type { SourceFile } from './bindings/SourceFile.js';
import { api } from './tauri.js';

/** A warning of the photograph check: the same page twice, or a blurred photograph. */
export type PhotoWarning = Extract<InputWarning, { kind: 'same_page' } | { kind: 'blurred' }>;

/** What identifies a warning, to remember that the researcher kept both photographs or the
 * blurred one. */
export function photoKey(w: PhotoWarning): string {
	return w.kind === 'same_page' ? `same:${w.other}:${w.source_id}` : `blurred:${w.source_id}`;
}

/** Waits this long after the photographs change before checking, so a folder added in several
 * steps is checked once. */
const SETTLE_MS = 400;

export class JobPhotos {
	/** What the last check found. */
	found = $state<PhotoChecks | null>(null);
	checking = $state(false);
	/** Warnings the researcher answered by keeping the photographs. */
	kept = $state<string[]>([]);
	/** The photographs the last check started was asked about. */
	#asked = new Set<string>();
	#run = 0;

	/** `photos` gives the photographs in the order they are read: the files', or the pages' of
	 * the document they make. Called while New job is set up, so its effect ends with it. */
	constructor(photos: () => SourceFile[]) {
		$effect(() => {
			const list = photos();
			if (list.length === 0) {
				this.found = null;
				this.#asked = new Set();
				return;
			}
			if (list.every((s) => this.#asked.has(s.id))) return;
			const sources = $state.snapshot(list) as SourceFile[];
			const timer = setTimeout(() => void this.#check(sources), SETTLE_MS);
			return () => clearTimeout(timer);
		});
	}

	async #check(sources: SourceFile[]) {
		const run = ++this.#run;
		this.#asked = new Set(sources.map((s) => s.id));
		this.checking = true;
		try {
			const found = await api.checkPhotos(sources);
			// Stopped by a newer check, or one has started since: that one settles it.
			if (found && run === this.#run) this.found = found;
		} catch {
			// A check that cannot run says nothing: the photographs are sent as they are.
		} finally {
			if (run === this.#run) this.checking = false;
		}
	}

	/** Stops a check still running, when New job is left. */
	stop() {
		this.#run++;
		this.checking = false;
		void api.stopPhotoChecks().catch(() => undefined);
	}

	/** The warnings about the photograph `sourceId` still to answer: both photographs of a pair
	 * still in the job's list (`present`), and not kept by the researcher. */
	warningsFor(sourceId: string, present: Set<string>): PhotoWarning[] {
		return (this.found?.warnings ?? []).filter(
			(w): w is PhotoWarning =>
				(w.kind === 'same_page' || w.kind === 'blurred') &&
				w.source_id === sourceId &&
				(w.kind !== 'same_page' || present.has(w.other)) &&
				!this.kept.includes(photoKey(w as PhotoWarning))
		);
	}

	view(sourceId: string): PhotoView | undefined {
		return this.found?.views.find((v) => v.source_id === sourceId);
	}

	/** The researcher keeps what a warning is about: both photographs, or the blurred one. */
	keep(w: PhotoWarning) {
		this.kept = [...this.kept, photoKey(w)];
	}
}
