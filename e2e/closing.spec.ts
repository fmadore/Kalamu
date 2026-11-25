// Closing the window with a correction not saved, against the sample data (`?mock`), in both
// engines: the shell asks the interface first, which asks the question leaving a screen asks;
// staying keeps the correction, leaving discards it and quits (#271). The mock closes the
// window as the shell does (src/lib/dev/mockWindow.ts).

import { expect, test, type Dialog, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

const UNSAVED = 'Your changes are not saved. Leave and discard them?';

const closing = (page: Page) => page.evaluate(() => window.__kalamuWindow);

/** Closes the window as its close button does; gives the question asked, if one was, answered
 * with `leave`. */
async function closeWindow(page: Page, leave: boolean): Promise<string | null> {
	let asked: string | null = null;
	const answer = async (dialog: Dialog) => {
		asked = dialog.message();
		await (leave ? dialog.accept() : dialog.dismiss());
	};
	page.once('dialog', answer);
	await page.evaluate(() => window.__kalamuCloseWindow?.());
	// The question comes back through the event and the interface: waited for, then let go.
	await expect
		.poll(() => asked !== null || closing(page).then((c) => c?.closed === true))
		.toBe(true);
	page.off('dialog', answer);
	return asked;
}

test('closing the window asks before a correction not saved is lost, and keeps it on staying', async ({
	page
}) => {
	await page.goto('/?mock&view=review');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');

	await main.getByRole('button', { name: 'Correct', exact: true }).first().click();
	const editor = main.getByRole('textbox', { name: /^Correct / });
	await expect(editor).toBeFocused();
	const typed = (await editor.inputValue()).replace('deux cents', 'quatre cents');
	await editor.fill(typed);
	// The shell is told that a change is not saved.
	await expect.poll(async () => (await closing(page))?.draft).toBe(true);

	// Closing asks the leave question; staying keeps the window and the correction.
	expect(await closeWindow(page, false)).toBe(UNSAVED);
	await expect(editor).toBeVisible();
	await expect(editor).toHaveValue(typed);
	expect(await closing(page)).toEqual({ draft: true, closed: false, quit: false });

	// Leaving discards the correction and quits, as the close would have.
	expect(await closeWindow(page, true)).toBe(UNSAVED);
	await expect(editor).toBeHidden();
	await expect.poll(async () => (await closing(page))?.quit).toBe(true);
	await expect.poll(async () => (await closing(page))?.draft).toBe(false);
});

test('closing the window with nothing unsaved closes it without a question', async ({ page }) => {
	await page.goto('/?mock&view=review');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	// A correction opened but not changed holds nothing to lose.
	await main.getByRole('button', { name: 'Correct', exact: true }).first().click();
	await expect(main.getByRole('textbox', { name: /^Correct / })).toBeFocused();

	expect(await closeWindow(page, false)).toBeNull();
	expect(await closing(page)).toEqual({ draft: false, closed: true, quit: false });
});
