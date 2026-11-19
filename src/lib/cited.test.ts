import { describe, expect, it } from 'vitest';
import type { CitedEvidence } from './bindings/CitedEvidence.js';
import type { CitedValue } from './bindings/CitedValue.js';
import type { Field } from './bindings/Field.js';
import {
	asText,
	cardsOf,
	citedOf,
	draftChanges,
	entryDraft,
	quoteStart,
	saidBy,
	utf16At,
	utf16Span
} from './cited.js';
import { recordingOf } from './citedAnswers.svelte.js';
import { interviewRequest } from './interviewStart.svelte.js';
import { pieceAt } from './pieces.js';
import { editCited, interviewPieces, interviewText, interviewTurns } from './dev/mockInterview.js';

/** A quotation's place in characters, as the engine counts them, of `words` in `text`. */
function anchorOf(text: string, words: string) {
	const at = text.indexOf(words);
	const start = Array.from(text.slice(0, at)).length;
	return { part: 0, start, end: start + Array.from(words).length };
}

describe('a quotation’s place in the text', () => {
	it('is the same in characters and units for plain Latin text', () => {
		const text = 'Bonjour, merci de me recevoir.';
		const span = utf16Span(text, anchorOf(text, 'merci'));
		expect(text.slice(span.start, span.end)).toBe('merci');
	});

	it('moves past an emoji, two units but one character', () => {
		const text = 'On rit 😄 puis on parle du tissu et des prix.';
		const anchor = anchorOf(text, 'du tissu');
		// The engine's count is one less than JavaScript's after the emoji.
		expect(anchor.start).toBe(text.indexOf('du tissu') - 1);
		const span = utf16Span(text, anchor);
		expect(text.slice(span.start, span.end)).toBe('du tissu');
	});

	it('holds Arabic and a quotation that itself holds an emoji', () => {
		const text = 'الحمد لله, mon tour est venu 😄 au bon moment.';
		for (const words of ['الحمد لله', 'mon tour est venu 😄 au bon']) {
			const span = utf16Span(text, anchorOf(text, words));
			expect(text.slice(span.start, span.end)).toBe(words);
		}
	});

	it('stops at the end of the text', () => {
		expect(utf16At('ab', 10)).toBe(2);
	});

	it('finds every quotation of the sample interview, after its emoji and Arabic', () => {
		const quotes = interviewPieces[0].ai_fields.flatMap(
			(a) => citedOf([a], a.key)?.entries.flatMap((e) => e.evidence) ?? []
		);
		expect(quotes.length).toBeGreaterThan(4);
		for (const q of quotes) {
			const span = utf16Span(interviewText, q.anchor);
			expect(interviewText.slice(span.start, span.end)).toBe(q.quote);
		}
		const last = interviewTurns.at(-1)!;
		const span = utf16Span(interviewText, { part: 0, start: last.start, end: last.end });
		expect(interviewText.slice(span.start, span.end)).toBe('Merci beaucoup, Mariam.');
	});
});

const fields: Field[] = [
	{ key: 'name', label: 'Name', instructions: '', kind: { type: 'text' }, reason: false },
	{
		key: 'description',
		label: 'Description',
		instructions: '',
		kind: { type: 'text' },
		reason: false
	}
];
const evidence = (quote: string, speaker: string | null, ms: number | null): CitedEvidence => ({
	anchor: { part: 0, start: 0, end: quote.length },
	quote,
	turn: 1,
	speaker,
	start_ms: ms,
	end_ms: null
});
const value: CitedValue = {
	entries: [
		{
			answers: [
				{ key: 'name', value: 'La tontine' },
				{ key: 'description', value: 'Elle paie la machine.' }
			],
			evidence: [evidence('chaque semaine', 'Mariam', 158_000), evidence('mon tour', null, null)]
		}
	],
	left_out: 1,
	dropped: 0
};

