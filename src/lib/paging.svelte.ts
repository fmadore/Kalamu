// Long tables drawn a page at a time (#143): the first rows, then more on asking, as the list of
// what to check already does. Filtering and sorting still run over every row, and a selection
// holds rows by their keys, so ticking all or filtering is unchanged by what is drawn; only the
// drawing is cut. A screen reader is told how many rows there are (`aria-rowcount`) and where
// each drawn row stands among them (`aria-rowindex`).

/** Rows drawn at first, and added each time more are asked for. */
export const PAGE_ROWS = 200;

/** How many rows asking for more would add, of `total`, with `limit` drawn. */
export function nextRows(total: number, limit: number, step = PAGE_ROWS): number {
	return Math.max(0, Math.min(step, total - limit));
}

/** The `aria-rowindex` of the row at `index` among the body's rows, the header being row 1. */
export function rowIndex(index: number): number {
	return index + 2;
}

export class Paging {
	/** How many rows are drawn. */
	limit = $state(PAGE_ROWS);
	readonly step: number;

	constructor(step = PAGE_ROWS) {
		this.step = step;
		this.limit = step;
	}

	/** The rows of `all` that are drawn. */
	of<T>(all: readonly T[]): readonly T[] {
		return all.length > this.limit ? all.slice(0, this.limit) : all;
	}

	/** How many more rows asking would add, of `total`. */
	next(total: number): number {
		return nextRows(total, this.limit, this.step);
	}

	/** Draws the next rows, and moves the focus to the first of them in `table` (a table or a
	 * list whose rows are numbered by `data-row`) once drawn, so the keyboard goes on from where
	 * they begin. */
	more(table?: HTMLElement | null): void {
		const first = this.limit;
		this.limit += this.step;
		if (!table) return;
		requestAnimationFrame(() => {
			const row = table.querySelector(`[data-row="${first}"]`);
			row?.querySelector<HTMLElement>('input, button, a, select')?.focus();
		});
	}

	/** Back to the first page, when the rows were filtered or sorted anew. */
	reset(): void {
		this.limit = this.step;
	}
}
