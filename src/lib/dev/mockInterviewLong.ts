// The long interviews of the browser preview (`?mock`): `interview-long` and
// `interview-unbroken` for what the estimate says of a transcript too long to read at once, and
// `interview-parts` for an analysis read in three parts, the last of which could not be read.
// Development builds only.

import type { Catalog } from '../bindings/Catalog.js';
import type { EstimateRequest } from '../bindings/EstimateRequest.js';
import type { PieceView } from '../bindings/PieceView.js';
import type { SetupRequest } from '../bindings/SetupRequest.js';
import type { TextTooLong } from '../bindings/TextTooLong.js';

export const inParts = () => new URLSearchParams(location.search).has('interview-parts');

/** The three parts of `interview-parts`, the last not read. */
const PARTS: [number, number][] = [
	[1, 5],
	[4, 9],
	[8, 13]
];
/** The analysis read in parts: the three parts, then the step that brought them together, with
 * the answers of `whole`. */
export const readInParts = (whole: PieceView): PieceView[] => [
	...PARTS.map(([first, last], i) => ({
		...whole,
		index: i,
		piece: { kind: 'turns' as const, number: i + 1, first, last },
		status: i === 2 ? ('failed' as const) : ('done' as const),
		error: i === 2 ? 'The provider is overloaded. Try again later.' : null,
		ai_fields: i === 2 ? [] : whole.ai_fields
	})),
	{ ...whole, index: PARTS.length }
];

/** Characters in the long transcript of `interview-long`: some 100,000 words. */
const LONG_CHARS = 600_000;

/** `n` to two significant figures, down, as the catalogue rounds what a model reads. */
const roundDown = (n: number) => {
	const step = 10 ** Math.max(0, Math.floor(Math.log10(Math.max(n, 1))) - 1);
	return Math.floor(n / step) * step;
};

/** What the estimate says of an interview's set-up with `interview-long`, as the engine measures
 * it: read in parts by a model that reads less than its four characters a token, a part at most
 * seven tenths of what it reads; with `interview-unbroken`, one turn too long for it even so. */
export function interviewLength(
	request: EstimateRequest,
	setup: SetupRequest,
	catalog: Catalog
): { too_long?: TextTooLong[]; parts?: number } {
	const search = new URLSearchParams(location.search);
	if (!search.has('interview-long')) return {};
	const preset = setup.preset_id ?? '';
	if (
		request.task !== 'analysis' ||
		!(preset === 'analysis.interview' || preset.startsWith('user-'))
	)
		return {};
	const limit = catalog.models.find((m) => m.provider === setup.provider && m.id === setup.model)
		?.limits.max_input_tokens;
	if (limit === undefined || LONG_CHARS <= limit * 4) return {};
	if (!search.has('interview-unbroken'))
		return { parts: Math.ceil(LONG_CHARS / Math.floor(limit * 4 * 0.7)) };
	return {
		too_long: [
			{
				source: request.sources[0]?.id ?? '',
				file: request.sources[0]?.file_name ?? '',
				words: 100_000,
				most_words: roundDown(Math.floor((limit * 4) / 6))
			}
		]
	};
}
