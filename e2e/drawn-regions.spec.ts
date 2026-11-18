// The researcher's own regions over the scan (#84), against the sample data (`?mock`), in both
// engines: nothing new until the regions are shown; a box drawn with the pointer, kept, its text
// typed; one drawn and adjusted from the keyboard alone; an engine's region given another box
// and its own put back; a region removed and brought back with Undo; a page the model placed
// nothing on inviting drawing; the analysis, table-of-contents and records reviews; a turned
// page; a small window.

import { expect, test, type Locator, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view = 'review'): Promise<Locator> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await expect(main.locator('.page-sheet canvas')).toBeVisible();
	return main;
}

/** Whether the page runs as on a Mac, where WebKit's Tab stops at fields only and Option+Tab
 * at every control (see add-items.spec.ts). */
const onMac = (page: Page) => page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));

/** Presses Tab (Shift+Tab going `back`), or Option+Tab on a Mac, until `target` has the focus, failing after `most`. */
async function tabTo(page: Page, target: Locator, most = 60, back = false): Promise<void> {
	const key = `${(await onMac(page)) ? 'Alt+' : ''}${back ? 'Shift+' : ''}Tab`;
	for (let i = 0; i < most; i++) {
		if (await target.evaluate((el) => el === document.activeElement)) return;
		await page.keyboard.press(key);
	}
	await expect(target).toBeFocused();
}

/** Goes to page 3 of the sample newspaper, which its model placed no text on. */
async function pageThree(page: Page, main: Locator): Promise<void> {
	await page.keyboard.press('PageDown');
	await page.keyboard.press('PageDown');
	await expect(main.getByText('Source · Page 3')).toBeVisible();
}

/** Drags on the page shown from one place to another, given in fractions of the page drawn. */
async function dragOn(
	page: Page,
	main: Locator,
	from: [number, number],
	to: [number, number]
): Promise<void> {
	const box = await main.locator('.page-sheet canvas').boundingBox();
	if (!box) throw new Error('the page is not drawn');
	const at = ([x, y]: [number, number]) => [box.x + x * box.width, box.y + y * box.height] as const;
	await page.mouse.move(...at(from));
	await page.mouse.down();
	await page.mouse.move(...at([(from[0] + to[0]) / 2, (from[1] + to[1]) / 2]), { steps: 4 });
	await page.mouse.move(...at(to), { steps: 4 });
	await page.mouse.up();
}

const regionsOn = (main: Locator) => main.getByRole('group', { name: /^Text regions on the page/ });

test('nothing new is shown until the regions are', async ({ page }) => {
	for (const view of ['review', 'review-analysis-scan', 'review-contents', 'review-records']) {
		const main = await open(page, view);
		// Only the text review has a model's regions; elsewhere the switch shows the reader's own.
		const name = view === 'review' ? 'Show text regions' : 'Show regions';
		const toggle = main.getByRole('button', { name, exact: true });
		await expect(toggle).toHaveAttribute('aria-pressed', 'false');
		await expect(main.getByRole('button', { name: 'Draw a region' })).toHaveCount(0);
		await expect(page.locator('.region-note')).toHaveCount(0);
		await expect(regionsOn(main)).toHaveCount(0);
	}
	// A text file has no scan: no switch at all.
	const main = page.getByRole('main');
	await page.goto('/?mock&view=review-analysis');
	await page.waitForLoadState('networkidle');
	await expect(main.getByRole('region', { name: 'Answers' })).toBeVisible();
	await expect(main.getByRole('button', { name: /^Show (text )?regions$/ })).toHaveCount(0);
});

