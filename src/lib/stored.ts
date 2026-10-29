// Conveniences remembered on this computer: where the review's panes divide, whether the job
// list is folded. Storage may be missing or refuse to write (a locked-down profile, a full
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
