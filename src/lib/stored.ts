// Conveniences remembered on this computer: where the review's panes divide, whether the job
// list is folded, how a table was sorted and filtered, the pane a project was left on, how a
// scan was turned. Storage may be missing or refuse to write (a locked-down profile, a full
// disk): reading then gives the default and writing does nothing, so nothing depends on it.

/** What is kept under `key`, if `read` accepts it; otherwise `fallback`. */
export function recall<T>(key: string, read: (stored: unknown) => T | null, fallback: T): T {
	try {
		const raw = localStorage.getItem(key);
		if (raw !== null) return read(JSON.parse(raw)) ?? fallback;
	} catch {
		// Nothing readable, or no storage.
	}
	return fallback;
}

/** Keeps `value` under `key`, as JSON. */
export function keep(key: string, value: unknown): void {
	try {
		localStorage.setItem(key, JSON.stringify(value));
	} catch {
		// Only a convenience.
	}
}

/** What is kept for `id` in the list under `key`, if `read` accepts it; otherwise `null`. */
export function recallFor<T>(
	key: string,
	id: string,
	read: (stored: unknown) => T | null
): T | null {
	const found = entries(key).find(([k]) => k === id);
	return found ? read(found[1]) : null;
}

/** Keeps `value` for `id` in the list under `key`, or forgets it for `null`. The list holds the
 * `limit` ids most recently kept, so it does not grow with every project or file ever opened. */
export function keepFor(key: string, id: string, value: unknown, limit: number): void {
	const kept = entries(key).filter(([k]) => k !== id);
	if (value !== null) kept.push([id, value]);
	keep(key, kept.slice(Math.max(0, kept.length - limit)));
}

/** The pairs of id and value kept under `key`, oldest first. */
function entries(key: string): [string, unknown][] {
	return recall(
		key,
		(v) =>
			Array.isArray(v)
				? v.filter((e): e is [string, unknown] => Array.isArray(e) && typeof e[0] === 'string')
				: null,
		[]
	);
}
