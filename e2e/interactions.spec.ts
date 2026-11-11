// What a researcher does on the main screens, against the sample data (`?mock`), in both engines:
// a correction made and found again, a dialog that keeps the keyboard until Escape, the window's
// shortcuts, a search in a project, and New job's form kept across a visit to Settings. Elements
// are found by their accessible names, as a screen reader finds them.

import { expect, test, type Page } from '@playwright/test';

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

/** The key the window's shortcuts go with where the page runs: ⌘ where it takes itself for a Mac
 * (src/lib/platform.ts reads the user agent, which Playwright's Safari gives as a Mac's on every
 * system), Ctrl elsewhere. */
async function primary(page: Page): Promise<'Meta' | 'Control'> {
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	return mac ? 'Meta' : 'Control';
}

test('a correction is saved and found again on coming back to the page', async ({ page }) => {
	await open(page, 'review');
	const main = page.getByRole('main');
	const text = main.getByRole('region', { name: 'Reading text' });
	await expect(text).toContainText('deux cents fidèles');

	await main.getByRole('button', { name: 'Correct', exact: true }).first().click();
	const editor = main.getByRole('textbox', { name: /^Correct / });
	await expect(editor).toBeFocused();
	const read = await editor.inputValue();
	expect(read).toContain('deux cents fidèles');
	await editor.fill(read.replace('deux cents fidèles', 'quatre cents fidèles'));
	await main.getByRole('button', { name: /^Save correction/ }).click();

	await expect(editor).toBeHidden();
	await expect(text).toContainText('quatre cents fidèles');
	await expect(text).not.toContainText('deux cents fidèles');

	// Another page, then back: the text shown is still the corrected one.
	await main.getByRole('button', { name: 'Next', exact: true }).click();
	const shown = main.getByRole('combobox', { name: 'Review' }).locator('option:checked');
	await expect(shown).toHaveText('Page 2');
	await expect(text).not.toContainText('quatre cents fidèles');
	await main.getByRole('button', { name: 'Previous', exact: true }).click();
	await expect(text).toContainText('quatre cents fidèles');
	await expect(text).not.toContainText('deux cents fidèles');
});

test('a PDF’s corrected text shows the text the model read beside it, the page a click away', async ({
	page
}) => {
	await open(page, 'review-correction');
	const main = page.getByRole('main');
	const source = main.getByRole('region', { name: 'Source' });
	await expect(source.getByRole('group', { name: /^Page image/ })).toBeVisible();
	await main.getByRole('button', { name: 'Show the text read' }).click();
	// The text layer as the model was given it, misreadings and all.
	await expect(source).toContainText('Le Gouvernenr général');
	await main.getByRole('button', { name: 'Next', exact: true }).click();
	await expect(source).toContainText('commis- sion');
	await main.getByRole('button', { name: 'Show the page' }).click();
	await expect(source.getByRole('group', { name: /^Page image/ })).toBeVisible();
});

test('a dialog keeps the focus inside until Escape closes it', async ({ page }) => {
	await open(page, 'project');
	// Opened from the keyboard: WebKit, as Safari does, leaves a clicked button without the
	// focus, so there would be nothing for the dialog to give it back to.
	const opener = page.getByRole('button', { name: 'Add an existing job' });
	await opener.focus();
	await page.keyboard.press('Enter');
	const dialog = page.getByRole('dialog', { name: 'Add an existing job' });
	await expect(dialog).toBeVisible();

	const focusInDialog = () =>
		dialog.evaluate(
			(box) => box.contains(document.activeElement) && document.activeElement !== box
		);
	expect(await focusInDialog()).toBe(true);
	// On a Mac, Tab stops at fields only unless the system's keyboard navigation is on, so
	// WebKit there may leave the focus on nothing at all: what counts is that it is never on
	// a control of the window behind.
	const focusKept = () =>
		dialog.evaluate(
			(box) => box.contains(document.activeElement) || document.activeElement === document.body
		);
	// Round the dialog's controls more than once each way: the focus never leaves it.
	for (const key of ['Tab', 'Shift+Tab']) {
		for (let i = 0; i < 8; i++) {
			await page.keyboard.press(key);
			expect(await focusKept(), `${key} × ${i + 1} stays in the dialog`).toBe(true);
		}
	}
	// The window behind it waits: its shortcuts do nothing while the dialog is open.
	await page.keyboard.press(`${await primary(page)}+n`);
	await expect(dialog).toBeVisible();

	await page.keyboard.press('Escape');
	await expect(dialog).toBeHidden();
	await expect(opener).toBeFocused();
});

