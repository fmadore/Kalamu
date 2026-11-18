// The search by meaning and the map for the browser preview (`?mock&view=search`, then Meaning;
// `?mock&view=map`): two models offered, one sent to a provider and one on this computer not yet
// downloaded; the consent, the estimate and a preparation that advances each time its status is
// read, so it can be paused; questions answered from the sample bulletin and interview by the
// words they share, as the fake embedder of the Rust tests does; a map of the project's
// documents (`docs=12` adds enough documents to draw one), with the documents nearest to each.
// `meaning=ready` starts with the cloud model chosen, agreed to and the text prepared.
// Development builds only.

import type { EmbeddingModel } from '../bindings/EmbeddingModel.js';
import type { EmbeddingRun } from '../bindings/EmbeddingRun.js';
import type { EmbeddingSpace } from '../bindings/EmbeddingSpace.js';
import type { Job } from '../bindings/Job.js';
import type { MapView } from '../bindings/MapView.js';
import type { MeaningPassage } from '../bindings/MeaningPassage.js';
import type { MeaningRequest } from '../bindings/MeaningRequest.js';
import type { MeaningStatus } from '../bindings/MeaningStatus.js';
import type { MeaningView } from '../bindings/MeaningView.js';
import type { Neighbour } from '../bindings/Neighbour.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { SpaceChoice } from '../bindings/SpaceChoice.js';
import { folded } from '../search.js';
import { audioJob, audioPieces, job, pieces } from './mockJobs.js';
import { DOCUMENT_ID, PROJECT_ID, documentsIn } from './mockProjects.js';

const INTERVIEW_ID = '00000000-0000-0000-0000-0000000000d2';

const MODELS: EmbeddingModel[] = [
	{
		id: 'text-embedding',
		provider: 'gemini',
		name: 'Gemini Embedding',
		dimensions: [768, 1536],
		max_chars: 8000,
		input_price: 0.15,
		local: false,
		multilingual: true
	},
	{
		id: 'embeddinggemma',
		provider: 'llamacpp',
		name: 'EmbeddingGemma',
		dimensions: [768],
		max_chars: 2000,
		input_price: null,
		local: true,
		multilingual: true
	}
];

const spaceOf = (m: EmbeddingModel): EmbeddingSpace => ({
	provider: m.provider,
	model: m.id,
	dimensions: m.dimensions[0]
});
const keyOf = (s: EmbeddingSpace) => `${s.provider}/${s.model}`;

let choice: SpaceChoice | null = null;
const consented = new Map<string, number>();
/** Chunks prepared, by model. */
const prepared = new Map<string, number>();
let runs: EmbeddingRun[] = [];
let running: { run: EmbeddingRun; total: number } | null = null;
let seeded = false;

/** One passage that can be found: a piece of the bulletin's preferred reading or of the
 * interview. */
interface Piece {
	documentId: string;
	of: Job;
	setup: number;
	view: PieceView;
	text: string;
}

function passages(): Piece[] {
	const bulletin = pieces
		.filter((v) => v.setup_id === job.setups[1].id)
		.map((view) => ({ documentId: DOCUMENT_ID, of: job, setup: 1, view }));
	const interview = audioPieces().map((view) => ({
		documentId: INTERVIEW_ID,
		of: audioJob,
		setup: 0,
		view
	}));
	return [...bulletin, ...interview]
		.map((p) => ({ ...p, text: p.view.corrected_text ?? p.view.ai_text ?? '' }))
		.filter((p) => !p.text.startsWith('[BLANK'));
}

/** The documents with text read: all but the letters, whose files are missing or unread. */
function readDocuments() {
	return documentsIn({}).filter((d) => !d.source.file_name.startsWith('lettre-1912'));
}

/** Pieces read across the project: the sample pieces, one for each other document read. */
function pieceCount(): number {
	const others = readDocuments().filter(
		(d) => d.id !== DOCUMENT_ID && d.id !== INTERVIEW_ID
	).length;
	return passages().length + others;
}

