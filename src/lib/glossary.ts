// The glossary: the words the app uses, listed alphabetically in the interface language, the
// words each points to, and the export formats. The definitions themselves are interface text, in the translation
// files; this module only orders and links them, so it can be tested on its own.

import type { ExportFormat } from './bindings/ExportFormat.js';
import type { Messages } from './i18n/index.js';
import { alted, backShortcut, isMac, shifted, shortcut } from './platform.js';
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

/** Where a shortcut works, in the order the page lists them. */
export type KeyPlace = keyof Messages['glossary']['keyPlaces'];

/** A shortcut: the keys that do it (any one of them), and what it does. */
export interface KeyRow {
	keys: string[];
	does: string;
}

/**
 * Every shortcut of the app, by where it works, written for this system (`platform.ts`). Each
 * is one a handler answers: the window's (`+page.svelte`), New job's, the review's frame,
 * text and search, the scan, the readings and the splitters. One added there is listed here.
 */
export function keyboard(m: Messages, mac = isMac): { place: KeyPlace; rows: KeyRow[] }[] {
	const k = m.glossary.keys;
	const back = backShortcut(false, mac);
	return [
		{
			place: 'everywhere',
			rows: [
				{ keys: [shortcut('N', mac)], does: k.newJob },
				{ keys: [shortcut('B', mac)], does: k.sidebar },
				{ keys: [shortcut('F', mac)], does: k.find },
				{
					keys: mac ? [back ?? '', alted('←', mac)] : [back ?? ''],
					does: k.back(alted('←', mac))
				},
				{ keys: [m.keys.escape], does: k.escape }
			]
		},
		{
			place: 'newJob',
			rows: [
				{ keys: [shortcut('↵', mac)], does: k.start },
				{ keys: ['↵'], does: k.addLinks },
				{ keys: [shifted('↵', mac)], does: k.newLine }
			]
		},
		{
			place: 'review',
			rows: [
				{ keys: [shortcut('↵', mac)], does: k.reviewed },
				{ keys: [m.keys.pageDown, alted('→', mac)], does: k.nextPage },
				{ keys: [m.keys.pageUp, alted('←', mac)], does: k.previousPage },
				{ keys: ['F'], does: k.nextDifference },
				{ keys: [shifted('F', mac)], does: k.previousDifference },
				{ keys: ['← →'], does: k.differenceSteps },
				{ keys: [m.keys.home, m.keys.end], does: k.differenceEnds },
				{ keys: ['↵', m.keys.space], does: k.openReadings },
				{ keys: [m.keys.escape], does: k.closeReadings },
				{ keys: ['← ↑ → ↓'], does: k.moveReadings },
				{ keys: ['← →'], does: k.splitter },
				{ keys: ['← →', m.keys.home, m.keys.end], does: k.nameSteps },
				{ keys: ['↵', m.keys.space, shifted('F10', mac), m.keys.menu], does: k.nameMenu },
				{ keys: [shortcut('F', mac)], does: k.search }
			]
		},
		{
			place: 'search',
			rows: [
				{ keys: ['↵'], does: k.nextMatch },
				{ keys: [shifted('↵', mac)], does: k.previousMatch },
				{ keys: [m.keys.escape], does: k.closeSearch }
			]
		},
		{
			place: 'scan',
			rows: [
				{ keys: [shortcut('+', mac)], does: k.zoomIn },
				{ keys: [shortcut('−', mac)], does: k.zoomOut },
				{ keys: [shortcut('0', mac)], does: k.fit },
				{ keys: ['← ↑ → ↓'], does: k.pan }
			]
		}
	];
}

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
