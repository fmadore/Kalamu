// The scan in the review, against the sample data (`?mock`), in both engines, with scrollbars that
// take room as Windows' do (Playwright hides them by default): the page is drawn at the pane's
// width, so the pane's width must not change with whether the page overflows it, or the page
// narrows, fits, widens and overflows again without end.

import { expect, test } from '@playwright/test';

test.use({ launchOptions: { ignoreDefaultArgs: ['--hide-scrollbars'] } });

test('the page keeps one width whether or not it overflows the pane', async ({ page }) => {
	await page.setViewportSize({ width: 1100, height: 900 });
	await page.goto('/?mock&view=review');
	await page.waitForLoadState('networkidle');
	const desk = page.locator('.desk.scan');
	await expect(desk.locator('canvas')).toBeVisible();

	const widths: number[] = [];
	// Short enough for the page to overflow, about where it just fits, and tall enough for it not to.
	for (const height of [600, 940, 1300]) {
		await page.setViewportSize({ width: 1100, height });
		await page.waitForTimeout(300);
		// The page rests: one width over a few frames, not two in turn.
		const seen = await desk.evaluate(async (box) => {
			const sizes = new Set<string>();
			for (let i = 0; i < 12; i++) {
				sizes.add(box.querySelector('canvas')?.style.width ?? '');
				await new Promise((r) => setTimeout(r, 40));
			}
			return [...sizes];
		});
		expect(seen, `at a height of ${height}`).toHaveLength(1);
		widths.push(await desk.evaluate((box) => box.clientWidth));
	}
	expect(new Set(widths).size).toBe(1);
});
