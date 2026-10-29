import { describe, expect, it } from 'vitest';
import type { SetupScore } from './bindings/SetupScore.js';
import { en } from './i18n/index.js';
import { describeScore, oneIn, scoredTask } from './scores.js';

const score = (cer: number | null, wer: number | null, pages: number): SetupScore => ({
	setup_id: 'a',
	label: 'gemini',
	cer,
	wer,
	scored_pages: pages,
	cost_usd: null
});

describe('describeScore', () => {
	it('gives both rates, the plain words and the pages counted', () => {
		expect(describeScore(score(0.014, 0.052, 12), 'text_extraction', en)).toBe(
			'1.4% of characters and 5.2% of words wrong (about 1 character in 71), on 12 reviewed pages'
		);
	});

	it('counts parts of a recording, not pages', () => {
		expect(describeScore(score(0.1, 0.2, 1), 'audio_transcription', en)).toBe(
			'10% of characters and 20% of words wrong (about 1 character in 10), on 1 reviewed part'
		);
	});

	it('says when nothing was wrong, and when nothing was reviewed', () => {
		expect(describeScore(score(0, 0, 3), 'text_extraction', en)).toBe(
			'No errors, on 3 reviewed pages'
		);
		expect(describeScore(score(null, null, 0), 'text_extraction', en)).toBeNull();
	});
});

describe('oneIn', () => {
	it('says how many characters there are for each error, as people round it', () => {
		expect(oneIn(0.014)).toBe(71);
		expect(oneIn(0.0012)).toBe(830);
		expect(oneIn(0.1)).toBe(10);
		expect(oneIn(0.4)).toBe(3);
	});

	it('has words for no errors and for very many', () => {
		expect(oneIn(0)).toBe('none');
		expect(oneIn(0.5)).toBe(2);
		expect(oneIn(0.6)).toBe('many');
		expect(oneIn(1.4)).toBe('many');
	});
});

describe('scoredTask', () => {
	it('scores tasks that produce text, not fields', () => {
		expect(scoredTask('text_extraction')).toBe(true);
		expect(scoredTask('audio_transcription')).toBe(true);
		expect(scoredTask('text_correction')).toBe(true);
		expect(scoredTask('analysis')).toBe(false);
		expect(scoredTask('contents')).toBe(false);
	});
});
