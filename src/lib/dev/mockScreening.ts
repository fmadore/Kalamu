// The sample project's documents read against a research question (`?mock&view=project`): what
// the sample screening answered for each, the texts "Which of these speak of my question?"
// freezes, its job created, and the table exported. Development builds only.

import type { ChoiceAnswers } from '../bindings/ChoiceAnswers.js';
import type { CitedEvidence } from '../bindings/CitedEvidence.js';
import type { Document } from '../bindings/Document.js';
import type { DocumentSources } from '../bindings/DocumentSources.js';
import type { EstimateRequest } from '../bindings/EstimateRequest.js';
import type { SetupRequest } from '../bindings/SetupRequest.js';
import type { TextTooLong } from '../bindings/TextTooLong.js';
import type { Job } from '../bindings/Job.js';
import type { NewJob } from '../bindings/NewJob.js';

export const SCREENING_JOB = '00000000-0000-0000-0000-0000000000s1';
const SCREENING_SETUP = '00000000-0000-0000-0000-0000000000s2';
/** The fingerprint of the prompt the sample question was asked with, and would be asked again
 * with: unchanged since (`screening-prompt-changed` says otherwise). */
const SCREENING_PROMPT = '3f6c0a9e1b2d4c5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6';
export const SCREENING_QUESTION =
	'How Catholic and Muslim associations dealt with the colonial administration';

/** What the sample screening answered, by the document's title. */
const ANSWERS: Record<string, string> = {
	'Bulletin, May 1957': 'Relevant',
	'Interview with the imam': 'Unsure',
	'lettre-gouverneur-1956': 'Partly relevant',
	'lettre-1912-pere-eric': 'Not relevant'
};

/** What the estimate says of a screening with `screening-long`: its first text too long for the
 * model even in parts, left out and the others priced. */
export function screeningLength(
	request: EstimateRequest,
	setup: SetupRequest
): { too_long?: TextTooLong[] } {
	const first = request.sources[0];
	if (!new URLSearchParams(location.search).has('screening-long')) return {};
	if (setup.preset_id !== 'analysis.screening' || !first) return {};
	return {
		too_long: [{ source: first.id, file: first.file_name, words: 90_000, most_words: 60_000 }]
	};
}

/** A passage of a page, as the engine keeps it. */
function passage(quote: string, page: number, turn: number): CitedEvidence {
	return {
		anchor: { part: page - 1, start: 0, end: quote.length },
		quote,
		turn,
		speaker: null,
		start_ms: null,
		end_ms: null,
		page: { kind: 'page', number: page }
	};
}

/** What the sample screening said beside its answers: a reason, and the passages it quoted. */
const SAID: Record<string, { reason: string; passages?: CitedEvidence[] }> = {
	'Bulletin, May 1957': {
		reason: 'The bulletin reports the association asking the administration for a meeting room.',
		passages: [
			passage('L’association a demandé une salle de réunion au commandant de cercle.', 2, 7),
			passage('Le commandant a répondu qu’il en référerait au gouverneur.', 3, 12)
		]
	},
	'Interview with the imam': {
		reason: 'The imam speaks of the authorities without saying which ones.'
	},
	'lettre-1912-pere-eric': {
		reason: 'The letter speaks only of the mission’s school.'
	}
};

/** The table exported last, and the estimates asked for, for the end-to-end tests to read. */
declare global {
	interface Window {
		__kalamuExportedAnswers?: { project: string; job: string; setup: string; path: string };
		__kalamuEstimates?: EstimateRequest[];
	}
}

/** The sample screening's choice, with each sample document's answer. */
export function screeningChoices(documents: Document[]): ChoiceAnswers[] {
	const search = new URLSearchParams(location.search);
	if (search.has('no-screening')) return [];
	// With `screening-new`, the bulletin stands for a document added since the question was asked.
	const added = (title: string) => search.has('screening-new') && title === 'Bulletin, May 1957';
	const answers = documents
		.filter((d) => ANSWERS[d.title] && !added(d.title))
		.map((d) => ({
			document: d.id,
			value: ANSWERS[d.title],
			job_id: SCREENING_JOB,
			...(SAID[d.title] ?? {})
		}));
	if (answers.length === 0) return [];
	return [
		{
			job_id: SCREENING_JOB,
			setup_id: SCREENING_SETUP,
			job_name: `My question: ${SCREENING_QUESTION.slice(0, 40)}…`,
			preset_id: 'analysis.screening',
			question: SCREENING_QUESTION,
			provider: 'gemini',
			model: 'gemini-4-carbon',
			languages: { mode: 'explicit', languages: ['fr'] },
			prompt: SCREENING_PROMPT,
			first_asked: Date.UTC(2026, 10, 12, 14, 5),
			unread: documents.filter((d) => added(d.title)).map((d) => d.id),
			field: {
				key: 'relevance',
				label: 'Relevance',
				instructions:
					"Whether the document speaks of the researcher's question, following the instructions above.",
				kind: {
					type: 'choice',
					values: ['Relevant', 'Partly relevant', 'Not relevant', 'Unsure']
				},
				reason: false
			},
			answers
		}
	];
}

/** Answers a screening command, or `undefined` for any other. `linked` says whether a document
 *  was read by a job; `analysisJob` stands for the job created. */
export function screeningCommand(
	cmd: string,
	a: Record<string, unknown>,
	documents: Document[],
	linked: (id: string) => boolean,
	analysisJob: () => Job | undefined
): unknown {
	switch (cmd) {
		case 'document_sources': {
			const chosen = documents.filter((d) => (a.documents as string[]).includes(d.id));
			const read = chosen.filter((d) => linked(d.id));
			const found: DocumentSources = {
				sources: read.map((d, i) => ({
					...d.source,
					id: `00000000-0000-0000-0000-00000000t${String(i).padStart(3, '0')}`,
					path: `C:\\Users\\me\\AppData\\Local\\Kalamu Data\\snapshots\\${i}.txt`,
					file_name: `${d.source.file_name.replace(/\.[^.]+$/, '')}.txt`,
					kind: 'text',
					page_count: null,
					duration_ms: null,
					derived_from: {
						snapshot_id: `snapshot-${d.id}`,
						job_id: '00000000-0000-0000-0000-000000000001',
						setup_id: '00000000-0000-0000-0000-000000000002',
						source_id: d.source.id,
						original_path: d.source.path
					}
				})),
				no_text: chosen.filter((d) => !linked(d.id)).map((d) => d.title),
				undecided: []
			};
			return found;
		}
		case 'create_job': {
			const asked = a.job as NewJob;
			if (asked.setups[0]?.preset_id !== 'analysis.screening') return undefined;
			window.__kalamuCreated = asked;
			return analysisJob();
		}
		case 'prompt_fingerprints': {
			const asked = a.job as NewJob;
			const changed = new URLSearchParams(location.search).has('screening-prompt-changed');
			return asked.setups.map(() => (changed ? 'edited-since' : SCREENING_PROMPT));
		}
		case 'export_answers':
			window.__kalamuExportedAnswers = a as unknown as Window['__kalamuExportedAnswers'];
			return [];
		case 'plugin:dialog|save':
			// Where the table goes; every other save keeps the browser's answer, none.
			return JSON.stringify(a).includes('.xlsx')
				? 'C:\\Users\\me\\Documents\\screening.xlsx'
				: undefined;
		default:
			return undefined;
	}
}
