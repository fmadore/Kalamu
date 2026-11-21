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

test('a project search finds one phrase or another, and a word of two letters', async ({
	page
}) => {
	await open(page, 'project');
	await page
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Search' })
		.click();
	const search = page.getByRole('search');
	const main = page.getByRole('main');
	await expect(main.getByText(/^Write OR in capitals between two phrases/)).toBeVisible();

	const box = search.getByRole('searchbox', { name: 'Words to find' });
	await box.fill('pelerinage OR mosquee de Bamako');
	await search.getByRole('button', { name: 'Search' }).click();
	await expect(main.getByText('3 matches in 1 document')).toBeVisible();
	await expect(main.locator('mark')).toHaveText([
		'pèlerinage',
		'mosquée de Bamako',
		'mosquée de Bamako'
	]);

	// Two letters are searched as a word on their own.
	await box.fill('du');
	await expect(search.getByRole('button', { name: 'Search' })).toBeEnabled();
	await expect(main.getByText('Type two letters at least.')).toBeHidden();
});

test('New job keeps what was chosen across a visit to Settings', async ({ page }) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('textbox', { name: 'Name of the job' }).fill('Bulletins de Dakar');
	await main.getByRole('checkbox', { name: 'Markdown (.md)' }).check();
	await main.getByRole('radio', { name: /^As on the page/ }).check();
	await main.getByRole('combobox', { name: 'Model' }).click();
	await main.getByRole('option', { name: /^Gemini 4 Argon/ }).click();

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
	await expect(main.getByRole('combobox', { name: 'Model' })).toHaveText(/Gemini 4 Argon/);
});

test('a Tesseract language not downloaded is offered at once in Settings', async ({ page }) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main
		.getByRole('combobox', { name: 'Provider' })
		.first()
		.selectOption({ label: 'Tesseract (on this computer)' });
	const box = main.getByRole('searchbox', { name: 'Type to find a language' });
	await box.fill('Arabic');
	await page.keyboard.press('Enter');
	await expect(main).toContainText('Language data not yet on this computer: Arabic.');
	await main.getByRole('button', { name: 'Download in Settings' }).click();

	// Settings lists Arabic among the rows, in view, its Download button there, without typing it
	// again; focused when no other download runs.
	await expect(main.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();
	const asked = main.locator('[data-wanted]');
	await expect(asked).toHaveCount(1);
	await expect(asked).toContainText('Arabic');
	await expect(asked).toContainText('New job needs this language');
	// (The sample data has another download running, so it waits its turn.)
	await expect(asked.getByRole('button', { name: 'Download' })).toBeVisible();
	await expect(asked).toBeInViewport();
});

