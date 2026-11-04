// The rows a researcher has ticked in a table or a list: ticked one by one, all at once, cleared,
// and kept only while the rows still exist. Rows are named by their keys, so the selection holds
// across filters and sorting.

export class Selection<K = string> {
	/** The keys ticked, in the order they were ticked. */
	keys = $state<K[]>([]);

	has(key: K): boolean {
		return this.keys.includes(key);
	}

	/** Ticks a row, or unticks it. */
	toggle(key: K): void {
		this.keys = this.has(key) ? this.keys.filter((k) => k !== key) : [...this.keys, key];
	}

	/** Ticks these rows as well, each once. */
	add(keys: Iterable<K>): void {
		this.keys = [...new Set([...this.keys, ...keys])];
	}

	clear(): void {
		this.keys = [];
	}

	/** Of `rows`, those ticked, in the rows' order. */
	of<T>(rows: readonly T[], keyOf: (row: T) => K): T[] {
		return rows.filter((row) => this.has(keyOf(row)));
	}

	/** Whether every one of these rows is ticked; false when there are none. */
	allOf(keys: readonly K[]): boolean {
		return keys.length > 0 && keys.every((k) => this.has(k));
	}

	/** Ticks every one of these rows, or clears the selection when they all are already. */
	toggleAll(keys: readonly K[]): void {
		this.keys = this.allOf(keys) ? [] : [...keys];
	}

	/** Forgets rows that no longer exist. */
	prune(existing: Iterable<K>): void {
		const kept = new Set(existing);
		this.keys = this.keys.filter((k) => kept.has(k));
	}
}