function seed() {
	if (seeded) return;
	seeded = true;
	if (new URLSearchParams(location.search).get('meaning') === 'ready') {
		const space = spaceOf(MODELS[0]);
		choice = {
			project_id: PROJECT_ID,
			space,
			chosen_at: Date.now() - 86_400_000,
			consented_at: Date.now() - 86_400_000,
			queries: 0,
			query_tokens: 0,
			query_cost_usd: null
		};
		consented.set(keyOf(space), choice.consented_at ?? 0);
		prepared.set(keyOf(space), pieceCount());
		runs = [run(space, 'done', pieceCount())];
	}
}

function run(space: EmbeddingSpace, state: EmbeddingRun['state'], chunks: number): EmbeddingRun {
	const tokens = chunks * 180;
	return {
		id: runs.length + 1,
		project_id: PROJECT_ID,
		space,
		state,
		chunks,
		tokens,
		cost_usd: space.provider === 'gemini' ? (tokens * 0.15) / 1e6 : null,
		started_at: Date.now() - 60_000,
		ended_at: state === 'running' ? null : Date.now(),
		error: null
	};
}

function model(): EmbeddingModel | null {
	return choice ? (MODELS.find((m) => keyOf(spaceOf(m)) === keyOf(choice!.space)) ?? null) : null;
}

function status(): MeaningStatus {
	seed();
	// A run in progress moves on each time it is looked at.
	if (running) {
		const space = running.run.space;
		const done = Math.min(running.total, running.run.chunks + Math.ceil(running.total / 3));
		prepared.set(keyOf(space), (prepared.get(keyOf(space)) ?? 0) + done - running.run.chunks);
		running.run = {
			...running.run,
			chunks: done,
			tokens: done * 180,
			cost_usd: space.provider === 'gemini' ? (done * 180 * 0.15) / 1e6 : null
		};
		if (done >= running.total) {
			runs = [{ ...running.run, state: 'done', ended_at: Date.now() }, ...runs.slice(1)];
			running = null;
		} else {
			runs = [running.run, ...runs.slice(1)];
		}
	}
	const mod = model();
	const total = pieceCount();
	const done = choice ? Math.min(prepared.get(keyOf(choice.space)) ?? 0, total) : 0;
	const left = total - done;
	const chars = left * 820;
	const spent = runs.reduce((n, r) => n + (r.cost_usd ?? 0), 0) + (choice?.query_cost_usd ?? 0);
	return {
		models: MODELS,
		choice: choice ? { ...choice, consented_at: consented.get(keyOf(choice.space)) ?? null } : null,
		model: mod,
		blocked: mod?.local ? 'provider.not_installed' : null,
		counts: choice
			? {
					pieces: total,
					prepared: done,
					missing: left,
					changed: 0,
					chunks: left,
					chars,
					tokens: Math.ceil(chars / 4),
					skipped: left > 0 ? 1 : 0
				}
			: null,
		estimated_cost_usd: mod && !mod.local ? (Math.ceil(chars / 4) * 0.15) / 1e6 : null,
		running: running
			? {
					project_id: PROJECT_ID,
					run: running.run,
					done: running.run.chunks,
					total: running.total
				}
			: null,
		runs,
		prepared_at: runs.find((r) => r.state === 'done')?.ended_at ?? null,
		spaces: [...prepared]
			.filter(([, n]) => n > 0)
			.map(([key, n]) => {
				const m = MODELS.find((x) => keyOf(spaceOf(x)) === key) ?? MODELS[0];
				return { space: spaceOf(m), chunks: n, bytes: n * m.dimensions[0] * 4 };
			}),
		spent_usd: spent > 0 ? spent : null
	};
}

const words = (text: string) =>
	new Set(
		folded(text)
			.split(/[^\p{L}\p{N}]+/u)
			.filter((w) => w.length > 2)
	);

