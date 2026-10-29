// Error rates against the researcher's review, as the engine computes them (`job_scores`,
// `compare_source`), said in plain words for the job page and the review.

import type { SetupScore } from './bindings/SetupScore.js';
import type { Task } from './bindings/Task.js';
import { formatPercent, type Messages } from './i18n/index.js';

/** The tasks whose results are text the review can score; analyses and tables of contents
 *  answer in fields. */
export function scoredTask(task: Task): boolean {
	return task === 'text_extraction' || task === 'audio_transcription' || task === 'text_correction';
}

/**
 * An error rate as "one in how many": `none` when nothing was wrong, `many` at one in two or
 * worse, otherwise a whole number rounded as people say it: to two significant figures (1 in
 * 71, not 1 in 71.4; 1 in 830, not 1 in 833).
 */
export function oneIn(rate: number): number | 'none' | 'many' {
	if (rate <= 0) return 'none';
	const n = 1 / rate;
	if (n < 2) return 'many';
	return n < 10 ? Math.round(n) : Number(n.toPrecision(2));
}

/**
 * A set-up's score in a sentence: "1.4% of characters and 5.2% of words wrong (about 1
 * character in 71), on 12 reviewed pages". `null` when no page has a reference.
 */
export function describeScore(score: SetupScore, task: Task, m: Messages): string | null {
	if (score.cer === null || score.wer === null) return null;
	const n = score.scored_pages;
	const on = task === 'audio_transcription' ? m.scores.parts(n) : m.scores.pages(n);
	const plain = oneIn(score.cer);
	// The same characters, white space aside, are the same words too.
	if (plain === 'none') return m.scores.noErrors(on);
	const rates = m.scores.rates(formatPercent(score.cer), formatPercent(score.wer));
	const words =
		plain === 'many'
			? m.scores.many
			: m.scores.oneIn(new Intl.NumberFormat(m.locale.tag).format(plain));
	return m.scores.summary(rates, words, on);
}
