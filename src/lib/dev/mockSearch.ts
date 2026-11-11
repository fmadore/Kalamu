// Project search for the browser preview (`?mock&view=project`, then Search): the sample
// bulletin's two readings and its analysis, and the sample interview, searched as the search
// crate searches them (`kalamu_search`), without its index: the query read as a phrase, every
// word or words near each other; the collection, tag, task and review filters; the preferred
// readings or all of them; and what models wrote about the documents when asked. Development
// builds only.

import type { DocumentHits } from '../bindings/DocumentHits.js';
import type { FieldAnswer } from '../bindings/FieldAnswer.js';
import type { FieldRef } from '../bindings/FieldRef.js';
import type { Hit } from '../bindings/Hit.js';
import type { Job } from '../bindings/Job.js';
import type { MatchMode } from '../bindings/MatchMode.js';
import type { PieceRef } from '../bindings/PieceRef.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { ResultHits } from '../bindings/ResultHits.js';
import type { SearchRequest } from '../bindings/SearchRequest.js';
import type { SearchView } from '../bindings/SearchView.js';
import type { Task } from '../bindings/Task.js';
import { MIN_QUERY } from '../projectSearch.js';
import { context, find, folded, type Match } from '../search.js';
import { analysisJob, analysisPieces, audioJob, audioPieces, job, pieces } from './mockJobs.js';
import { DOCUMENT_ID, documentsIn } from './mockProjects.js';
import taskRules from './taskRules.json';