test('the window’s shortcuts focus the screen’s search field and open New job', async ({
	page
}) => {
	await open(page, 'project');
	const key = await primary(page);
	const find = page.getByRole('textbox', { name: 'Find a document' });
	await page.keyboard.press(`${key}+f`);
	await expect(find).toBeFocused();

	// Typed into the field, the letters are the field's.
	await page.keyboard.type('Bulletin');
	await expect(find).toHaveValue('Bulletin');

	await page.getByRole('heading', { level: 1 }).click();
	await page.keyboard.press(`${key}+n`);
	await expect(page.getByRole('heading', { level: 1, name: 'New job' })).toBeVisible();
});

test('a project search finds a passage, accents aside, and opens it in the review', async ({
	page
}) => {
	await open(page, 'project');
	await page
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Search' })
		.click();
	const search = page.getByRole('search');
	await search.getByRole('searchbox', { name: 'Words to find' }).fill('pelerinage');
	await search.getByRole('button', { name: 'Search' }).click();

	const main = page.getByRole('main');
	await expect(main.getByText('1 match in 1 document')).toBeVisible();
	const passage = main.getByRole('button', { name: /^Open Page 1 in the review/ });
	await expect(passage.locator('mark')).toHaveText('pèlerinage');

	await passage.click();
	await expect(main.getByRole('heading', { level: 1, name: 'bulletin-1957-05.pdf' })).toBeVisible();
	await expect(main.getByRole('region', { name: 'Reading text' })).toContainText('pèlerinage');
});

test('New job keeps what was chosen across a visit to Settings', async ({ page }) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('textbox', { name: 'Name of the job' }).fill('Bulletins de Dakar');
	await main.getByRole('checkbox', { name: 'Markdown (.md)' }).check();
	await main.getByRole('radio', { name: /^As on the page/ }).check();
	await main.getByRole('combobox', { name: 'Model' }).selectOption({ label: 'Gemini 4 Argon' });

	const sidebar = page.getByRole('navigation', { name: 'Jobs' });
	await sidebar.getByRole('button', { name: 'Settings' }).click();
	await expect(main.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();
	await sidebar.getByRole('button', { name: /^New job/ }).click();

	await expect(main.getByRole('heading', { level: 1, name: 'New job' })).toBeVisible();
	await expect(main.getByRole('textbox', { name: 'Name of the job' })).toHaveValue(
		'Bulletins de Dakar'
	);
	await expect(main.getByRole('checkbox', { name: 'Markdown (.md)' })).toBeChecked();
	await expect(main.getByRole('radio', { name: /^As on the page/ })).toBeChecked();
	await expect(main.getByRole('combobox', { name: 'Model' }).locator('option:checked')).toHaveText(
		'Gemini 4 Argon'
	);
});

test('a large job’s failed files are listed alone, however far down they sit', async ({ page }) => {
	// A thousand files more, one in fifty failing under the second set-up (src/lib/dev/mockLarge.ts).
	await open(page, 'jobs&rows=1000');
	const main = page.getByRole('main');
	const far = main.getByText('page-00208.pdf', { exact: true });
	await expect(main.getByText('page-00009.pdf', { exact: true })).toBeVisible();
	await expect(far).toHaveCount(0);

	const shown = main.getByRole('combobox', { name: 'Files shown' });
	await shown.selectOption('failed');
	await expect(far).toBeVisible();
	await expect(main.getByText('page-00009.pdf', { exact: true })).toHaveCount(0);
	await expect(main.getByRole('status').filter({ hasText: 'with failures listed' })).toHaveText(
		/^\d+ files with failures listed, of 100\d$/
	);

	await shown.selectOption('all');
	await expect(far).toHaveCount(0);
	await expect(main.getByText('page-00009.pdf', { exact: true })).toBeVisible();
});
