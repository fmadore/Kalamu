// Where the keyboard goes in a menu opened from a job or a name (`ContextMenu`). Pure, so it is
// tested without the app.

/**
 * The item a key moves to among `count` items, from the one at `at` (-1 when none has the
 * focus): ↓ and ↑ go round, from the last to the first and back; Home and End go to the first
 * and last when the menu takes them. -1 for any other key, which the menu leaves alone.
 */
export function menuTarget(count: number, at: number, key: string, homeEnd: boolean): number {
	if (key === 'ArrowDown') return (at + 1) % count;
	if (key === 'ArrowUp') return (at - 1 + count) % count;
	if (homeEnd && key === 'Home') return 0;
	if (homeEnd && key === 'End') return count - 1;
	return -1;
}