test('New job lists models by tier across providers, and remembers it', async ({ page }) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	const setups = main.getByRole('region', { name: /^Model and prompt/ });
	const model = setups.getByRole('combobox', { name: 'Model' }).first();
	// By provider: a provider field, and the provider's models under their tiers, each with its
	// description before it is chosen.
	await expect(setups.getByRole('combobox', { name: 'Provider' }).first()).toBeVisible();
	await model.click();
	const list = page.getByRole('listbox', { name: 'Model' });
	await expect(list.getByRole('group').first()).toHaveAccessibleName('Most capable');
	await expect(list.getByRole('option', { name: /^Gemini 4 Hydrogen/ })).toContainText(
		'which replaces 3.8 Flash'
	);
	// A model replaced by a newer one is not offered.
	await expect(list.getByRole('option', { name: /^Gemini 3\.8 Flash/ })).toHaveCount(0);
	// Opening puts the focus on the chosen model, once the browser has told the list it is open:
	// a key pressed before then reaches the button, not the list.
	await expect(list.getByRole('option', { name: /^Gemini 4 Hydrogen/ })).toBeFocused();
	// The keys move through the list, Enter chooses, and the focus goes back to the field.
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(list).toBeHidden();
	await expect(model).toBeFocused();
	await expect(model).toHaveText(/Gemini 3\.5 Flash-Lite/);

	await setups.getByRole('button', { name: 'Tier', exact: true }).click();
	await expect(setups.getByRole('button', { name: 'Tier', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	await expect(setups.getByRole('combobox', { name: 'Provider' })).toHaveCount(0);
	// One list of every provider's models: choosing Claude's chooses Anthropic.
	await model.click();
	await list.getByRole('option', { name: /^Claude Sonnet 5\.5/ }).click();
	await expect(model).toHaveText(/Claude Sonnet 5\.5\s*Anthropic Claude/);
	await expect(setups.getByText('Balanced', { exact: true }).filter({ visible: true })).toHaveCount(
		1
	);

	await page.reload();
	await page.waitForLoadState('networkidle');
	await expect(
		page.getByRole('main').getByRole('button', { name: 'Tier', exact: true })
	).toHaveAttribute('aria-pressed', 'true');
	await page.getByRole('main').getByRole('button', { name: 'Provider', exact: true }).click();
	await expect(
		page.getByRole('main').getByRole('combobox', { name: 'Provider' }).first()
	).toBeVisible();
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

test('a folder of photographs becomes one document, its pages put in order', async ({ page }) => {
	// The sample folder holds IMG_10, IMG_2, IMG_1 and IMG_3 (src/lib/dev/mockPages.ts).
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Add a folder' }).click();
	await expect(main.getByRole('button', { name: 'Remove IMG_10.jpg' })).toBeVisible();

	const separate = main.getByRole('radio', { name: /^Separate documents/ });
	await expect(separate).toBeChecked();
	await main.getByRole('radio', { name: /^Pages of one document/ }).check();
	await expect(main.getByRole('textbox', { name: 'Name of the document' })).toHaveValue('Box 3');

	const order = main.getByRole('list', { name: 'Order of the pages' });
	const names = () => order.getByRole('listitem').locator('.name').allTextContents();
	await expect.poll(names).toEqual(['IMG_1.jpg', 'IMG_2.jpg', 'IMG_3.jpg', 'IMG_10.jpg']);
	await expect(order.getByText('No files numbered 4 to 9 before this one')).toBeVisible();

	// Moved with the keyboard: the focus stays on the page moved.
	await order.getByRole('button', { name: 'Move IMG_10.jpg up' }).focus();
	await page.keyboard.press('Enter');
	await expect.poll(names).toEqual(['IMG_1.jpg', 'IMG_2.jpg', 'IMG_10.jpg', 'IMG_3.jpg']);
	await expect(order.getByRole('button', { name: 'Move IMG_10.jpg up' })).toBeFocused();
	await expect(order.getByRole('button', { name: 'Move IMG_1.jpg up' })).toBeDisabled();

	await expect(main.getByText(/^In the order you chose/)).toBeVisible();

	await main.getByRole('button', { name: 'Back to the order of the names' }).click();
	await expect.poll(names).toEqual(['IMG_1.jpg', 'IMG_2.jpg', 'IMG_3.jpg', 'IMG_10.jpg']);
	await expect(main.getByText(/^Sorted by the numbers in the file names/)).toBeVisible();

	// Dragged with the pointer, which the window leaves to the page (its own drag and drop is
	// kept for files dropped onto it): IMG_10 goes before IMG_2, then IMG_1 below IMG_3. The mouse
	// goes where the rows are, so the list is brought into view first, as a researcher scrolls to
	// it: focusing the button above does not always scroll the window in WebKit, which left the
	// last row below the window's edge, pressed where no row receives the press.
	const drag = async (name: string, onto: string) => {
		const row = (file: string) =>
			order.getByRole('listitem').filter({ has: page.getByText(file, { exact: true }) });
		await order.scrollIntoViewIfNeeded();
		const from = await row(name).locator('.name').boundingBox();
		const to = await row(onto).locator('.name').boundingBox();
		if (!from || !to) throw new Error('A page is out of view');
		await page.mouse.move(from.x + 4, from.y + from.height / 2);
		await page.mouse.down();
		await page.mouse.move(to.x + 4, to.y + to.height / 2, { steps: 8 });
		await page.mouse.up();
	};
	await drag('IMG_10.jpg', 'IMG_2.jpg');
	await expect.poll(names).toEqual(['IMG_1.jpg', 'IMG_10.jpg', 'IMG_2.jpg', 'IMG_3.jpg']);
	await drag('IMG_1.jpg', 'IMG_3.jpg');
	await expect.poll(names).toEqual(['IMG_10.jpg', 'IMG_2.jpg', 'IMG_3.jpg', 'IMG_1.jpg']);
	await expect(main.getByText(/^In the order you chose/)).toBeVisible();

	// The job is named after the document, as long as no name of its own was typed.
	const jobName = main.getByRole('textbox', { name: 'Name of the job' });
	await main.getByRole('textbox', { name: 'Name of the document' }).fill('Box 3 letters');
	await expect(jobName).toHaveValue('Box 3 letters');

	// Read as separate documents again, the order is not asked for, and the folder names the job.
	await separate.check();
	await expect(order).toHaveCount(0);
	await expect(jobName).toHaveValue('Box 3');
});

test('New job says why Start waits for the language to translate into', async ({ page }) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: /^Translate a text/ }).click();
	await main.getByRole('button', { name: 'Add files' }).click();
	const start = page.getByRole('button', { name: /^Start(?! over)/ });
	await expect(start).toBeDisabled();
	await expect(start).toHaveAccessibleDescription('Choose the language to translate into.');
});

test('New job offers a translation of text without times no subtitles, and says why', async ({
	page
}) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: /^Translate a text/ }).click();
	await main.getByRole('button', { name: 'Add files' }).click();
	const formats = main.getByRole('group', { name: /^Formats/ });
	await expect(formats.getByRole('checkbox', { name: 'Word (.docx)' })).toBeVisible();
	await expect(formats.getByRole('checkbox', { name: /^Subtitles/ })).toHaveCount(0);
	await expect(formats).toContainText(
		'Subtitles (SRT, WebVTT) are offered for the translation of a transcript with times'
	);
	// A transcription has its own times: no such note.
	await main.getByRole('button', { name: /^Transcribe audio/ }).click();
	await expect(main.getByRole('checkbox', { name: 'Subtitles (.srt)' })).toBeVisible();
	await expect(main.getByText('are offered for the translation of a transcript')).toHaveCount(0);
});