function similarity(a: Set<string>, b: Set<string>): number {
	if (a.size === 0 || b.size === 0) return 0;
	let shared = 0;
	for (const w of a) if (b.has(w)) shared += 1;
	return shared / Math.sqrt(a.size * b.size);
}

function search(request: MeaningRequest): MeaningView {
	const s = status();
	if (!choice) throw { id: 'meaning.no_model' };
	if (!model()?.local && !consented.has(keyOf(choice.space))) throw { id: 'meaning.no_consent' };
	choice = {
		...choice,
		queries: choice.queries + 1,
		query_tokens: choice.query_tokens + 12,
		query_cost_usd: (choice.query_cost_usd ?? 0) + (12 * 0.15) / 1e6
	};
	const inScope = new Set(
		documentsIn({ collection: request.collection, tag: request.tag }).map((d) => d.id)
	);
	const q = words(request.query);
	const ready = (s.counts?.prepared ?? 0) > 0;
	const scored = passages()
		.filter((p) => ready && inScope.has(p.documentId))
		.filter((p) => !request.task || p.of.task === request.task)
		.filter((p) =>
			request.review === 'any' ? true : (request.review === 'reviewed') === p.view.reviewed
		)
		.map((p) => ({ p, score: similarity(q, words(p.text)) }));
	const titles = new Map(documentsIn({}).map((d) => [d.id, d.title]));
	const byDocument = new Map<string, { p: Piece; score: number }[]>();
	for (const x of scored)
		byDocument.set(x.p.documentId, [...(byDocument.get(x.p.documentId) ?? []), x]);
	const documents = [...byDocument]
		.map(([id, list]) => {
			list.sort((a, b) => b.score - a.score);
			return {
				document_id: id,
				title: titles.get(id) ?? id,
				similarity: list[0].score,
				passages: list.slice(0, 3).map(({ p, score }): MeaningPassage => {
					const firstLine = p.text.split('\n')[0];
					const term = firstLine.split(/\s+/).slice(0, 6).join(' ');
					const start = p.view.piece.kind === 'segment' ? Number(p.view.piece.start_ms) : null;
					return {
						job_id: p.of.id,
						job_name: p.of.name,
						setup_id: p.of.setups[p.setup].id,
						setup_label: p.of.setups[p.setup].label,
						source_id: p.view.source_id,
						task: p.of.task,
						preferred: p.of.id === job.id,
						similarity: score,
						hit: {
							index: p.view.index,
							piece: p.view.piece,
							term,
							nth: 0,
							field: null,
							entry: null,
							entry_number: null,
							reason: false,
							before: '',
							words: p.text.replace(/\s+/g, ' ').slice(0, 280),
							after: '',
							marks: [],
							at_ms: start,
							reviewed: p.view.reviewed,
							corrected: p.view.corrected_text !== null
						}
					};
				})
			};
		})
		.sort((a, b) => b.similarity - a.similarity);
	const read = readDocuments().filter((d) => inScope.has(d.id));
	return {
		space: choice.space,
		documents,
		compared: documents.length,
		coverage: {
			documents: inScope.size,
			compared: documents.length,
			// Prepared, the sample's other documents count as read in nothing searched.
			not_prepared: ready ? 0 : read.length,
			unread: ready ? inScope.size - documents.length : inScope.size - read.length,
			pieces: s.counts?.pieces ?? 0,
			pieces_prepared: s.counts?.prepared ?? 0,
			pieces_missing: s.counts?.missing ?? 0,
			pieces_changed: 0,
			pieces_unread: 0,
			without_preference: 0
		}
	};
}

/** A document's place on the sample map: spread by its title, the same every time, the
 * documents of one collection drawn towards one another. */
