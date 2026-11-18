// The text regions over the scan in the text review (#84), against the sample data (`?mock`), in
// both engines: drawn on request and remembered, gone from region to region with the keyboard,
// each opening its place in the text where the app placed it and saying why not elsewhere; a
// click in the text choosing its region; a page the model placed no text on falling back to the
// page as a whole. The researcher's own regions are drawn-regions.spec.ts's.

import { expect, test, type Locator, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

async function openReview(page: Page): Promise<Locator> {
	await page.goto('/?mock&view=review');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await expect(main.getByRole('region', { name: 'Reading text' })).toContainText(
		'deux cents fidèles'
	);
	return main;
}

/** The text marked in the text pane as the chosen region's place, if any. */
function marked(page: Page): Promise<string | null> {
	return page.evaluate(() => {
		const mark = CSS.highlights.get('region');
		const range = mark ? [...mark][0] : undefined;
		return range ? range.toString() : null;
	});
}

/** Zooms in `times` steps, each let finish before the next: a step glides over a few frames. */
async function zoomIn(main: Locator, times: number): Promise<void> {
	const shown = main.locator('.zoom-value');
	for (let i = 0; i < times; i++) {
		const before = await shown.textContent();
		await main.getByRole('button', { name: 'Zoom in' }).click();
		await expect(shown).not.toHaveText(before ?? '');
		await expect
			.poll(async () => {
				const now = await shown.textContent();
				await new Promise((r) => setTimeout(r, 250));
				return now === (await shown.textContent());
			})
			.toBe(true);
	}
}

/** Presses Tab until `target` has the focus, failing after `most`. */
async function tabTo(page: Page, target: Locator, most = 30): Promise<void> {
	for (let i = 0; i < most; i++) {
		if (await target.evaluate((el) => el === document.activeElement)) return;
		await page.keyboard.press('Tab');
	}
	await expect(target).toBeFocused();
}

test('regions are drawn on request, chosen from the keyboard, and open their place in the text', async ({
	page
}) => {
	const main = await openReview(page);
	const toggle = main.getByRole('button', { name: 'Show text regions' });
	await expect(toggle).toHaveAttribute('aria-pressed', 'false');
	await toggle.click();
	await expect(toggle).toHaveAttribute('aria-pressed', 'true');

	// At the pane's width, the blocks: page 1's eight paragraphs, in two columns.
	const regions = main.getByRole('group', { name: /^Text regions on the page/ });
	const drawn = regions.getByRole('button');
	await expect(drawn).toHaveCount(8);
	await expect(drawn.first()).toHaveAccessibleName('Réception au Gouvernement général');

	// One tab stop; the arrow keys go from one to the next in reading order and choose it.
	await tabTo(page, regions.locator('[tabindex="0"]'));
	await expect(drawn.first()).toBeFocused();
	await page.keyboard.press('ArrowRight');
	await page.keyboard.press('ArrowRight');
	const third = drawn.nth(2);
	await expect(third).toBeFocused();
	await expect(third).toHaveAttribute('aria-pressed', 'true');
	const note = page.locator('.region-note');
	await expect(note).toContainText('Le Gouverneur général a reçu ce matin');
	await expect(note).toContainText('Marked in the text.');
	await expect.poll(() => marked(page)).toMatch(/^Le Gouverneur général a reçu/);
	// The last region, in the second column, with End.
	await page.keyboard.press('End');
	await expect(drawn.last()).toBeFocused();
	await expect.poll(() => marked(page)).toMatch(/^Le commandant de cercle/);

	// Escape lets the choice go: nothing is marked.
	await page.keyboard.press('Escape');
	await expect(note).toBeHidden();
	await expect.poll(() => marked(page)).toBeNull();

	// A click in the text chooses the region whose place holds it.
	const text = main.getByRole('region', { name: 'Reading text' });
	await text.getByText('Une réunion à Bamako').click();
	await expect(regions.getByRole('button', { name: 'Une réunion à Bamako' })).toHaveAttribute(
		'aria-pressed',
		'true'
	);

	// Zoomed in, the lines are drawn instead.
	await zoomIn(main, 3);
	await expect(drawn).toHaveCount(23);

	// Shown again after a restart, as the review's other choices are.
	await page.reload();
	await page.waitForLoadState('networkidle');
	await expect(page.getByRole('button', { name: 'Show text regions' })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await expect(drawn).toHaveCount(8);
});

test('estimated regions look approximate, and a correction leaves a region unmarked', async ({
	page
}) => {
	const main = await openReview(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await page.keyboard.press('PageDown');
	await expect(main.getByText('Source · Page 2')).toBeVisible();
	const regions = main.getByRole('group', { name: /^Text regions on the page/ });
	const block = regions.getByRole('button');
	await expect(block).toHaveCount(1);
	// The block is the model's own: outlined. Its line, estimated within it: dashed.
	expect(await block.evaluate((el) => getComputedStyle(el).borderTopStyle)).toBe('solid');
	await zoomIn(main, 3);
	const line = regions.getByRole('button');
	expect(await line.first().evaluate((el) => getComputedStyle(el).borderTopStyle)).toBe('dashed');
	await line.first().click();
	const note = page.locator('.region-note');
	await expect(note).toContainText('Approximate');
	await expect(note).toContainText('Marked in the text.');

	// The researcher corrects the page: the region's words no longer stand in it as read.
	await main.getByRole('button', { name: 'Correct', exact: true }).first().click();
	await main.getByRole('textbox', { name: /^Correct / }).fill('Une tout autre lecture de la page.');
	await main.getByRole('button', { name: /^Save correction/ }).click();
	await line.first().click();
	await expect(note).toContainText('the page was corrected');
	await expect.poll(() => marked(page)).toBeNull();
});

test('a page the model placed no text on falls back to the page as a whole', async ({ page }) => {
	const main = await openReview(page);
	const toggle = main.getByRole('button', { name: 'Show text regions' });
	await toggle.click();
	await page.keyboard.press('PageDown');
	await page.keyboard.press('PageDown');
	await expect(main.getByText('Source · Page 3')).toBeVisible();
	// The switch stays on, since the researcher can draw there; the page is to be checked whole.
	await expect(toggle).toHaveAttribute('aria-pressed', 'true');
	await expect(toggle).not.toHaveAttribute('aria-disabled', 'true');
	await expect(page.locator('.region-note')).toContainText('check the text against the whole page');
	await expect(main.getByRole('group', { name: /^Text regions on the page/ })).toHaveCount(0);
	await toggle.focus();
	await page.keyboard.press('Enter');
	await expect(toggle).toHaveAttribute('aria-pressed', 'false');
	await expect(page.locator('.region-note')).toBeHidden();
});
