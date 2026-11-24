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
	// The file written is checked before the save says it is done.
	await expect(dialog).toContainText(
		'The project is saved in C:\\Users\\me\\Documents\\Letters from Kayes.kalamu and checked: everything in it can be opened.'
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

test('a file saved without a corrected text a job read says which, and who read it', async ({
	page
}) => {
	// As 1.10.0 could save it (#239): it opens, and the texts missing are named.
	await open(page, '&missing-text');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	const said = main.getByRole('status').filter({ hasText: 'is open' });
	await expect(said).toContainText('One corrected text that a job read was not in the file');
	await expect(said).toContainText('Letter of 1915, read by Mission letters, summary');
});

test('a complete project file names no missing text', async ({ page }) => {
	await open(page);
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	const said = main.getByRole('status').filter({ hasText: 'is open' });
	await expect(said).toBeVisible();
	await expect(said).not.toContainText('was not in the file');
});

test('a project file is checked without opening it, the verdict first', async ({ page }) => {
	await open(page, '&renamed');
	const main = page.getByRole('main');
	// The list is there before it is counted: a count does not wait for it.
	await expect(main.getByRole('listitem').first()).toBeVisible();
	const projects = await main.getByRole('listitem').count();
	await main.getByRole('button', { name: 'Check a project file…' }).click();
	const dialog = page.getByRole('dialog', { name: 'Check a project file' });
	const report = dialog.getByRole('status');
	await expect(report).toContainText('This file is whole: everything in it can be opened.');
	await expect(report).toContainText('“Mission letters”, saved on');
	await expect(report).toContainText('The 2 corrected texts the jobs read are in it, intact.');
	await expect(report).toContainText('It was saved without the original files');
	// The documents to find again on the other computer, by name.
	await expect(report).toContainText('these 4 documents will show as “File missing”');
	await expect(report.getByRole('listitem').filter({ hasText: 'Map of Kayes' })).toBeVisible();
	await expect(report).toContainText('Draft of 1917: draft?.pdf becomes draft_.pdf');
	await dialog.getByRole('button', { name: 'Close' }).click();
	await expect(dialog).toHaveCount(0);
	// Nothing was opened.
	await expect(main.getByRole('listitem')).toHaveCount(projects);
	await expect(main.getByRole('status').filter({ hasText: 'is open' })).toHaveCount(0);
});

test('a check names what is damaged or missing', async ({ page }) => {
	await open(page, '&altered-file');
	let main = page.getByRole('main');
	await main.getByRole('button', { name: 'Check a project file…' }).click();
	let dialog = page.getByRole('dialog', { name: 'Check a project file' });
	await expect(dialog).toContainText('This file is damaged');
	await expect(dialog).toContainText('This part no longer matches what was saved:');
	// Named first in the report, before the documents to find again.
	await expect(
		dialog.getByRole('listitem').filter({ hasText: 'Letter of 1915' }).first()
	).toBeVisible();

	await open(page, '&missing-text');
	main = page.getByRole('main');
	await main.getByRole('button', { name: 'Check a project file…' }).click();
	dialog = page.getByRole('dialog', { name: 'Check a project file' });
	await expect(dialog).toContainText('This file opens, but some texts are missing from it.');
	await expect(dialog).toContainText('Letter of 1915, read by Mission letters, summary');

	await open(page, '&newer-file');
	main = page.getByRole('main');
	await main.getByRole('button', { name: 'Check a project file…' }).click();
	dialog = page.getByRole('dialog', { name: 'Check a project file' });
	await expect(dialog.getByRole('alert')).toContainText(
		'This project file was saved by a newer version of Kalamu'
	);
});

test('a damaged project file is refused in plain words', async ({ page }) => {
	await open(page, '&damaged-file');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	await expect(main.getByRole('alert')).toContainText('This project file is damaged');
	await expect(main.getByRole('status').filter({ hasText: 'is open' })).toHaveCount(0);
});