function place(id: string, title: string, collections: string[]): { x: number; y: number } {
	let h = 2166136261;
	for (const c of title + id) h = Math.imul(h ^ c.charCodeAt(0), 16777619);
	const pull =
		collections.length > 0
			? (collections[0].charCodeAt(collections[0].length - 1) % 3) * 0.3
			: 0.45;
	return {
		x: Math.min(1, Math.max(0, pull + (((h >>> 0) % 1000) / 1000) * 0.4)),
		y: ((h >>> 10) % 1000) / 1000
	};
}

function mapView(collectionsOf: (id: string) => string[]): MapView {
	const s = status();
	if (!choice) throw { id: 'meaning.no_model' };
	const all = documentsIn({});
	const read = readDocuments();
	const ready = (s.counts?.prepared ?? 0) > 0;
	const placed = ready ? read : [];
	const points = placed.map((d) => {
		const at = place(d.id, d.title, collectionsOf(d.id));
		return {
			document_id: d.id,
			title: d.title,
			x: at.x,
			y: at.y,
			collections: collectionsOf(d.id),
			tags: d.tags,
			chunks: d.id === DOCUMENT_ID ? 2 : d.id === INTERVIEW_ID ? 2 : 1
		};
	});
	return {
		space: choice.space,
		points: points.length >= 5 ? points : [],
		placed: points.length,
		documents: all.length,
		unread: all.length - read.length,
		not_prepared: ready ? 0 : read.length,
		changed: 0,
		without_preference: 0,
		shown: 0.38
	};
}

function neighbours(collectionsOf: (id: string) => string[], id: string): Neighbour[] {
	const points = mapView(collectionsOf).points;
	const me = points.find((p) => p.document_id === id);
	if (!me) return [];
	return points
		.filter((p) => p.document_id !== id)
		.map((p) => ({
			document_id: p.document_id,
			title: p.title,
			similarity: Math.max(-1, 1 - Math.hypot(p.x - me.x, p.y - me.y))
		}))
		.sort((a, b) => b.similarity - a.similarity)
		.slice(0, 10);
}

export function meaningCommand(
	cmd: string,
	a: Record<string, unknown>,
	collectionsOf: (id: string) => string[]
): unknown {
	switch (cmd) {
		case 'meaning_status':
			return status();
		case 'choose_meaning_model': {
			const m = MODELS.find((x) => x.provider === a.provider && x.id === a.model);
			if (!m) throw { id: 'meaning.unknown_model' };
			const space = spaceOf(m);
			choice = {
				project_id: PROJECT_ID,
				space,
				chosen_at: Date.now(),
				consented_at: consented.get(keyOf(space)) ?? null,
				queries: choice?.queries ?? 0,
				query_tokens: choice?.query_tokens ?? 0,
				query_cost_usd: choice?.query_cost_usd ?? null
			};
			return choice;
		}
		case 'consent_meaning':
			if (!choice) throw { id: 'meaning.no_model' };
			consented.set(keyOf(choice.space), Date.now());
			return null;
		case 'prepare_meaning': {
			const s = status();
			if (!choice) throw { id: 'meaning.no_model' };
			if (s.blocked) throw { id: s.blocked };
			if (!model()?.local && !consented.has(keyOf(choice.space)))
				throw { id: 'meaning.no_consent' };
			if (running) throw { id: 'meaning.running' };
			running = { run: run(choice.space, 'running', 0), total: s.counts?.chunks ?? 0 };
			runs = [running.run, ...runs];
			return null;
		}
		case 'pause_meaning':
		case 'cancel_meaning':
			if (running) {
				runs = [
					{
						...running.run,
						state: cmd === 'pause_meaning' ? 'paused' : 'cancelled',
						ended_at: Date.now()
					},
					...runs.slice(1)
				];
				running = null;
			}
			return null;
		case 'remove_meaning_space':
			prepared.delete(keyOf(a.space as EmbeddingSpace));
			return null;
		case 'meaning_search':
			return search(a.request as MeaningRequest);
		case 'meaning_map':
			return mapView(collectionsOf);
		case 'meaning_neighbours':
			return neighbours(collectionsOf, String(a.document));
	}
	return undefined;
}
