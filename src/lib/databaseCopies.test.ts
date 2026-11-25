import { describe, expect, it } from 'vitest';

import type { DatabaseCopies } from './bindings/DatabaseCopies.js';
import type { DatabaseCopy } from './bindings/DatabaseCopy.js';
import { newestCopy } from './databaseCopies.js';
import { messages, setLocale } from './i18n/index.js';

function copy(name: string, kind: DatabaseCopy['kind'], made: number): DatabaseCopy {
	return { name, kind, made, bytes: 1, path: name };
}

function view(copies: DatabaseCopy[]): DatabaseCopies {
	return {
		folder: 'backups',
		copies,
		damaged: null,
		checked: true,
		skipped: null,
		restored: null,
		restore_failed: null,
		started_empty: false
	};
}

describe('newestCopy', () => {
	it('offers the newest copy that can stand in for the database, never a file set aside', () => {
		const aside = copy('jobs-set-aside-2026-11-25-120000.sqlite', 'set_aside', 3);
		const daily = copy('jobs-daily-2026-11-24-120000.sqlite', 'daily', 2);
		const update = copy('jobs-before-update-2026-11-23-120000.sqlite', 'before_update', 1);
		expect(newestCopy(view([aside, daily, update]))).toBe(daily);
		expect(newestCopy(view([aside, update]))).toBe(update);
		expect(newestCopy(view([aside]))).toBeNull();
	});
});

describe('the words for unreadable jobs', () => {
	it('names one job, or several, in each language', () => {
		setLocale('en');
		expect(messages().database.unreadable('“Letters”', 1)).toContain('One job');
		expect(messages().database.unreadable('“A” and “B”', 2)).toContain('2 jobs');
		setLocale('fr');
		expect(messages().database.unreadable('« Lettres »', 1)).toContain('Un traitement');
		setLocale('en');
	});
});