test('a box drawn with the pointer is kept, with what is written there; a slip keeps nothing', async ({
	page
}) => {
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await pageThree(page, main);
	// The page with no regions of the model's invites drawing.
	const note = page.locator('.region-note');
	await expect(note).toContainText('or draw a region yourself');
	const draw = main.getByRole('button', { name: 'Draw a region' });

	// A click without a drag, and a box too small to mean anything: nothing kept, drawing ended.
	await draw.click();
	await expect(draw).toHaveAttribute('aria-pressed', 'true');
	await dragOn(page, main, [0.5, 0.5], [0.5, 0.5]);
	await expect(draw).toHaveAttribute('aria-pressed', 'false');
	await draw.click();
	await dragOn(page, main, [0.5, 0.5], [0.502, 0.6]);
	await expect(draw).toHaveAttribute('aria-pressed', 'false');
	await expect(regionsOn(main)).toHaveCount(0);

	// Drawn: kept at once, chosen, in the hand's blue with a heavier line, named as the
	// researcher's.
	await draw.click();
	await dragOn(page, main, [0.1, 0.1], [0.5, 0.25]);
	const mine = regionsOn(main).getByRole('button', { name: 'Your region' });
	await expect(mine).toHaveCount(1);
	await expect(mine).toHaveAttribute('aria-pressed', 'true');
	await expect(mine).toBeFocused();
	expect(await mine.evaluate((el) => getComputedStyle(el).borderTopWidth)).toBe('2px');
	await expect(note).toContainText('Your region');

	// What is written there: kept when the field is left.
	const field = note.getByRole('textbox', { name: 'What is written here (optional)' });
	await field.fill('Note en marge');
	await field.press('Tab');
	await expect(
		regionsOn(main).getByRole('button', { name: 'Your region: Note en marge' })
	).toHaveCount(1);
	// With Ctrl Enter (⌘ on a Mac) too, which does not mark the page reviewed meanwhile.
	const reviewed = main.getByRole('button', { name: /^Mark as reviewed/ });
	const was = await reviewed.count();
	await field.fill('Note en marge, au crayon');
	await field.press((await onMac(page)) ? 'Meta+Enter' : 'Control+Enter');
	await expect(
		regionsOn(main).getByRole('button', { name: 'Your region: Note en marge, au crayon' })
	).toHaveCount(1);
	await expect(main.getByRole('button', { name: /^Mark as reviewed/ })).toHaveCount(was);

	// Read again from the app with the page: still there.
	await main.getByRole('button', { name: 'Previous' }).first().click();
	await expect(main.getByText('Source · Page 2')).toBeVisible();
	await main.getByRole('button', { name: 'Next' }).first().click();
	await expect(main.getByText('Source · Page 3')).toBeVisible();
	await expect(
		regionsOn(main).getByRole('button', { name: 'Your region: Note en marge, au crayon' })
	).toHaveCount(1);
});

test('a region is drawn and adjusted from the keyboard alone', async ({ page }) => {
	const main = await open(page);
	await pageThree(page, main);
	const toggle = main.getByRole('button', { name: 'Show text regions' });
	await tabTo(page, toggle);
	await page.keyboard.press('Enter');
	const draw = main.getByRole('button', { name: 'Draw a region' });
	await tabTo(page, draw);
	// Pressed, then pressed again: a box in the middle of the view, the focus on it.
	await page.keyboard.press('Enter');
	await expect(draw).toHaveAttribute('aria-pressed', 'true');
	await page.keyboard.press('Enter');
	const box = main.getByRole('application', { name: 'The region’s box' });
	await expect(box).toBeFocused();
	await expect(box).toHaveAccessibleDescription(/the arrow keys move it/);
	const before = await box.boundingBox();
	await page.keyboard.press('ArrowRight');
	await page.keyboard.press('ArrowRight');
	const live = page.locator('.region-tools [aria-live]');
	await expect(live).toHaveText('Moved right');
	await page.keyboard.press('Shift+ArrowDown');
	await expect(live).toHaveText('Taller');
	const after = await box.boundingBox();
	expect(after && before && after.x > before.x && after.height > before.height).toBe(true);
	// Enter keeps it: chosen, and the focus on it.
	await page.keyboard.press('Enter');
	const mine = regionsOn(main).getByRole('button', { name: 'Your region' });
	await expect(mine).toBeFocused();
	await expect(live).toHaveText('Region saved.');

	// Moved again from its card, then put back as it was with Escape.
	const kept = await mine.boundingBox();
	await tabTo(page, page.locator('.region-note').getByRole('button', { name: 'Move or resize' }));
	await page.keyboard.press('Enter');
	await expect(box).toBeFocused();
	await page.keyboard.press('ArrowLeft');
	await page.keyboard.press('Escape');
	await expect(mine).toBeFocused();
	expect(await mine.boundingBox()).toEqual(kept);

	// Escape leaves drawing too, from the button, before the page.
	await tabTo(page, draw, 20, true);
	await page.keyboard.press('Enter');
	await expect(draw).toHaveAttribute('aria-pressed', 'true');
	await page.keyboard.press('Escape');
	await expect(draw).toHaveAttribute('aria-pressed', 'false');
});