const INTERVIEW_ID = '00000000-0000-0000-0000-0000000000d2';
/** The most matches listed, as `kalamu_search::MAX_LISTED`; all are counted. */
const MAX_LISTED = 1000;
/** Tasks in the order a document's results are listed, as `Task::ALL`. */
const TASKS: Task[] = [
	'text_extraction',
	'text_correction',
	'translation',
	'analysis',
	'contents',
	'audio_transcription'
];
const QUOTES = /["“”«»]/;
const PUNCTUATION = /^[!-/:-@[-`{-~]+|[!-/:-@[-`{-~]+$/g;

const yieldsText = (task: Task) => taskRules.find((r) => r.task === task)?.yields_text ?? false;

/** A word or words looked for: as typed, which the review searches for when a match opens. */
interface Term {
	typed: string;
	needle: string;
}

/** The terms of a query as `mode` reads it, as `terms::parse`: the whole query for a phrase,
 * else each word, words between quotes kept together, punctuation around a word set aside. A
 * term shorter than the shortest searched for is left out and named. */
export function parseQuery(query: string, mode: MatchMode): { terms: Term[]; ignored: string[] } {
	const written =
		mode === 'phrase'
			? [query.trim()]
			: query.split(QUOTES).flatMap((chunk, i) =>
					i % 2 === 1
						? [chunk.split(/\s+/).filter(Boolean).join(' ')]
						: chunk
								.split(/\s+/)
								.filter(Boolean)
								.map((w) => w.replace(PUNCTUATION, ''))
				);
	const out: { terms: Term[]; ignored: string[] } = { terms: [], ignored: [] };
	for (const typed of written.filter(Boolean)) {
		const needle = folded(typed);
		if ([...needle].length < MIN_QUERY) {
			if (!out.ignored.includes(typed)) out.ignored.push(typed);
		} else if (!out.terms.some((t) => t.needle === needle)) {
			out.terms.push({ typed, needle });
		}
	}
	return out;
}

/** One part of a piece searched apart from the others: its text, or one answer or reason. */
interface Part {
	text: string;
	/** A recording's timed lines: when each begins, from the piece's start, and its text. */
	lines: [number, string][];
	field: FieldRef | null;
	reason: boolean;
}

interface Passage {
	index: number;
	piece: PieceRef;
	startMs: number | null;
	reviewed: boolean;
	corrected: boolean;
	parts: Part[];
}

/** An analysis's answers as parts, each field's answer and its reason, as `answer_parts`. */
function answerParts(answers: FieldAnswer[], labels: Map<string, string>): Part[] {
	return answers.flatMap((a) => {
		const field = { key: a.key, label: labels.get(a.key) ?? a.key };
		const value = Array.isArray(a.value) ? a.value.join('; ') : String(a.value ?? '');
		return [
			...(value.trim() ? [{ text: value, lines: [], field, reason: false }] : []),
			...(a.reason?.trim() ? [{ text: a.reason, lines: [], field, reason: true }] : [])
		];
	});
}

function passage(of: Job, v: PieceView): Passage {
	const labels = new Map(
		(of.setups.find((s) => s.id === v.setup_id)?.prompt.schema?.fields ?? []).map((f) => [
			f.key,
			f.label
		])
	);
	const text = v.corrected_text ?? v.ai_text ?? '';
	return {
		index: v.index,
		piece: v.piece,
		startMs: v.piece.kind === 'segment' ? Number(v.piece.start_ms) : null,
		reviewed: v.reviewed,
		corrected: v.corrected_text !== null || v.corrected_fields !== null,
		parts: yieldsText(of.task)
			? [
					{
						text,
						lines: v.segments.map((s) => [Number(s.start_ms), s.text]),
						field: null,
						reason: false
					}
				]
			: answerParts((v.corrected_fields ?? v.ai_fields) as FieldAnswer[], labels)
	};
}

/** One place a term stands in a part. */
interface Place {
	term: number;
	/** Which match of its term it is in the piece, as the review counts them. */
	nth: number;
	span: Match;
	atMs: number | null;
}

/** For each character of `text`, the number of the word it is in, from 1. */
function wordNumbers(text: string): number[] {
	let n = 0;
	let space = true;
	return [...text].flatMap((c) => {
		const white = /\s/u.test(c);
		if (!white && space) n += 1;
		space = white;
		return c.length === 2 ? [n, n] : [n];
	});
}

/** Where every term stands with at most `near` words between the first and the last, as
 * `terms::clusters`. */
function clusters(text: string, places: Place[], terms: number, near: number): Place[][] {
	const words = wordNumbers(text);
	const word = (at: number) => words[at] ?? 0;
	let latest: (Place | undefined)[] = Array(terms).fill(undefined);
	const out: Place[][] = [];
	for (const p of places) {
		latest[p.term] = p;
		const chosen = latest.filter((q) => q !== undefined);
		if (chosen.length < terms) continue;
		const first = chosen.reduce((a, b) => (b.span.start < a.span.start ? b : a));
		const between = Math.max(0, word(p.span.start) - word(Math.max(0, first.span.end - 1)) - 1);
		if (between <= near) {
			out.push([...chosen].sort((a, b) => a.span.start - b.span.start || a.term - b.term));
			latest = Array(terms).fill(undefined);
		}
	}
	return out;
}

/** How many characters of the words shown stand before `at`, where a run of white space is one. */
function shown(text: string, start: number, at: number): number {
	let n = 0;
	let space = false;
	for (const c of text.slice(start, at)) {
		const white = /\s/u.test(c);
		if (!(white && space)) n += 1;
		space = white;
	}
	return n;
}

/** The matches of the terms in one piece, as `terms::hits`: a match never runs from one part
 * into the next, and every term's matches are counted through the whole piece. */
function passageHits(at: Passage, terms: Term[], mode: MatchMode, near: number): Hit[] {
	const out: { hit: Hit; order: [number, number] }[] = [];
	const nth = terms.map(() => 0);
	at.parts.forEach((part, p) => {
		const places: Place[] = [];
		let every = true;
		terms.forEach((term, t) => {
			const found = find(part.text, term.typed);
			every &&= found.length > 0;
			const lines = part.lines.flatMap(([ms, text]) => find(text, term.typed).map(() => ms));
			const timed = at.startMs !== null && lines.length > 0 && lines.length === found.length;
			found.forEach((span, i) =>
				places.push({
					term: t,
					nth: nth[t]++,
					span,
					atMs: timed ? (at.startMs ?? 0) + lines[i] : null
				})
			);
		});
		if (!every) return;
		places.sort((a, b) => a.span.start - b.span.start || a.term - b.term);
		const make = (first: Place, span: Match, marks: Match[]) => {
			const words = context(part.text, span);
			out.push({
				order: [p, span.start],
				hit: {
					index: at.index,
					piece: at.piece,
					term: terms[first.term].typed,
					nth: first.nth,
					field: part.field,
					entry: null,
					entry_number: null,
					reason: part.reason,
					...words,
					marks: marks.map((m) => ({
						start: shown(part.text, span.start, m.start),
						end: shown(part.text, span.start, m.end)
					})),
					at_ms: first.atMs,
					reviewed: at.reviewed,
					corrected: at.corrected
				}
			});
		};
		if (mode === 'near') {
			for (const cluster of clusters(part.text, places, terms.length, near)) {
				const end = Math.max(...cluster.map((q) => q.span.end));
				make(
					cluster[0],
					{ start: cluster[0].span.start, end },
					cluster.map((q) => q.span)
				);
			}
		} else {
			for (const q of places) make(q, q.span, [q.span]);
		}
	});
	return out.sort((a, b) => a.order[0] - b.order[0] || a.order[1] - b.order[1]).map((f) => f.hit);
}

/** A result a document may be searched in: a set-up of a job over its source. */
interface Candidate {
	of: Job;
	setup: number;
	views: PieceView[];
	preferred: boolean;
}

/** The sample project's results, by document: the bulletin's two readings (the second
 * preferred) and its analysis, and the interview. */
function candidates(documentId: string): Candidate[] {
	const each = (of: Job, views: PieceView[], preferred = -1) =>
		of.setups.map((setup, i) => ({
			of,
			setup: i,
			views: views.filter((v) => v.setup_id === setup.id),
			preferred: i === preferred
		}));
	if (documentId === DOCUMENT_ID)
		return [...each(job, pieces, 1), ...each(analysisJob, analysisPieces)];
	if (documentId === INTERVIEW_ID) return each(audioJob, audioPieces());
	return [];
}

function search(request: SearchRequest): SearchView {
	const query = parseQuery(request.query, request.mode);
	if (query.terms.length === 0) throw { id: 'search.too_short' };
	const inScope = documentsIn({ collection: request.collection, tag: request.tag });
	const wanted = (task: Task) =>
		(yieldsText(task) || request.generated) && (request.task === null || request.task === task);
	const coverage = {
		documents: inScope.length,
		searched: 0,
		unread: 0,
		pieces_unread: 0,
		without_preference: 0,
		pieces: 0
	};
	const documents: DocumentHits[] = [];
	for (const document of inScope) {
		const results: ResultHits[] = [];
		let read = false;
		for (const task of TASKS) {
			const all = candidates(document.id).filter((c) => c.of.task === task && wanted(task));
			const hasPreferred = all.some((c) => c.preferred);
			if (!hasPreferred && all.length > 1) coverage.without_preference += 1;
			const chosen =
				request.results === 'preferred' && hasPreferred ? all.filter((c) => c.preferred) : all;
			for (const c of chosen) {
				const done = c.views.filter((v) => v.status === 'done' || v.status === 'cached');
				coverage.pieces += done.length;
				coverage.pieces_unread += c.views.length - done.length;
				read ||= done.length > 0;
				const hits = done
					.filter((v) => request.review === 'any' || v.reviewed === (request.review === 'reviewed'))
					.sort((a, b) => a.index - b.index)
					.flatMap((v) => passageHits(passage(c.of, v), query.terms, request.mode, request.near));
				if (hits.length === 0) continue;
				const setup = c.of.setups[c.setup];
				results.push({
					job_id: c.of.id,
					job_name: c.of.name,
					setup_id: setup.id,
					setup_label: setup.label,
					source_id: c.of.sources[0].id,
					task,
					preferred: c.preferred,
					generated: !yieldsText(task),
					stale: false,
					hits
				});
			}
		}
		if (read) coverage.searched += 1;
		if (results.length > 0) {
			documents.push({ document_id: document.id, title: document.title, results });
		}
	}
	coverage.unread = coverage.documents - coverage.searched;
	documents.sort((a, b) => folded(a.title).localeCompare(folded(b.title)));
	// The first matches listed, all counted.
	let matches = 0;
	const listed = documents
		.map((d) => ({
			...d,
			results: d.results
				.map((r) => {
					const room = Math.max(0, MAX_LISTED - matches);
					matches += r.hits.length;
					return { ...r, hits: r.hits.slice(0, room) };
				})
				.filter((r) => r.hits.length > 0)
		}))
		.filter((d) => d.results.length > 0);
	return {
		documents: listed,
		matches,
		listed: Math.min(matches, MAX_LISTED),
		terms: query.terms.map((t) => t.typed),
		ignored: query.ignored,
		coverage
	};
}

export function searchCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'project_search':
			return search(a.request as SearchRequest);
		case 'rebuild_search_index':
		case 'prepare_project_search':
			return null;
	}
	return undefined;
}
