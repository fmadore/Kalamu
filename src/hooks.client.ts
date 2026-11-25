// A screen that breaks as it is drawn: SvelteKit hands the screens' boundary (`+page.svelte`)
// only `{ status, message: 'Internal Error' }`, so the error itself is sent to the log here,
// where it is still whole (`unexpected.ts`). Nothing is returned: the boundary needs no more.

import { reportFault } from '#lib/unexpected.js';

/** SvelteKit's client `handleError` hook, given the error as it was thrown. */
export function handleError({ error }: { error: unknown }): void {
	reportFault('screen', error);
}