describe('the cards', () => {
	it('title an entry of two fields by its first, the others before its quotations', () => {
		const [card] = cardsOf(value, fields);
		expect(card.title).toBe('La tontine');
		expect(card.notes.map((n) => n.value)).toEqual(['Elle paie la machine.']);
		expect(card.notesFirst).toBe(true);
		expect(card.quotes.map((q) => q.quote)).toEqual(['chaque semaine', 'mon tour']);
	});

	it('lead with the quotations of an entry of one field, untitled', () => {
		const one = [fields[1]];
		const [card] = cardsOf(value, one);
		expect(card.title).toBeNull();
		expect(card.notesFirst).toBe(false);
		expect(card.notes[0].value).toBe('Elle paie la machine.');
	});

	it('say who speaks and when, or only what is known', () => {
		expect(saidBy(value.entries[0].evidence[0])).toBe('Mariam · 00:02:38');
		expect(saidBy(value.entries[0].evidence[1])).toBe('');
		expect(saidBy(evidence('x', null, 3_723_000))).toBe('01:02:03');
	});

	it('name a long quotation by its first words', () => {
		expect(quoteStart('une couture droite, c’est une cliente qui revient.')).toBe(
			'une couture droite, c’est une cliente…'
		);
		expect(quoteStart('mon tour')).toBe('mon tour');
	});

	it('read a cited value out of the answers, and nothing out of another field', () => {
		const answers = [
			{ key: 'themes', value },
			{ key: 'summary', value: 'Un résumé' }
		];
		expect(citedOf(answers, 'themes')).toBe(value);
		expect(citedOf(answers, 'summary')).toBeNull();
		expect(asText(['a', 'b'])).toBe('a; b');
	});

	it('type an entry anew and tell only what changed', () => {
		const draft = entryDraft(value.entries[0], fields);
		expect(draft).toEqual({ name: 'La tontine', description: 'Elle paie la machine.' });
		expect(draftChanges({ ...draft, name: ' La tontine ' }, draft)).toEqual([]);
		expect(draftChanges({ ...draft, name: 'La tontine du quartier' }, draft)).toEqual([
			{ key: 'name', value: 'La tontine du quartier' }
		]);
	});
});

describe('a change to the entries, as the preview makes it', () => {
	const answers = [{ key: 'themes', value }];

	it('removes one quotation and keeps the entry', () => {
		const out = editCited(answers, 'themes', { kind: 'remove_quotation', entry: 0, quotation: 0 });
		expect(citedOf(out, 'themes')?.entries[0].evidence.map((e) => e.quote)).toEqual(['mon tour']);
		// The answers given are not changed: the model's stay as they are.
		expect(value.entries[0].evidence).toHaveLength(2);
	});

	it('removes an entry, and types an entry’s field anew', () => {
		expect(
			citedOf(editCited(answers, 'themes', { kind: 'remove_entry', entry: 0 }), 'themes')?.entries
		).toEqual([]);
		const typed = editCited(answers, 'themes', {
			kind: 'answer',
			entry: 0,
			key: 'name',
			value: 'Entraide'
		});
		expect(citedOf(typed, 'themes')?.entries[0].answers[0].value).toBe('Entraide');
	});

	it('refuses an entry or a quotation that is not there', () => {
		expect(() => editCited(answers, 'themes', { kind: 'remove_entry', entry: 3 })).toThrow();
		expect(() =>
			editCited(answers, 'themes', { kind: 'remove_quotation', entry: 0, quotation: 5 })
		).toThrow();
	});
});

describe('where a quotation plays', () => {
	const from = {
		snapshot_id: 's',
		job_id: 'transcript',
		setup_id: 'a',
		source_id: 'recording',
		original_path: ''
	};
	const review = { source: { id: 'text', derived_from: from } } as never;
	const transcription = () => 'audio_transcription';

	it('is the transcript’s review at the turn’s time', () => {
		expect(recordingOf(evidence('x', 'Mariam', 158_000), review, [], transcription)).toEqual({
			name: 'review',
			jobId: 'transcript',
			sourceId: 'recording',
			at: 158_000
		});
	});

	it('is nowhere without a time, a transcript, or once the transcript is gone', () => {
		expect(recordingOf(evidence('x', null, null), review, [], transcription)).toBeNull();
		expect(recordingOf(evidence('x', null, 5), review, [], () => 'text_extraction')).toBeNull();
		const gone = [{ source_id: 'text', from, freshness: 'gone' as const }];
		expect(recordingOf(evidence('x', null, 5), review, gone, transcription)).toBeNull();
		const plain = { source: { id: 'text' } } as never;
		expect(recordingOf(evidence('x', null, 5), plain, [], transcription)).toBeNull();
	});

	it('is in the piece of the recording that holds the moment', () => {
		const piece = (index: number, start: number) =>
			({
				index,
				piece: { kind: 'segment', index, start_ms: start, end_ms: start + 360_000 }
			}) as never;
		expect(pieceAt([piece(1, 360_000), piece(0, 0)], 527_000)).toEqual({ index: 1, ms: 167_000 });
		expect(pieceAt([piece(0, 0)], 4_000)).toEqual({ index: 0, ms: 4_000 });
		expect(pieceAt([], 4_000)).toBeNull();
	});
});

describe('the interview’s request', () => {
	const request = {
		provider: 'gemini',
		model: 'm',
		preset_id: 'analysis.interview',
		label: null,
		options: { temperature: null, diarize: false, timestamps: false, speakers: null }
	};

	it('carries the question typed, and none when it was left empty', () => {
		expect(interviewRequest(request, '  Les femmes artisanes. ').options.question).toBe(
			'Les femmes artisanes.'
		);
		expect('question' in interviewRequest(request, '  ').options).toBe(false);
	});
});
