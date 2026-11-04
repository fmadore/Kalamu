// Each main screen opens in WebKit without an error, shortcuts are written as on a Mac, and the
// player plays the formats the engine sends it on macOS (`Playback::WEBKIT` in crates/media).

import { expect, test, type Page } from '@playwright/test';

const VIEWS = [
	'',
	'new',
	'jobs',
	'all-jobs',
	'settings',
	'glossary',
	'prompts-analysis',
	'review',
	'review-audio',
	'review-analysis',
	'review-contents',
	'review-correction',
	'projects',
	'document',
	'queue',
	'workflow',
	'run'
];

/** Errors the page throws or logs while it opens. */
function collectErrors(page: Page): string[] {
	const errors: string[] = [];
	page.on('pageerror', (e) => errors.push(e.message));
	page.on('console', (msg) => {
		if (msg.type() === 'error') errors.push(msg.text());
	});
	return errors;
}

for (const view of VIEWS) {
	test(`the ${view || 'start'} screen opens without an error`, async ({ page }) => {
		const errors = collectErrors(page);
		await page.goto(`/?mock${view ? `&view=${view}` : ''}`);
		await page.waitForLoadState('networkidle');
		await expect(page.locator('main, [role="main"]').first()).toBeVisible();
		expect(errors).toEqual([]);
	});
}

test('shortcuts are written with ⌘ on a Mac', async ({ page }) => {
	test.skip(process.platform !== 'darwin', 'WebKit reports a Mac only on macOS');
	await page.goto('/?mock');
	await expect(page.locator('.kbd').first()).toHaveText('⌘N');
});

test('the player plays every format the engine gives it on macOS', async ({ page }) => {
	test.skip(process.platform !== 'darwin', 'the codecs are the system’s, so only a Mac tells');
	await page.goto('/?mock');
	const support = await page.evaluate(() => {
		const audio = document.createElement('audio');
		const types = {
			// Played as is.
			mp3: 'audio/mpeg',
			m4a: 'audio/mp4',
			aac: 'audio/aac',
			wav: 'audio/wav',
			flac: 'audio/flac',
			mp4: 'video/mp4'
		};
		return Object.fromEntries(
			Object.entries(types).map(([name, type]) => [name, audio.canPlayType(type)])
		);
	});
	for (const [name, answer] of Object.entries(support)) {
		expect(answer, `${name} plays in WebKit`).not.toBe('');
	}
});
