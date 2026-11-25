// Text in Arabic beside text in French, against the sample data (`?mock&arabic`), in both
// engines: each paragraph of the text being checked, of the correction editor and of a
// transcript takes its direction from its own words, so Arabic starts on the right and French
// beside it stays on the left (#272). The interface around them stays left to right.

import { expect, test, type Locator, type Page } from '@playwright/test';
import { ARABIC_PARAGRAPH, ARABIC_TURN } from '../src/lib/dev/mockJobs.js';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&arabic&view=${view}`);
	await page.waitForLoadState('networkidle');
}

/** The side a paragraph's first letter (or the speaker's label it opens with) is drawn on in its
 * box: the right where it reads right to left and starts on its own side, the left where it reads
 * left to right. */
async function startSide(el: Locator): Promise<'left' | 'right' | 'neither'> {
	return el.evaluate((node) => {
		const box = node.getBoundingClientRect();
		const side = (r: DOMRect) =>
			Math.abs(box.right - r.right) < 4
				? 'right'
				: Math.abs(r.left - box.left) < 4
					? 'left'
					: 'neither';
		const label = node.querySelector('.who');
		if (label) return side(label.getBoundingClientRect());
		const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT, {
			acceptNode: (t) => (t.textContent?.trim() ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP)
		});
		const first = walker.nextNode();
		if (!first) return 'neither';
		const at = (first.textContent ?? '').search(/\S/);
		const range = document.createRange();
		range.setStart(first, at);
		range.setEnd(first, at + 1);
		return side(range.getBoundingClientRect());
	});
}

test('the text being checked and its correction take each paragraph’s own direction', async ({
	page
}) => {
	await open(page, 'review');
	const main = page.getByRole('main');
	const text = main.getByRole('region', { name: 'Reading text' });
	const arabic = text.locator('.t', { hasText: ARABIC_PARAGRAPH.slice(0, 12) });
	const french = text.locator('.t', { hasText: 'Le commandant de cercle' });
	await expect(arabic).toBeVisible();
	expect(await startSide(arabic)).toBe('right');
	expect(await startSide(french)).toBe('left');
	// The line numbers stay in the left margin: the page around the text is not turned.
	expect(await text.evaluate((el) => getComputedStyle(el).direction)).toBe('ltr');

	// The correction editor gives each of its lines its own direction, so the Arabic paragraph
	// typed or kept there reads from the right while the French above it reads from the left.
	await main.getByRole('button', { name: 'Correct', exact: true }).first().click();
	const editor = main.getByRole('textbox', { name: /^Correct / });
	await expect(editor).toBeFocused();
	expect(await editor.inputValue()).toContain(ARABIC_PARAGRAPH);
	const style = await editor.evaluate((el) => {
		const s = getComputedStyle(el);
		return { bidi: s.unicodeBidi, align: s.textAlign, direction: s.direction };
	});
	expect(style).toEqual({ bidi: 'plaintext', align: 'start', direction: 'ltr' });
});

test('a transcript’s Arabic turn reads right to left beside the French ones', async ({ page }) => {
	await open(page, 'review-audio');
	const main = page.getByRole('main');

	// Under the player, each turn's opening words in their own direction.
	const turns = main.getByRole('list', { name: 'Transcript lines' });
	const said = turns.locator('.said', { hasText: ARABIC_TURN.slice(0, 8) });
	await expect(said).toBeVisible();
	expect(await said.evaluate((el) => getComputedStyle(el).direction)).toBe('rtl');
	const french = turns.locator('.said', { hasText: 'Bonjour, merci' });
	expect(await french.evaluate((el) => getComputedStyle(el).direction)).toBe('ltr');

	// In the text beside it, the line takes the direction of the words said, not of the
	// speaker's label it opens with.
	const text = main.getByRole('region', { name: 'Reading text' });
	const line = text.locator('.t', { hasText: ARABIC_TURN.slice(0, 8) }).first();
	await expect(line).toBeVisible();
	expect(await startSide(line)).toBe('right');
	expect(await startSide(text.locator('.t', { hasText: 'Bonjour, merci' }).first())).toBe('left');
});
