// The copies of the job database in the browser preview: three daily copies and one made
// before an update. `?database=damaged`: the check after launch found damage; `=skipped`: the
// day's copy was not made, the disk too full; `=none`: no copy yet; `=restored`: a copy was
// opened at this launch. `?unreadable`: one job this version cannot read.

import type { DatabaseCopies } from '#lib/bindings/DatabaseCopies.js';
import type { DatabaseCopy } from '#lib/bindings/DatabaseCopy.js';
import type { UnreadableJob } from '#lib/bindings/UnreadableJob.js';

const FOLDER = 'C:\\Users\\you\\AppData\\Local\\Kalamu Data\\backups';
const DAY = 86_400_000;

function copy(kind: DatabaseCopy['kind'], daysAgo: number, stamp: string): DatabaseCopy {
	const name = `jobs-${kind.replace('_', '-')}-${stamp}.sqlite`;
	return {
		name,
		kind,
		made: Date.now() - daysAgo * DAY,
		bytes: 48_300_000 - daysAgo * 120_000,
		path: `${FOLDER}\\${name}`
	};
}

function copies(): DatabaseCopies {
	const mode = new URLSearchParams(location.search).get('database');
	const kept =
		mode === 'none'
			? []
			: [
					copy('daily', 0, '2026-11-25-091204'),
					copy('daily', 1, '2026-11-24-083311'),
					copy('daily', 2, '2026-11-23-101540'),
					copy('before_update', 2, '2026-11-23-101538')
				];
	return {
		folder: FOLDER,
		copies: kept,
		damaged: mode === 'damaged' ? 'database disk image is malformed' : null,
		checked: true,
		skipped: mode === 'skipped' ? { needed: 48_300_000, free: 812_000_000 } : null,
		restored: mode === 'restored' ? (kept[1]?.name ?? null) : null,
		restore_failed: null,
		started_empty: false
	};
}

/** The jobs `app_info` names as unreadable. */
export function unreadableJobs(): UnreadableJob[] {
	return new URLSearchParams(location.search).has('unreadable')
		? [
				{
					id: '00000000-0000-0000-0000-00000000dead',
					name: 'Registres paroissiaux',
					reason: 'unknown variant `divination`'
				}
			]
		: [];
}

export function databaseCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'database_copies':
			return copies();
		case 'open_database_copy':
			// The app restarts with the copy; here the page reloads, as it would.
			console.info('open_database_copy', a.name);
			location.reload();
			return null;
		default:
			return undefined;
	}
}
