// The glossary: the words the app uses, in the order the page lists them, the words each points
// to, and the export formats. The definitions themselves are interface text, in the translation
// files; this module only orders and links them, so it can be tested on its own.

import type { ExportFormat } from './bindings/ExportFormat.js';
import type { Messages } from './i18n/index.js';
import { find } from './search.js';

export type Term = keyof Messages['glossary']['terms'];

/** What a model is, then how it is used, then how its results are checked and recorded. */
export const TERMS: Term[] = [
	'ocr',
	'llm',
	'whisper',
	'apiKey',
	'prompt',
	'setup',
	'pipeline',
	'tokens',
	'reasoning',
	'economy',
	'groundTruth',
	'errorRates',
	'hallucination',
	'provenance',
	'machineReadable'
];

/** The words a definition leans on, shown after it as "See also". */
export const SEE_ALSO: Partial<Record<Term, Term[]>> = {
	ocr: ['llm', 'hallucination'],
	llm: ['prompt', 'tokens', 'hallucination'],
	whisper: ['apiKey'],
	prompt: ['pipeline', 'provenance'],
	setup: ['errorRates'],
	tokens: ['reasoning', 'economy'],
	reasoning: ['tokens'],
	groundTruth: ['errorRates'],
	errorRates: ['groundTruth', 'hallucination'],
	hallucination: ['groundTruth'],
	provenance: ['pipeline', 'machineReadable']
};

/** Every export format, keyed by the type generated from Rust, so a format added there fails the
 * type check until it is placed here and described in the translation files. */
const FORMAT_PLACES: Record<ExportFormat, number> = {
	txt: 0,
	markdown: 1,
	docx: 2,
	json: 3,
	csv: 4,
	srt: 5,
	vtt: 6,
	searchable_pdf: 7
};

/** Every export format, in the order New job offers them, then the provenance file. */
export const FORMATS: (ExportFormat | 'provenance')[] = [
	...(Object.keys(FORMAT_PLACES) as ExportFormat[]).sort(
		(a, b) => FORMAT_PLACES[a] - FORMAT_PLACES[b]
	),
	'provenance'
];

/** The anchor of a term on the page, which links from elsewhere in the app go to. */
export const anchor = (term: Term | 'formats') => `glossary-${term}`;

/** The terms whose word or definition holds the words typed, ignoring capitals and accents. */
export function filterTerms(m: Messages, query: string): Term[] {
	if (query.trim() === '') return TERMS;
	return TERMS.filter((t) => {
		const { word, text } = m.glossary.terms[t];
		return find(word, query).length > 0 || find(text, query).length > 0;
	});
}
