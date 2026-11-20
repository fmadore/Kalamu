// The sample project's documents read against a research question (`?mock&view=project`): what
// the sample screening answered for each, the texts "Which of these speak of my question?"
// freezes, its job created, and the table exported. Development builds only.

import type { ChoiceAnswers } from '../bindings/ChoiceAnswers.js';
import type { Document } from '../bindings/Document.js';
import type { DocumentSources } from '../bindings/DocumentSources.js';
import type { Job } from '../bindings/Job.js';
import type { NewJob } from '../bindings/NewJob.js';

export const SCREENING_JOB = '00000000-0000-0000-0000-0000000000s1';
const SCREENING_SETUP = '00000000-0000-0000-0000-0000000000s2';
export const SCREENING_QUESTION =
	'How Catholic and Muslim associations dealt with the colonial administration';

/** What the sample screening answered, by the document's title. */
const ANSWERS: Record<string, string> = {
	'Bulletin, May 1957': 'Relevant',
	'Interview with the imam': 'Unsure',
	'lettre-gouverneur-1956': 'Partly relevant',
	'lettre-1912-pere-eric': 'Not relevant'
};

/** The table exported last, for the end-to-end tests to read. */
declare global {
	interface Window {
		__kalamuExportedAnswers?: { project: string; job: string; setup: string; path: string };
	}
}

/** The sample screening's choice, with each sample document's answer. */
export function screeningChoices(documents: Document[]): ChoiceAnswers[] {
	if (new URLSearchParams(location.search).has('no-screening')) return [];
	const answers = documents
		.filter((d) => ANSWERS[d.title])
		.map((d) => ({ document: d.id, value: ANSWERS[d.title] }));
	if (answers.length === 0) return [];
	return [
		{
			job_id: SCREENING_JOB,
			setup_id: SCREENING_SETUP,
			job_name: `My question: ${SCREENING_QUESTION.slice(0, 40)}…`,
			preset_id: 'analysis.screening',
			question: SCREENING_QUESTION,
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
		case 'export_answers':
			window.__kalamuExportedAnswers = a as unknown as Window['__kalamuExportedAnswers'];
			return null;
		case 'plugin:dialog|save':
			// Where the table goes; every other save keeps the browser's answer, none.
			return JSON.stringify(a).includes('.xlsx')
				? 'C:\\Users\\me\\Documents\\screening.xlsx'
				: undefined;
		default:
			return undefined;
	}
}