test('the language box opens its list, moves through it with the arrows and takes the first found with Enter', async ({
	page
}) => {
	await open(page, 'new');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: /^Translate a text/ }).click();
	await main.getByRole('button', { name: 'Add files' }).click();
	const box = main.getByRole('searchbox', { name: 'Type to find a language' });
	const list = page.locator(`[id="${await box.getAttribute('aria-controls')}"]`);
	const picks = list.locator('button.pick');

	// ↓ from the box goes to the first language, ↓ and ↑ move, ↑ from the first goes back.
	await box.focus();
	await expect(list).toBeVisible();
	await page.keyboard.press('ArrowDown');
	await expect(picks.first()).toBeFocused();
	await page.keyboard.press('ArrowDown');
	await expect(picks.nth(1)).toBeFocused();
	await page.keyboard.press('ArrowUp');
	await expect(picks.first()).toBeFocused();
	await page.keyboard.press('ArrowUp');
	await expect(box).toBeFocused();

	// Escape from the list closes it and goes back to the box, the list staying closed; ↓ opens
	// it again, and Escape in the box closes it.
	await page.keyboard.press('ArrowDown');
	await expect(picks.first()).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(box).toBeFocused();
	await expect(list).toBeHidden();
	await page.keyboard.press('ArrowDown');
	await expect(picks.first()).toBeFocused();
	await page.keyboard.press('ArrowUp');
	await expect(box).toBeFocused();
	await expect(list).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(list).toBeHidden();

	// Typing opens it again; Enter takes the first language found as a chip, empties the box and
	// keeps the focus in it.
	const chips = main.getByRole('button', { name: /^Remove / });
	const before = await chips.count();
	await page.keyboard.type('a');
	await expect(list).toBeVisible();
	await expect(picks.first()).toBeVisible();
	await page.keyboard.press('Enter');
	await expect(chips).toHaveCount(before + 1);
	await expect(box).toHaveValue('');
	await expect(box).toBeFocused();

	// Escape with something typed closes the list and keeps what was typed: the box, a search
	// box, would empty itself and open the list again.
	await page.keyboard.type('fr');
	await expect(list).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(list).toBeHidden();
	await expect(box).toHaveValue('fr');
	await expect(box).toBeFocused();
});

test('the language data box takes the same keys, in Settings', async ({ page }) => {
	await open(page, 'settings');
	const main = page.getByRole('main');
	await main.getByRole('tab', { name: 'Offline models' }).click();
	const box = main.getByRole('searchbox', { name: 'Find a language by its name' }).first();
	await box.scrollIntoViewIfNeeded();
	const list = page.locator(`[id="${await box.getAttribute('aria-controls')}"]`);

	// The sample data has a download running: the list says so, and its languages wait, out of
	// the arrows' reach, the focus staying in the box.
	await box.focus();
	await expect(list).toBeVisible();
	await expect(list).toContainText('One download at a time');
	await expect(list.getByRole('button').first()).toBeDisabled();
	await page.keyboard.press('ArrowDown');
	await expect(box).toBeFocused();
	// Enter takes nothing while it runs; Escape closes the list, and typing opens it again.
	await page.keyboard.press('Enter');
	await expect(list).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(list).toBeHidden();
	await page.keyboard.type('ar');
	await expect(list).toBeVisible();
	await expect(list).toContainText('Arabic');
	// With something typed, Escape closes it all the same, and what was typed stays.
	await page.keyboard.press('Escape');
	await expect(list).toBeHidden();
	await expect(box).toHaveValue('ar');
});

