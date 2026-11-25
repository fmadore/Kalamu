// A failure nobody expected, against the sample data (`?mock`), in both engines (#280): a screen
// made to fail as it is drawn (`?broken=prompts`, `mockBroken`) says so in its place and offers
// the way out; the sidebar keeps working; the failure reaches the log, which the mock keeps on
// `window.__kalamuLoggedErrors` (src/lib/dev/mockWindow.ts). An error no code caught goes to
// the log too, said once for a burst.

import { expect, test, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

const logged = (page: Page) =>
	page.evaluate(() => (window.__kalamuLoggedErrors ?? []) as Record<string, string>[]);

test('a screen that fails says so, the sidebar still works, and the log has the failure', async ({
	page
}) => {
	await page.goto('/?mock&broken=prompts');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	const sidebar = page.getByRole('navigation').first();

	// The screen that fails: what happened, and the way out.
	await page.getByRole('button', { name: 'Prompts', exact: true }).click();
	await expect(main.getByRole('heading', { name: 'This screen stopped working' })).toBeVisible();
	await expect(main.getByRole('alert')).toContainText('Everything you saved is kept');
	await expect(main.getByRole('button', { name: 'Reload the window' })).toBeVisible();

	// The failure reached the log: where it happened and what it was, nothing more.
	await expect.poll(async () => (await logged(page)).length).toBe(1);
	const [entry] = await logged(page);
	expect(entry).toMatchObject({
		kind: 'screen',
		screen: 'prompts',
		name: 'Error',
		message: 'A screen made to fail (?mock&broken)'
	});
	expect(entry.stack).not.toBe('');
	// The screen's own message says it; no toast on top of it.
	await expect(page.locator('.toast.error')).toHaveCount(0);

	// Back goes to the screen left, drawn as before.
	await main.getByRole('button', { name: 'Go back' }).click();
	await expect(main.getByRole('heading', { name: 'This screen stopped working' })).toHaveCount(0);

	// The sidebar opens another screen in the failed one's place.
	await sidebar.getByRole('button', { name: 'Prompts', exact: true }).click();
	await expect(main.getByRole('heading', { name: 'This screen stopped working' })).toBeVisible();
	await sidebar.getByRole('button', { name: /^Settings/ }).click();
	await expect(main.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible();
	await expect(main.getByRole('heading', { name: 'This screen stopped working' })).toHaveCount(0);
});

test('an error no code caught goes to the log, and is said once for a burst', async ({ page }) => {
	await page.goto('/?mock');
	await page.waitForLoadState('networkidle');
	// The net is laid once the app is mounted: its first screen drawn.
	await expect(page.getByRole('main').getByRole('heading').first()).toBeVisible();
	// From a script of the page's own: what is thrown from evaluated code never reaches the
	// window's listeners.
	await page.addScriptTag({
		content: `for (let i = 0; i < 3; i++) setTimeout(() => {
			throw new TypeError('Nothing to read in "Le Gouverneur général" ' + i);
		});
		Promise.reject(new RangeError('Out of range'));`
	});
	await expect.poll(async () => (await logged(page)).length).toBe(4);
	const entries = await logged(page);
	expect(entries.map((e) => e.kind).sort()).toEqual(['error', 'error', 'error', 'rejection']);
	// One message for the burst.
	await expect(page.locator('.toast.error')).toHaveCount(1);
	await expect(page.locator('.toast.error')).toContainText('Something went wrong in Kalamu');
});
