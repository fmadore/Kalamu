// New job's check of photographs against each other (#216), against the sample data's reading
// room (`?photos`, src/lib/dev/mockPhotos.ts): IMG_0142 and IMG_0143 are one page taken twice,
// the second sharper, and IMG_0150 is blurred. Nothing is left out without the researcher, a
// photograph left out comes back with Undo, and keeping both puts the warning away.

import { expect, test, type Page } from '@playwright/test';

async function openReadingRoom(page: Page): Promise<void> {
	await page.goto('/?mock&view=new&photos');
	await page.waitForLoadState('networkidle');
	await page.getByRole('main').getByRole('button', { name: 'Add a folder' }).click();
}

test('two photographs of one page: the sharper is kept, and Undo brings the other back', async ({
	page
}) => {
	await openReadingRoom(page);
	const main = page.getByRole('main');
	const pair = main.getByRole('group', {
		name: 'IMG_0142.jpg and IMG_0143.jpg look like the same page'
	});
	await expect(pair).toBeVisible();
	await expect(pair.getByRole('img', { name: 'IMG_0142.jpg, whole photo' })).toBeVisible();
	await expect(pair.getByRole('img', { name: 'IMG_0143.jpg, close-up' })).toBeVisible();
	await expect(pair.getByText('IMG_0143.jpg is sharper.', { exact: false })).toBeVisible();
	// Nothing is left out before the researcher says so.
	await expect(main.getByRole('button', { name: 'Remove IMG_0142.jpg' })).toBeVisible();

	await pair.getByRole('button', { name: 'Keep the sharper one' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_0142.jpg' })).toHaveCount(0);
	await expect(main.getByRole('button', { name: 'Remove IMG_0143.jpg' })).toBeFocused();
	await expect(pair).toHaveCount(0);

	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_0142.jpg' })).toBeVisible();
	await expect(main.getByRole('button', { name: 'Remove IMG_0142.jpg' })).toBeFocused();
	await expect(pair).toBeVisible();

	await pair.getByRole('button', { name: 'Keep both' }).click();
	await expect(pair).toHaveCount(0);
	await expect(main.getByRole('button', { name: 'Remove IMG_0142.jpg' })).toBeVisible();
	await expect(main.getByRole('button', { name: 'Remove IMG_0143.jpg' })).toBeVisible();
});

test('a blurred photograph is kept or left out, as the researcher chooses', async ({ page }) => {
	await openReadingRoom(page);
	const main = page.getByRole('main');
	const blurred = main.getByRole('group', { name: 'IMG_0150.jpg looks blurred' });
	await expect(blurred).toBeVisible();

	await blurred.getByRole('button', { name: 'Leave it out' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_0150.jpg' })).toHaveCount(0);
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(blurred).toBeVisible();

	// Ctrl+Z (⌘Z on a Mac) undoes it as well, while the message shows.
	await blurred.getByRole('button', { name: 'Leave it out' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_0150.jpg' })).toHaveCount(0);
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	await page.keyboard.press(mac ? 'Meta+z' : 'Control+z');
	await expect(blurred).toBeVisible();
	await expect(page.getByRole('button', { name: 'Undo' })).toHaveCount(0);

	await blurred.getByRole('button', { name: 'Keep it' }).click();
	await expect(blurred).toHaveCount(0);
	await expect(main.getByRole('button', { name: 'Remove IMG_0150.jpg' })).toBeVisible();
});

test('photographs with nothing to say about them show nothing new', async ({ page }) => {
	// The sample folder without `?photos`: four pages, none taken twice, none blurred.
	await page.goto('/?mock&view=new');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Add a folder' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_10.jpg' })).toBeVisible();
	await expect(main.getByRole('status').filter({ hasText: 'Looking for photos' })).toHaveCount(0, {
		timeout: 5000
	});
	await expect(main.getByText(/look like the same page|looks blurred/)).toHaveCount(0);
});
