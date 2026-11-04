// The review's search: words found across every piece of the text shown, corrections
// included, and going from one match to the next (a recording plays from the match's line).
// Opened from a project's search, it searches the words searched there and goes to the match
// chosen; a match in another set-up's text opens that text beside the text shown.

import type { PieceView } from './bindings/PieceView.js';
import { shownText } from './compare.svelte.js';
import type { ListedMatch } from './components/SearchBar.svelte';
import { messages, type Messages } from './i18n/index.js';
import { clock } from './pieces.js';
import { context, search, searchable, type Hit, type Searched } from './search.js';
import { toast, type ReviewFind } from './state.svelte.js';

/** Each piece as it is searched: the first set-up's text as shown, corrections included. */
export function searchedPieces(
	indices: number[],
	views: PieceView[],
	lead: string | undefined
): Searched[] {
	return indices.map((i) => {
		const v = views.find((x) => x.index === i && x.setup_id === lead);
		return {
			index: i,
			text: shownText(v),
			startMs: v?.piece.kind === 'segment' ? Number(v.piece.start_ms) : 0,
			segments: v?.segments ?? []
		};
	});
}

/** What the search bar says: nothing until the words can be searched, then how many matches
 * and which one is shown. */
export function searchSummary(query: string, hits: number, at: number, m: Messages): string {
	if (!searchable(query)) return '';
	if (hits === 0) return m.review.notFound;
	return at >= 0 ? m.review.foundAt(at + 1, hits) : m.review.found(hits);
}

/** What the review is: its pieces and set-ups, the piece shown, and how to move. */
export interface Reviewed {
	views(): PieceView[];
	/** The first set-up's id: its text is the one searched. */
	lead(): string | undefined;
	indices(): number[];
	/** The piece shown. */
	index(): number;
	recording(): boolean;
	/** A piece's name in the list of matches. */
	labelOf(v: PieceView | undefined): string;
	goTo(index: number): void;
	/** Opens another set-up's text beside the text shown. */
	compare(setupId: string): void;
}

export class ReviewSearch {
	/** Whether the search bar is open. */
	searching = $state(false);
	query = $state('');
	/** The match gone to, among all of them; -1 before any. */
	at = $state(-1);
	/** Where a recording should play from, after going to a match. */
	seek = $state<{ ms: number; id: number } | null>(null);
	/** Set when going to a match, until its words are in view. */
	revealing = false;
	readonly #review: Reviewed;
	/** A match chosen in a project's search, until it is gone to. */
	#pending = $state<ReviewFind | null>(null);

	readonly searched = $derived.by(() =>
		searchedPieces(this.#review.indices(), this.#review.views(), this.#review.lead())
	);
	readonly hits: Hit[] = $derived.by(() => search(this.searched, this.query));
	readonly listed: ListedMatch[] = $derived.by(() =>
		this.hits.map((h) => {
			const piece = this.searched.find((p) => p.index === h.index);
			return {
				group: this.#review.labelOf(this.#review.views().find((v) => v.index === h.index)),
				time:
					this.#review.recording() && h.lineMs !== null
						? clock((piece?.startMs ?? 0) + h.lineMs)
						: null,
				...context(piece?.text ?? '', h)
			};
		})
	);
	readonly summary = $derived.by(() =>
		searchSummary(this.query, this.hits.length, this.at, messages())
	);
	/** Which match on the piece shown is the one gone to. */
	readonly here = $derived.by(() =>
		this.hits[this.at]?.index === this.#review.index() ? this.hits[this.at].nth : -1
	);

	constructor(review: Reviewed, opening?: ReviewFind) {
		this.#review = review;
		this.#pending = opening ?? null;
		// Closing the search forgets the words; new words start from the first match.
		$effect(() => {
			if (!this.searching) this.query = '';
		});
		$effect(() => {
			void this.query;
			this.at = -1;
		});
		$effect(() => this.#open());
	}

	/** Goes to match `i`: its piece, and in a recording its line. */
	goToHit = (i: number): void => {
		const h = this.hits[i];
		if (!h) return;
		this.at = i;
		this.revealing = true;
		if (h.index !== this.#review.index()) this.#review.goTo(h.index);
		if (this.#review.recording() && h.lineMs !== null) {
			this.seek = { ms: h.lineMs, id: (this.seek?.id ?? 0) + 1 };
		}
	};

	/** Opened from a project's search: the words searched, and the match chosen gone to. In
	 * another set-up's text, its page is opened with that text compared beside it. */
	#open(): void {
		const want = this.#pending;
		const review = this.#review;
		if (!want || review.views().length === 0 || review.lead() === undefined) return;
		if (!this.searching) {
			this.searching = true;
			this.query = want.query;
			return;
		}
		this.#pending = null;
		if (want.setupId !== review.lead()) {
			review.goTo(want.index);
			review.compare(want.setupId);
			return;
		}
		const at = this.hits.findIndex((h) => h.index === want.index && h.nth === want.nth);
		if (at >= 0) {
			this.goToHit(at);
		} else {
			review.goTo(want.index);
			toast(messages().search.moved);
		}
	}
}