test('a difference’s readings show whole in a short text pane, wherever its words stand', async ({
	page
}) => {
	await page.setViewportSize({ width: 1100, height: 480 });
	await open(page, 'review');
	const text = page.getByRole('main').getByRole('region', { name: 'Reading text' });
	const pane = text.locator('.page');
	const readings = page.getByRole('dialog', { name: 'Choose the correct reading' });
	const marks = text.locator('[data-entry]');
	const count = Math.min(await marks.count(), 6);
	expect(count).toBeGreaterThan(0);
	for (let i = 0; i < count; i++) {
		const mark = marks.nth(i);
		await mark.scrollIntoViewIfNeeded();
		await mark.click();
		await expect(readings).toBeVisible();
		const [area, box] = [await pane.boundingBox(), await readings.boundingBox()];
		if (!area || !box) throw new Error('not drawn');
		const bottom = Math.min(area.y + area.height, 480);
		expect(box.y, `difference ${i + 1}: top`).toBeGreaterThanOrEqual(area.y - 1);
		expect(box.y + box.height, `difference ${i + 1}: bottom`).toBeLessThanOrEqual(bottom + 1);
		await readings.getByRole('button', { name: 'Close' }).click();
		await expect(readings).toBeHidden();
	}
});

test('Add an existing job fits a small window, its buttons in reach', async ({ page }) => {
	await page.setViewportSize({ width: 560, height: 700 });
	await open(page, 'project');
	await page.getByRole('button', { name: 'Add an existing job' }).click();
	const dialog = page.getByRole('dialog', { name: 'Add an existing job' });
	const job = dialog.getByRole('combobox').first();
	const first = await job.locator('option').nth(1).getAttribute('value');
	await job.selectOption(first ?? '');
	await expect(dialog.getByRole('table')).toBeVisible();
	const sideways = await dialog.evaluate((el) => el.scrollWidth - el.clientWidth);
	expect(sideways).toBeLessThanOrEqual(1);
	for (const name of ['Cancel', 'Add']) {
		const button = dialog.getByRole('button', { name, exact: true });
		await expect(button).toBeInViewport({ ratio: 1 });
	}
});

test('a place where a model read nothing says so in plain words, apart from what was read', async ({
	page
}) => {
	await open(page, 'review');
	const main = page.getByRole('main');
	const text = main.getByRole('region', { name: 'Reading text' });
	await text.locator('[data-entry]', { hasText: 'ce matin' }).click();
	const readings = page.getByRole('dialog', { name: 'Choose the correct reading' });
	await expect(readings).toBeVisible();

	// The second model left the words out: its reading says there is no text, in the
	// interface's italic and muted words rather than as words a model wrote.
	const none = readings.getByRole('button', { name: /^\(no text\)/ });
	await expect(none).toBeVisible();
	await expect(readings).not.toContainText('om.');
	const label = none.getByText('(no text)', { exact: true });
	await expect(label).toHaveCSS('font-style', 'italic');

	// Taking it leaves the words out of the text.
	await none.click();
	await expect(readings).toBeHidden();
	await expect(text).toContainText('a reçu les délégués');
	await expect(text).not.toContainText('reçu ce matin');
});

test('a name pointed at in an analysis is brought into view in the text beside it', async ({
	page
}) => {
	// A short window, so the text runs past the foot of its pane.
	await page.setViewportSize({ width: 1100, height: 420 });
	await open(page, 'review-analysis');
	const main = page.getByRole('main');
	const source = main.getByRole('region', { name: 'Source', exact: true });
	const text = source.locator('.document');
	const names = main.getByRole('region', { name: 'Answers' }).locator('button.item');
	await expect(names.first()).toBeVisible();

	// The name whose first place in the text lies furthest down, out of view.
	const below = await text.evaluate(
		(pane, all) => {
			const whole = (pane.textContent ?? '').toLowerCase();
			const placed = all
				.map((name) => ({ name, at: whole.indexOf(name.toLowerCase()) }))
				.filter((n) => n.at >= 0)
				.sort((a, b) => b.at - a.at);
			return placed[0]?.name ?? null;
		},
		await names.allTextContents()
	);
	expect(below).not.toBeNull();
	expect(await text.evaluate((pane) => pane.scrollHeight > pane.clientHeight)).toBe(true);

	await names
		.filter({ hasText: below ?? '' })
		.first()
		.hover();
	const mark = source.locator('mark').first();
	await expect(mark).toHaveText(new RegExp(below ?? '', 'i'));
	await expect
		.poll(() =>
			text.evaluate((pane) => {
				const box = pane.getBoundingClientRect();
				return [...pane.querySelectorAll('mark')].some((m) => {
					const r = m.getBoundingClientRect();
					return r.top >= box.top && r.bottom <= box.bottom;
				});
			})
		)
		.toBe(true);
	expect(await text.evaluate((pane) => pane.scrollTop)).toBeGreaterThan(0);
});

