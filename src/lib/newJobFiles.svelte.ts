// New job's files: those added (or, for videos, their links), what the last ones added gave
// that the task does not read, several images read as the pages of one document in the order
// the researcher leaves (#52), and the job's name, which follows the files until one of the
// researcher's own is typed.

import type { SetAside } from './components/SetAsideNotice.svelte';
import type { ImageDocument } from './bindings/ImageDocument.js';
import type { InspectedFile } from './bindings/InspectedFile.js';
import type { Task } from './bindings/Task.js';
import type { TaskRules } from './bindings/TaskRules.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import {
	type Mode,
	type NewJobForm,
	nameOfFile,
	readersOf,
	suggestName,
	suggestionFits
} from './newJob.js';
import {
	type Grouping,
	imageSources,
	jobSources,
	offersOneDocument,
	sameImages
} from './pageOrder.js';
import { app } from './state.svelte.js';
import { api } from './tauri.js';

/** What New job gives its files: the task and its rules, whether the files are text frozen from
 * results, and where to say what went wrong (null clears it). */
export interface FilesSide {
	mode: () => Mode;
	task: () => Task;
	rule: () => TaskRules;
	frozen: boolean;
	error: (message: string | null) => void;
}

export class JobFiles {
	readonly #side: FilesSide;
	readonly #m = messages();
	list = $state<InspectedFile[]>([]);
	inspecting = $state(false);
	/** What the last files added gave that the task does not read, and where else it goes. */
	setAside = $state<SetAside | null>(null);
	/** Several images: each a document of its own, or the pages of one document (#52). */
	grouping = $state<Grouping>('separate');
	/** The document the engine made of the images, in the order the researcher left. */
	pageDoc = $state<ImageDocument | null>(null);
	docName = $state('');
	/** A page was moved by hand since the engine last put the pages in order. */
	byHand = $state(false);
	sorting = $state(false);
	name = $state('');
	/** The name Kalamu suggested, as long as the field still holds it. */
	suggested = $state('');
	/** The name the files suggest, kept while the document's name stands in for it. */
	fromFiles = $state('');
	readonly images = $derived(imageSources(this.list));
	readonly oneOffered = $derived.by(() => offersOneDocument(this.#side.rule(), this.images));
	/** The job's sources: the files, or the images as one document. */
	readonly sources = $derived(
		jobSources(this.list, this.oneOffered ? this.grouping : 'separate', this.pageDoc, this.docName)
	);

	/** Called while New job is set up, so its effects end with it. */
	constructor(side: FilesSide, kept: NewJobForm | null) {
		this.#side = side;
		this.list = kept?.files ?? [];
		this.name = kept?.name ?? '';
		this.suggested = kept?.suggested ?? '';
		this.fromFiles = kept?.fromFiles ?? kept?.suggested ?? '';
		this.grouping = kept?.grouping ?? 'separate';
		this.pageDoc = kept?.pageDoc ?? null;
		this.docName = kept?.docName ?? '';
		this.byHand = kept?.byHand ?? false;

		// A suggested name follows the files: once the file or folder it came from is removed, the
		// first file left names the job, and no file leaves the field empty again. Images read as
		// the pages of one document name the job after the document.
		$effect(() => {
			if (!this.suggested || this.name !== this.suggested) return;
			if (!suggestionFits(this.fromFiles, this.list, side.frozen)) {
				this.fromFiles = this.list[0] ? nameOfFile(this.list[0], side.frozen) : '';
			}
			const next =
				(this.oneOffered && this.grouping === 'one' && this.docName.trim()) || this.fromFiles;
			if (next === this.suggested) return;
			this.name = next;
			this.suggested = next;
		});
		$effect(() => {
			if (this.oneOffered && this.grouping === 'one' && !sameImages(this.pageDoc, this.images))
				void this.orderPages();
		});
	}

	/** Fills the name from the files, unless the researcher typed one of their own. */
	suggest(value: string) {
		this.fromFiles = value;
		if (this.name && this.name !== this.suggested) return;
		this.name = value;
		this.suggested = value;
	}

	/** Asks the engine for the document of these images, in the natural order of their names:
	 * when they are first read as one, when the images change, or to undo the researcher's order. */
	async orderPages(again = false) {
		const wanted = $state.snapshot(this.images);
		if (!again && sameImages(this.pageDoc, wanted)) return;
		this.sorting = true;
		try {
			const made = await api.imageDocument(this.docName, wanted);
			// Images changed while the engine sorted: the newer call settles it.
			if (!sameImages(made, imageSources(this.list))) return;
			this.pageDoc = made;
			this.byHand = false;
			if (!this.docName.trim()) this.docName = made.source.file_name;
		} catch (e) {
			this.#side.error(describeError(e, this.#m));
		} finally {
			this.sorting = false;
		}
	}

	async add(paths: string[]) {
		if (paths.length === 0) return;
		this.#side.error(null);
		this.setAside = null;
		try {
			const expanded = await api.expandPaths(paths, this.#side.task());
			// Videos are added by their link: recordings and videos dropped here are offered to
			// Transcribe audio, which reads them.
			if (this.#side.mode() === 'youtube') {
				const move = expanded.paths;
				this.setAside = {
					refused: [],
					readers: move.length > 0 ? ['audio_transcription'] : [],
					move,
					youtube: true
				};
				return;
			}
			if (expanded.refused.length > 0) {
				this.setAside = {
					refused: expanded.refused,
					readers: readersOf(app.rules, expanded.refused, this.#side.mode()),
					move: expanded.refused.filter((r) => r.kind).map((r) => r.path),
					youtube: false
				};
			}
			if (expanded.paths.length === 0) return;
			this.inspecting = true;
			const known = new Set(this.list.map((f) => f.path));
			const inspected = await api.inspectFiles(expanded.paths.filter((p) => !known.has(p)));
			this.list = [...this.list, ...inspected];
			this.suggest(suggestName(paths, expanded.paths));
		} catch (e) {
			this.#side.error(describeError(e, this.#m));
		} finally {
			this.inspecting = false;
		}
	}
}
