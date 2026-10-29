// What the three reviews share: the job and the source's pieces, loaded and saved; the page of
// a scan shown on the lightbox; and where the boundaries between the panes sit.

import type { ComparisonReport } from './bindings/ComparisonReport.js';
import type { FieldAnswer } from './bindings/FieldAnswer.js';
import type { Job } from './bindings/Job.js';
import type { PagePreview } from './bindings/PagePreview.js';
import type { PieceView } from './bindings/PieceView.js';
import type { Setup } from './bindings/Setup.js';
import { describeError } from './errors.js';
import { messages } from './i18n/index.js';
import { toast } from './state.svelte.js';
import { keep, recall } from './stored.js';
import { api } from './tauri.js';

/** One source of a job under review. */
export class Review {
	readonly jobId: string;
	readonly sourceId: string;
	job = $state<Job | null>(null);
	views = $state<PieceView[]>([]);
	/** The whole text of a text file, shown as the source. */
	text = $state<string | null>(null);
	/** The set-ups' error rates on this source, for reviews that give them. */
	report = $state<ComparisonReport | null>(null);
	/** The page of a scan on the lightbox: null while it loads. */
	preview = $state<PagePreview | null>(null);
	error = $state<string | null>(null);
	readonly source = $derived(this.job?.sources.find((s) => s.id === this.sourceId));
	readonly setups = $derived(this.job?.setups ?? []);
	/** A scan or images, shown page by page. */
	readonly paged = $derived(this.source?.kind === 'pdf' || this.source?.kind === 'image');
	readonly #scored: boolean;
	#page = -1;

	constructor(jobId: string, sourceId: string, { scored = false } = {}) {
		this.jobId = jobId;
		this.sourceId = sourceId;
		this.#scored = scored;
	}

	async load() {
		try {
			this.job = await api.getJob(this.jobId);
			await this.refresh();
			if (this.source?.kind === 'text') {
				this.text = await api.sourceText(this.jobId, this.sourceId);
			}
		} catch (e) {
			this.error = describeError(e, messages());
		}
	}

	/** Reads the pieces again, and the error rates they give. */
	async refresh() {
		this.views = await api.sourcePieces(this.jobId, this.sourceId);
		if (this.#scored) this.report = await api.compareSource(this.jobId, this.sourceId);
	}

	/** Shows page `index` of a scan; only the last page asked for is shown. */
	showPage(index: number) {
		this.#page = index;
		this.preview = null;
		if (!this.paged) return;
		api
			.piecePreview(this.jobId, this.sourceId, index)
			.then((p) => {
				if (this.#page === index) this.preview = p;
			})
			.catch(() => {
				if (this.#page === index) this.preview = { mime: 'none', data: '' };
			});
	}

	/** Stores a correction, or only whether a piece is reviewed, and reads the pieces again (the
	 * result files are written again too). False when it failed, the error shown. */
	async save(c: {
		setup: Setup;
		index: number;
		text?: string | null;
		fields?: FieldAnswer[] | null;
		reviewed: boolean;
	}): Promise<boolean> {
		this.error = null;
		try {
			await api.saveCorrection({
				job_id: this.jobId,
				setup_id: c.setup.id,
				source_id: this.sourceId,
				index: c.index,
				text: c.text ?? null,
				fields: c.fields ?? undefined,
				reviewed: c.reviewed,
				updated_at: 0
			});
			await this.refresh();
			toast(messages().review.saved);
			return true;
		} catch (e) {
			this.error = describeError(e, messages());
			return false;
		}
	}
}

/** Where each boundary between panes sits, as shares of the width, the panes left of `i`
 * and right of it each kept at least `min` wide. */
export function moveCut(cuts: number[], i: number, at: number, min: number): number[] {
	const lo = (cuts[i - 1] ?? 0) + min;
	const hi = (cuts[i + 1] ?? 1) - min;
	return cuts.map((c, j) => (j === i ? Math.min(Math.max(at, lo), hi) : c));
}

/** The grid columns for panes divided at `cuts`. */
export function columns(cuts: number[]): string {
	return [...cuts, 1].map((c, i) => `minmax(0, ${c - (cuts[i - 1] ?? 0)}fr)`).join(' ');
}

/** The narrowest share a pane may take: about 16rem, or less in a narrow window. */
export function minShare(width: number): number {
	return width > 0 ? Math.min(0.3, 260 / width) : 0.2;
}

const LAYOUT_KEY = 'kalamu.review.layout';
const TWO = 0.475;
const THREE: [number, number] = [0.33, 0.665];

type Stored = { two: number; three: [number, number]; compare: boolean };

function readLayout(s: unknown): Stored | null {
	const v = s as Partial<Stored> | null;
	if (typeof v?.two !== 'number' || !Array.isArray(v.three) || v.three.length !== 2) return null;
	return {
		two: v.two,
		three: [Number(v.three[0]), Number(v.three[1])],
		compare: v.compare === true
	};
}

/** The panes' boundaries, the same in every review: one beside the text, two while a compared
 * text is open beside it. Remembered on this computer, with whether that text is open. */
class Layout {
	two = $state(TWO);
	three = $state<[number, number]>([...THREE]);
	compare = $state(false);

	constructor() {
		const s = recall(LAYOUT_KEY, readLayout, null);
		if (s) {
			this.two = s.two;
			this.three = s.three;
			this.compare = s.compare;
		}
	}

	cuts(third: boolean): number[] {
		return third ? this.three : [this.two];
	}

	/** Moves boundary `i` to `at` in a review `width` pixels wide. */
	move(i: number, at: number, width: number, third: boolean) {
		const cuts = moveCut(this.cuts(third), i, at, minShare(width));
		if (third) this.three = [cuts[0], cuts[1]];
		else this.two = cuts[0];
		this.#keep();
	}

	/** Puts boundary `i` back where it starts. */
	reset(i: number, width: number, third: boolean) {
		this.move(i, third ? THREE[i] : TWO, width, third);
	}

	toggleCompare() {
		this.compare = !this.compare;
		this.#keep();
	}

	#keep() {
		keep(LAYOUT_KEY, { two: this.two, three: this.three, compare: this.compare });
	}
}

export const layout = new Layout();
