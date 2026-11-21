import { describe, expect, it } from 'vitest';
import { draftOf, dropDraft, keepDraft } from './drafts.js';

describe('dialog drafts', () => {
	it('keeps what was typed until it is replaced, forgotten or spent', () => {
		expect(draftOf('interview')).toBeNull();
		keepDraft('interview', { question: 'How women learn a trade', themes: '' });
		expect(draftOf('interview')).toEqual({ question: 'How women learn a trade', themes: '' });
		// Read again at each opening, until the job starts.
		expect(draftOf('interview')).toEqual({ question: 'How women learn a trade', themes: '' });
		// A dialog closed as it opened keeps nothing.
		keepDraft('interview', null);
		expect(draftOf('interview')).toBeNull();
		keepDraft('screening:p1:new', 'Which speak of the strike?');
		expect(draftOf('screening:p2:new')).toBeNull();
		dropDraft('screening:p1:new');
		expect(draftOf('screening:p1:new')).toBeNull();
	});

	it('gives a copy, so a change in the dialog is not kept until it closes', () => {
		const typed = { question: 'a' };
		keepDraft('k', typed);
		typed.question = 'b';
		const read = draftOf<{ question: string }>('k') ?? { question: '' };
		expect(read.question).toBe('a');
		read.question = 'c';
		expect(draftOf<{ question: string }>('k')?.question).toBe('a');
	});
});
