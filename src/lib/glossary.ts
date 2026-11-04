// The glossary: the words the app uses, listed alphabetically in the interface language, the
// words each points to, and the export formats. The definitions themselves are interface text, in the translation
// files; this module only orders and links them, so it can be tested on its own.

import type { ExportFormat } from './bindings/ExportFormat.js';
import type { Messages } from './i18n/index.js';
import { find } from './search.js';

export type Term = keyof Messages['glossary']['terms'];

/** Every word, by theme: what a model is, then how it is used, then how its results are checked
 * and recorded. The page lists them alphabetically (`filterTerms`). */
export const TERMS: Term[] = [
	'ocr',
	'llm',
	'whisper',
	'tesseract',
	'apiKey',
	'prompt',
	'setup',
	'pipeline',
	'workflow',
	'tokens',
	'reasoning',
	'economy',
	'groundTruth',
	'errorRates',
	'hallucination',
	'provenance',
	'machineReadable'
];

/** The screen where a word's thing is kept, which its bubble links to beside the glossary. */
export const PLACES: Partial<Record<Term, 'prompts'>> = { prompt: 'prompts' };

/** The words a definition leans on, shown after it as "See also". */
export const SEE_ALSO: Partial<Record<Term, Term[]>> = {
	ocr: ['llm', 'tesseract', 'hallucination'],
	llm: ['prompt', 'tokens', 'hallucination'],
	whisper: ['apiKey'],
	tesseract: ['ocr', 'machineReadable'],
	prompt: ['pipeline', 'provenance'],
	pipeline: ['workflow'],
	workflow: ['pipeline', 'groundTruth'],
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
	searchable_pdf: 7,
	alto: 8
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

/** The terms whose word or definition holds the words typed, ignoring capitals and accents, in
 * the alphabetical order of their words in the interface language. */
export function filterTerms(m: Messages, query: string): Term[] {
	const order = new Intl.Collator(m.locale.tag, { sensitivity: 'base' });
	const shown =
		query.trim() === ''
			? [...TERMS]
			: TERMS.filter((t) => {
					const { word, text } = m.glossary.terms[t];
					return find(word, query).length > 0 || find(text, query).length > 0;
				});
	return shown.sort((a, b) => order.compare(m.glossary.terms[a].word, m.glossary.terms[b].word));
}