test('an engine’s region is given another box, and its own put back', async ({ page }) => {
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	const block = regionsOn(main).getByRole('button', { name: /^Le Gouverneur général a reçu/ });
	await block.click();
	const was = await block.boundingBox();
	const note = page.locator('.region-note');
	await note.getByRole('button', { name: 'Move or resize' }).click();
	const box = main.getByRole('application', { name: 'The region’s box' });
	const frame = await box.boundingBox();
	if (!frame || !was) throw new Error('no box');
	// Dragged whole by the pointer, 30 pixels right; the page does not move under it.
	const desk = main.locator('.desk.scan');
	const scrolled = await desk.evaluate((el) => el.scrollLeft);
	await page.mouse.move(frame.x + frame.width / 2, frame.y + frame.height / 2);
	await page.mouse.down();
	await page.mouse.move(frame.x + frame.width / 2 + 30, frame.y + frame.height / 2, { steps: 6 });
	await page.mouse.up();
	expect(await desk.evaluate((el) => el.scrollLeft)).toBe(scrolled);
	await note.getByRole('button', { name: 'Done' }).click();

	// The engine's region, with the researcher's box: its text kept, named as theirs.
	const moved = regionsOn(main).getByRole('button', { name: /\(your box\)$/ });
	await expect(moved).toHaveCount(1);
	const now = await moved.boundingBox();
	expect(now && Math.round(now.x - was.x)).toBeGreaterThanOrEqual(28);
	await expect(note).toContainText('The model’s own box is kept');
	await expect(note).toContainText('Le Gouverneur général a reçu');

	// Put back: the engine's box again; Undo brings the researcher's back.
	await note.getByRole('button', { name: 'Put back the model’s box' }).click();
	await expect(moved).toHaveCount(0);
	await expect(block).toBeFocused();
	expect(await block.boundingBox()).toEqual(was);
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(moved).toHaveCount(1);
	await expect(moved).toBeFocused();
});

test('a region removed is brought back with Undo, the focus never lost', async ({ page }) => {
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await pageThree(page, main);
	const draw = main.getByRole('button', { name: 'Draw a region' });
	await draw.click();
	await dragOn(page, main, [0.3, 0.42], [0.7, 0.55]);
	const mine = regionsOn(main).getByRole('button', { name: 'Your region' });
	await expect(mine).toHaveCount(1);
	await page.locator('.region-note').getByRole('button', { name: 'Remove' }).click();
	await expect(mine).toHaveCount(0);
	await expect(draw).toBeFocused();
	await expect(page.getByRole('status').filter({ hasText: 'Region removed.' })).toBeVisible();
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(mine).toHaveCount(1);
	await expect(mine).toBeFocused();
});

test('the analysis, table-of-contents and records reviews take regions on their scan', async ({
	page
}) => {
	const main = await open(page, 'review-analysis-scan');
	await main.getByRole('button', { name: 'Show regions', exact: true }).click();
	const note = page.locator('.region-note');
	await expect(note).toContainText('No region on this page yet');
	await main.getByRole('button', { name: 'Draw a region' }).click();
	await dragOn(page, main, [0.2, 0.2], [0.6, 0.4]);
	await expect(regionsOn(main).getByRole('button', { name: 'Your region' })).toHaveCount(1);
	const field = note.getByRole('textbox', { name: 'What is written here (optional)' });
	await field.fill('Titre');
	await field.press('Tab');
	await expect(regionsOn(main).getByRole('button', { name: 'Your region: Titre' })).toHaveCount(1);
	// Another page has its own; back on the first, the region is there.
	await main.getByRole('button', { name: 'Next' }).click();
	await expect(regionsOn(main)).toHaveCount(0);
	await main.getByRole('button', { name: 'Previous' }).click();
	await expect(regionsOn(main).getByRole('button', { name: 'Your region: Titre' })).toHaveCount(1);

	for (const view of ['review-contents', 'review-records']) {
		const shown = await open(page, view);
		// Shown already: the switch is remembered on this computer.
		await expect(shown.getByRole('button', { name: 'Show regions', exact: true })).toHaveAttribute(
			'aria-pressed',
			'true'
		);
		await shown.getByRole('button', { name: 'Draw a region' }).click();
		await dragOn(page, shown, [0.1, 0.5], [0.4, 0.7]);
		await expect(regionsOn(shown).getByRole('button', { name: 'Your region' })).toHaveCount(1);
	}
});

