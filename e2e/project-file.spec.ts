// A project saved as one file and opened again (issue #213): the dialog says what the file holds
// and never holds and what it weighs, the original files are one named choice, and once written
// it says where the file is. A file opened comes in as a new project, said in a sentence, and a
// damaged file is refused in plain words.

import { expect, test, type Page } from '@playwright/test';

async function open(page: Page, query = ''): Promise<void> {
	await page.goto(`/?mock&view=projects${query}`);
	await page.waitForLoadState('networkidle');
}

test('a project is saved to one file, its original files only when chosen', async ({ page }) => {
	await open(page);
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'New project' }).click();
	await main.getByRole('textbox', { name: 'Name' }).fill('Letters from Kayes');
	await main.getByRole('button', { name: 'Create' }).click();
	await expect(main.getByRole('heading', { level: 1, name: 'Letters from Kayes' })).toBeVisible();

	await main.getByRole('button', { name: 'Save to a file…' }).click();
	const dialog = page.getByRole('dialog', { name: 'Save this project to a file' });
	await expect(dialog).toBeVisible();
	// What goes in, what never does, and what it weighs, before anything is written.
	await expect(dialog).toContainText('4 documents and what 3 jobs read from them');
	await expect(dialog).toContainText('Your corrections and what you marked as checked');
	await expect(dialog).toContainText('Never in the file: your API keys, your settings');
	await expect(dialog).toContainText('The file will take about 12 MB.');
	await expect(dialog).toContainText('1 original file can no longer be found on this computer');
	// The originals: one named choice, off by default, its weight said beside it.
	const originals = dialog.getByRole('checkbox', {
		name: /Include the original files \(scans, recordings\)/
	});
	await expect(originals).not.toBeChecked();
	await expect(dialog).toContainText('4 files, about 3.4 GB.');
	await originals.check();
	await expect(dialog).toContainText('The file will take about 3.4 GB.');

	await dialog.getByRole('button', { name: 'Save…' }).click();
	await expect(dialog).toContainText(
		'The project is saved in C:\\Users\\me\\Documents\\Letters from Kayes.kalamu.'
	);
	await expect(dialog).toContainText('1 original file was left out');
	await expect(
		dialog.getByRole('button', { name: /^Show in (File Explorer|Finder)$/ })
	).toBeVisible();
	const saved = await page.evaluate(() => window.__kalamuSavedProject);
	expect(saved?.withOriginals).toBe(true);
	expect(saved?.path).toBe('C:\\Users\\me\\Documents\\Letters from Kayes.kalamu');
	await dialog.getByRole('button', { name: 'Close' }).click();
	await expect(dialog).toHaveCount(0);
});

test('a project file opens as a new project, said in a sentence', async ({ page }) => {
	await open(page);
	const main = page.getByRole('main');
	const finished = page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('heading', { level: 2, name: /^Finished \d+$/ });
	const before = Number((await finished.textContent())?.match(/\d+/)?.[0]);
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	const said = main.getByRole('status').filter({ hasText: 'is open' });
	await expect(said).toContainText('“Mission letters” is open');
	await expect(said).toContainText('It is a new project beside your others');
	// Saved without its originals: what works, and how the pages come back.
	await expect(said).toContainText('The original files were not saved in it');
	await expect(main.getByRole('button', { name: /^Mission letters/ }).first()).toBeVisible();
	// The jobs it brought are listed at once with the others, not at the next refresh.
	await expect(finished).toHaveAccessibleName(`Finished ${before + 1}`);
	await said.getByRole('button', { name: 'Mission letters' }).click();
	await expect(main.getByRole('heading', { level: 1, name: 'Mission letters' })).toBeVisible();
});

test('a damaged project file is refused in plain words', async ({ page }) => {
	await open(page, '&damaged-file');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	await expect(main.getByRole('alert')).toContainText('This project file is damaged');
	await expect(main.getByRole('status').filter({ hasText: 'is open' })).toHaveCount(0);
});