test('what to check draws a sample at random and keeps an item aside with a note', async ({
	page
}) => {
	await open(page, 'queue');
	const main = page.getByRole('main');
	const sample = main.getByRole('region', { name: /^Drawn at random/ });
	await sample.getByRole('spinbutton', { name: 'Pages' }).fill('2');
	await sample.getByRole('button', { name: 'Draw at random' }).click();
	// The seed is shown, to cite the sample, and the pages drawn are listed apart.
	await expect(sample.getByText(/^Seed \d+/)).toBeVisible();
	await expect(sample.getByText('0 of 2 reviewed')).toBeVisible();
	await expect(sample.getByRole('button', { name: /^Open Interview with the imam/ })).toHaveCount(
		2
	);
	await expect(sample.getByRole('button', { name: 'Draw again' })).toBeVisible();

	await main.getByRole('button', { name: 'Keep Bulletin, May 1957, Page 1, aside' }).click();
	await main.getByRole('textbox', { name: 'Note' }).fill('Ask the archivist about the stamp');
	await main.getByRole('button', { name: 'Keep aside', exact: true }).click();
	const kept = main.getByRole('heading', { level: 3, name: /^Kept aside/ });
	await expect(kept).toBeVisible();
	await expect(main.getByText('Ask the archivist about the stamp')).toBeVisible();

	await main.getByRole('button', { name: 'Put back' }).click();
	await expect(kept).toBeHidden();
});

test('what to check draws the same sample again from a seed typed by hand', async ({ page }) => {
	await open(page, 'queue');
	const main = page.getByRole('main');
	const sample = main.getByRole('region', { name: /^Drawn at random/ });
	const seed = sample.getByRole('textbox', { name: 'Seed' });
	// Seven digits are not a seed: said, and nothing drawn.
	await seed.fill('1234567');
	await seed.press('Enter');
	await expect(sample.getByRole('alert')).toHaveText('A seed is a number of six digits at most.');
	await expect(seed).toHaveAttribute('aria-invalid', 'true');
	await expect(seed).toBeFocused();
	await expect(sample.getByText(/^Seed \d+/)).toBeHidden();

	// A seed read in a manifest draws with it, and the box empties for the next draw.
	await seed.fill('271828');
	await expect(sample.getByRole('alert')).toBeHidden();
	await sample.getByRole('button', { name: 'Draw at random' }).click();
	await expect(sample.getByText(/^Seed 271828,/)).toBeVisible();
	await expect(seed).toHaveValue('');
});

test('a result set aside from what to check stays aside on coming back, until brought back', async ({
	page
}) => {
	await open(page, 'queue');
	const main = page.getByRole('main');
	const choice = () => main.getByRole('checkbox', { name: /Bulletin, people and places/ }).first();
	const items = () => main.getByRole('button', { name: /^Open Bulletin, May 1957/ });
	await main.getByText('Results read').click();
	await expect(choice()).toBeChecked();
	const before = await items().count();
	await choice().uncheck();
	await expect(choice()).not.toBeChecked();
	await expect(items()).toHaveCount(before - 1);

	// Another pane, then back: the project kept it aside.
	await page.getByRole('button', { name: /^All documents/ }).click();
	await page.getByRole('button', { name: /^To check/ }).click();
	await expect(main.getByRole('heading', { level: 2, name: 'To check' })).toBeVisible();
	await main.getByText('Results read').click();
	await expect(choice()).not.toBeChecked();
	await expect(items()).toHaveCount(before - 1);

	// Brought back, it is read again.
	await choice().check();
	await expect(choice()).toBeChecked();
	await expect(items()).toHaveCount(before);
});