test('on a turned page, a box is drawn where the pointer was', async ({ page }) => {
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await pageThree(page, main);
	await main.getByRole('button', { name: 'Turn the page right' }).click();
	const canvas = main.locator('.page-sheet canvas');
	await expect
		.poll(async () => {
			const b = await canvas.boundingBox();
			return b ? b.width > b.height : false;
		})
		.toBe(true);
	await main.getByRole('button', { name: 'Draw a region' }).click();
	await dragOn(page, main, [0.6, 0.1], [0.9, 0.3]);
	const mine = regionsOn(main).getByRole('button', { name: 'Your region' });
	const page_ = await canvas.boundingBox();
	const drawn = await mine.boundingBox();
	if (!page_ || !drawn) throw new Error('nothing drawn');
	const frac = (b: { x: number; y: number; width: number; height: number }) => ({
		x: (b.x - page_.x) / page_.width,
		y: (b.y - page_.y) / page_.height,
		width: b.width / page_.width,
		height: b.height / page_.height
	});
	const near = (a: object, b: object) =>
		Object.entries(b).forEach(([k, v]) =>
			expect(Math.abs((a as Record<string, number>)[k] - v)).toBeLessThan(0.02)
		);
	near(frac(drawn), { x: 0.6, y: 0.1, width: 0.3, height: 0.2 });
	// Turned back, it is at the top left of the page as read, turned with it.
	await main.getByRole('button', { name: 'Turn the page left' }).click();
	await expect
		.poll(async () => {
			const b = await canvas.boundingBox();
			return b ? b.width < b.height : false;
		})
		.toBe(true);
	// Measured once the page is drawn again, sharp, at its new turn.
	await expect
		.poll(async () => {
			const upright = await canvas.boundingBox();
			const back = await mine.boundingBox();
			if (!upright || !back) return 1;
			const want = { x: 0.1, y: 0.1, width: 0.2, height: 0.3 };
			const got = {
				x: (back.x - upright.x) / upright.width,
				y: (back.y - upright.y) / upright.height,
				width: back.width / upright.width,
				height: back.height / upright.height
			};
			return Math.max(...Object.entries(want).map(([k, v]) => Math.abs(got[k as 'x'] - v)));
		})
		.toBeLessThan(0.02);
});

test('in a window of 640 by 560, the tools and the card stay within reach', async ({ page }) => {
	await page.setViewportSize({ width: 640, height: 560 });
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await pageThree(page, main);
	const draw = main.getByRole('button', { name: 'Draw a region' });
	await expect(draw).toBeInViewport();
	await draw.click();
	await draw.click();
	const note = page.locator('.region-note');
	await expect(note.getByRole('button', { name: 'Done' })).toBeInViewport();
	await note.getByRole('button', { name: 'Done' }).click();
	await expect(regionsOn(main).getByRole('button', { name: 'Your region' })).toBeFocused();
	for (const name of ['Move or resize', 'Remove']) {
		await expect(note.getByRole('button', { name })).toBeVisible();
	}
	expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(640);
});

test('a region drawn is read again by a model, its reading used as the region’s text and undone', async ({
	page
}) => {
	const main = await open(page);
	await main.getByRole('button', { name: 'Show text regions' }).click();
	await pageThree(page, main);
	await main.getByRole('button', { name: 'Draw a region' }).click();
	await dragOn(page, main, [0.1, 0.1], [0.5, 0.25]);
	const mine = regionsOn(main).getByRole('button', { name: /^Your region/ });
	await expect(mine).toHaveAccessibleName('Your region');

	// The card offers it; the dialog says what is sent and what it costs before anything is.
	await page
		.locator('.region-note')
		.getByRole('button', { name: 'Read this region again…' })
		.click();
	const dialog = page.getByRole('dialog', { name: 'Read this region again' });
	await expect(dialog).toBeVisible();
	await expect(dialog).toContainText(/Only this part of the page is sent|Read on this computer/);
	await page.screenshot({ path: 'test-results/region-read-ask.png' });
	await dialog.getByRole('button', { name: 'Read', exact: true }).click();
	const use = dialog.getByRole('button', { name: 'Use as this region’s text' });
	await expect(use).toBeVisible();
	await page.screenshot({ path: 'test-results/region-read-answer.png' });
	await use.click();
	await expect(dialog).toHaveCount(0);

	// Written as the region's own text, and taken back from the message.
	await expect(mine).toHaveAccessibleName(/^Your region: .+/);
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(mine).toHaveAccessibleName('Your region');
});
