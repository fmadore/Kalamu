// What the review of 9 November (#146, #149) asked to be done without a pointer, checked with
// the keyboard against the sample data (`?mock`), in both engines: a text page reviewed,
// corrected and marked as reviewed; an analysis's name located, renamed and removed; a failure
// that stays until dismissed; Back to where a review was opened from, as it was left. Elements
// are found by their accessible names, as a screen reader finds them.

import { expect, test, type Locator, type Page } from '@playwright/test';

// WebKit opens the dev server's pages slowly while other workers start.
test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

/** The key the window's shortcuts go with where the page runs (see interactions.spec.ts). */
async function primary(page: Page): Promise<'Meta' | 'Control'> {
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	return mac ? 'Meta' : 'Control';
}

/** Presses Tab until `target` has the focus, as a researcher would, failing after `most`. */
async function tabTo(page: Page, target: Locator, most = 60): Promise<void> {
	for (let i = 0; i < most; i++) {
		if (await target.evaluate((el) => el === document.activeElement)) return;
		await page.keyboard.press('Tab');
	}
	await expect(target).toBeFocused();
}

test('a text page is reviewed, corrected and marked as reviewed from the keyboard', async ({
	page
}) => {
	await open(page, 'review');
	const main = page.getByRole('main');
	const text = main.getByRole('region', { name: 'Reading text' });
	await expect(text).toContainText('deux cents fidèles');

	// The differences are one tab stop, reached from the page's start.
	const first = text.locator('[data-entry]').first();
	await main.getByRole('button', { name: 'Bulletin, May 1957' }).focus();
	await tabTo(page, text.locator('[data-entry][tabindex="0"]'));
	await expect(first).toBeFocused();

	// → steps to the next difference; Enter opens its readings and takes the focus into them.
	await page.keyboard.press('ArrowRight');
	const second = text.locator('[data-entry]').nth(1);
	await expect(second).toBeFocused();
	await page.keyboard.press('Enter');
	const readings = page.getByRole('dialog', { name: 'Choose the correct reading' });
	await expect(readings).toBeVisible();
	// The focus moves in once the readings are drawn: waited for, not read at once.
	await expect
		.poll(() => readings.evaluate((box) => box.contains(document.activeElement)))
		.toBe(true);

	// Escape closes them and gives the focus back to the words.
	await page.keyboard.press('Escape');
	await expect(readings).toBeHidden();
	await expect(second).toBeFocused();

	// F steps to the difference holding "deux"; its first reading taken from the keyboard.
	let found = false;
	for (let i = 0; i < 12 && !found; i++) {
		await page.keyboard.press('f');
		await expect(readings).toBeVisible();
		found = (await readings.textContent())?.includes('trois') ?? false;
	}
	expect(found, 'F reaches the difference on “deux”').toBe(true);
	const use = readings.getByRole('button', { name: /trois/ });
	await expect(use).toBeFocused();
	await page.keyboard.press('Enter');
	await expect(readings).toBeHidden();
	await expect(text).toContainText('trois cents fidèles');
	// The focus stays in the text, on the place corrected or the next difference.
	await expect
		.poll(() => text.evaluate((pane) => pane.contains(document.activeElement)))
		.toBe(true);

	// The window's shortcut marks it as not reviewed and as reviewed in turn (the sample's
	// first page starts reviewed): it ends reviewed.
	const toggle = main.getByRole('button', { name: /^(Mark as reviewed|Reviewed)\s*(Ctrl|⌘)/ });
	const key = await primary(page);
	if ((await toggle.getAttribute('aria-pressed')) === 'true') {
		await page.keyboard.press(`${key}+Enter`);
		await expect(toggle).toHaveAttribute('aria-pressed', 'false');
	}
	await page.keyboard.press(`${key}+Enter`);
	await expect(toggle).toHaveAttribute('aria-pressed', 'true');
	await expect(toggle).toHaveAccessibleName(/^Reviewed/);
});

/** The words a text file's source pane marks. */
function foundInSource(page: Page): Promise<string[]> {
	return page
		.getByRole('main')
		.getByRole('region', { name: 'Source', exact: true })
		.locator('mark')
		.allTextContents();
}

test('a name in an analysis is located, renamed and removed from the keyboard', async ({
	page
}) => {
	await open(page, 'review-analysis');
	const main = page.getByRole('main');
	const answers = main.getByRole('region', { name: 'Answers' });
	const places = answers.locator('[data-field="places"]');
	await expect(places).toContainText('Dakar');

	// Each answer's names are one tab stop; reaching one marks it in the source.
	const dakar = places.getByRole('button', { name: 'Dakar', exact: true });
	await main.getByRole('button', { name: 'Bulletin, people and places' }).focus();
	await tabTo(page, dakar);
	await expect.poll(() => foundInSource(page)).toContain('Dakar');

	// → goes to the next name, which the source then marks.
	await page.keyboard.press('ArrowRight');
	const bamako = places.getByRole('button', { name: 'Bamako', exact: true });
	await expect(bamako).toBeFocused();
	await expect.poll(() => foundInSource(page)).toContain('Bamako');

	// Enter opens its menu; Rename turns it into a field holding the name.
	await page.keyboard.press('Enter');
	const menu = page.getByRole('menu', { name: 'What to do with “Bamako”' });
	await expect(menu.getByRole('menuitem', { name: 'Rename' })).toBeFocused();
	await page.keyboard.press('Enter');
	const field = page.getByRole('textbox', { name: 'Rename' });
	await expect(field).toBeFocused();
	await page.keyboard.type('Bamako (Soudan)');
	await page.keyboard.press('Enter');
	const renamed = places.getByRole('button', { name: 'Bamako (Soudan)', exact: true });
	await expect(renamed).toBeVisible();
	await expect(renamed).toBeFocused();

	// The menu again, down to Remove: the name goes, the focus to the next one, and the toast
	// offers to put it back.
	await page.keyboard.press('Enter');
	await expect(page.getByRole('menu', { name: 'What to do with “Bamako (Soudan)”' })).toBeVisible();
	await page.keyboard.press('ArrowDown');
	await expect(page.getByRole('menuitem', { name: 'Remove from the answers' })).toBeFocused();
	await page.keyboard.press('Enter');
	await expect(renamed).toHaveCount(0);
	expect(await places.evaluate((f) => f.contains(document.activeElement))).toBe(true);
	const toast = page.getByRole('status').filter({ hasText: 'removed from the answers' });
	await expect(toast).toContainText('“Bamako (Soudan)” removed from the answers');

	// Undo, from the keyboard, puts the name back and the focus on it.
	await toast.getByRole('button', { name: 'Undo' }).focus();
	await page.keyboard.press('Enter');
	await expect(renamed).toBeVisible();
	await expect(renamed).toBeFocused();
});

/** Goes back from a review with the keyboard: ⌘[ on a Mac; elsewhere Alt ← turns the review's
 * pages, so the Back button is reached and pressed. */
async function backFromReview(page: Page): Promise<void> {
	if ((await primary(page)) === 'Meta') {
		await page.keyboard.press('Meta+BracketLeft');
		return;
	}
	await page.getByRole('main').getByRole('button', { name: 'Back', exact: true }).focus();
	await page.keyboard.press('Enter');
}

test('Back from a review returns to the project search it was opened from, as it was', async ({
	page
}) => {
	await open(page, 'project');
	const main = page.getByRole('main');
	await main
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Search' })
		.click();
	const words = main.getByRole('searchbox', { name: 'Words to find' });
	await words.fill('pelerinage');
	await page.keyboard.press('Enter');
	await expect(main.getByText('1 match in 1 document')).toBeVisible();
	const passage = main.getByRole('button', { name: /^Open Page 1 in the review/ });
	await passage.focus();
	await page.keyboard.press('Enter');
	await expect(main.getByRole('heading', { level: 1, name: 'bulletin-1957-05.pdf' })).toBeVisible();

	await backFromReview(page);
	await expect(main.getByRole('searchbox', { name: 'Words to find' })).toHaveValue('pelerinage');
	await expect(main.getByText('1 match in 1 document')).toBeVisible();
});

test('Back from a review returns to the list of what to check, as it was set', async ({ page }) => {
	await open(page, 'queue');
	const main = page.getByRole('main');
	const reviewed = main.getByRole('checkbox', { name: 'Include reviewed' });
	await reviewed.check();
	const item = main.getByRole('button', { name: 'Open Bulletin, May 1957, Page 3, in the review' });
	await item.focus();
	await page.keyboard.press('Enter');
	await expect(main.getByRole('heading', { level: 1, name: 'bulletin-1957-05.pdf' })).toBeVisible();

	await backFromReview(page);
	await expect(main.getByRole('heading', { level: 2, name: 'To check' })).toBeVisible();
	await expect(main.getByRole('checkbox', { name: 'Include reviewed' })).toBeChecked();
});

test('Back from a review returns to the document it was opened from', async ({ page }) => {
	await open(page, 'project');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Bulletin, May 1957', exact: true }).click();
	await expect(main.getByRole('heading', { level: 1, name: 'Bulletin, May 1957' })).toBeVisible();
	const review = main
		.getByRole('region', { name: 'Results' })
		.getByRole('button', { name: 'Review' });
	await review.first().focus();
	await page.keyboard.press('Enter');
	await expect(main.getByRole('heading', { level: 1, name: 'bulletin-1957-05.pdf' })).toBeVisible();

	await backFromReview(page);
	await expect(main.getByRole('heading', { level: 1, name: 'Bulletin, May 1957' })).toBeVisible();
	await expect(main.getByRole('region', { name: 'Results' })).toBeVisible();

	// Out of a review, Alt ← (⌘[ on a Mac) goes back too: to the project the document is in.
	await main.getByRole('heading', { level: 1 }).click();
	await page.keyboard.press(
		(await primary(page)) === 'Meta' ? 'Meta+BracketLeft' : 'Alt+ArrowLeft'
	);
	await expect(
		main.getByRole('heading', { level: 1, name: 'Missions in French West Africa' })
	).toBeVisible();
});

test('a failure stays until it is dismissed, in the region read out at once', async ({ page }) => {
	// Settings that could not be read were moved aside: said as a failure (src/lib/dev/mock.ts).
	await open(page, 'jobs&set-aside');
	const alert = page.getByRole('alert');
	await expect(alert).toContainText('settings');
	// Longer than a confirmation stays.
	await page.waitForTimeout(5000);
	await expect(alert).toContainText('settings');

	// Dismissed from the keyboard; the region itself stays, empty, for the next one, and the
	// focus goes back to where it was before it went to the message.
	const rename = page.getByRole('main').getByRole('button', { name: 'Rename' });
	await rename.focus();
	const dismiss = alert.getByRole('button', { name: 'Dismiss this message' });
	await dismiss.focus();
	await page.keyboard.press('Enter');
	await expect(alert).toBeEmpty();
	await expect(alert).toBeAttached();
	await expect(rename).toBeFocused();
});

test('Ctrl F (⌘F on a Mac) goes to the filter of Jobs, a project’s dictionary and the Glossary', async ({
	page
}) => {
	const key = await primary(page);
	const main = page.getByRole('main');
	const dictionary = () =>
		main
			.getByRole('navigation', { name: 'Collections' })
			.getByRole('button', { name: 'Dictionary' })
			.click();
	const fields: [string, string, (() => Promise<void>) | null][] = [
		['all-jobs', 'Search names and files', null],
		['project', 'Find a name', dictionary],
		['glossary', 'Find a word', null]
	];
	for (const [view, name, before] of fields) {
		await open(page, view);
		await before?.();
		const field = main.getByRole('textbox', { name }).or(main.getByRole('searchbox', { name }));
		// WebKit may take a while to draw a screen while other workers start.
		await expect(field).toBeVisible({ timeout: 15_000 });
		await main.getByRole('heading', { level: 1 }).first().click();
		await page.keyboard.press(`${key}+f`);
		await expect(field, `${view}: ${name}`).toBeFocused();
	}
});

test('the choices remembered on this computer are there again after a restart', async ({
	page
}) => {
	const main = page.getByRole('main');
	// The Jobs page's filters and sort.
	await open(page, 'all-jobs');
	await main.getByRole('combobox', { name: 'Task' }).selectOption({ label: 'Extract text' });
	await main.getByRole('combobox', { name: 'Sort by' }).selectOption({ label: 'Name' });
	// The pane a project was left on.
	await open(page, 'project');
	await main
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Dictionary' })
		.click();
	await expect(main.getByRole('textbox', { name: 'Find a name' })).toBeVisible();

	// A page loaded again is a new launch: what is kept is the webview's storage.
	await open(page, 'all-jobs');
	await expect(main.getByRole('combobox', { name: 'Task' }).locator('option:checked')).toHaveText(
		'Extract text'
	);
	await expect(
		main.getByRole('combobox', { name: 'Sort by' }).locator('option:checked')
	).toHaveText('Name');
	await open(page, 'project');
	await expect(main.getByRole('textbox', { name: 'Find a name' })).toBeVisible();
});

test('a window of 640 by 560 sets the job list above the work, with no sideways scrolling', async ({
	page
}) => {
	await page.setViewportSize({ width: 640, height: 560 });
	for (const view of ['jobs', 'review', 'project', 'new', 'settings']) {
		await open(page, view);
		const bar = await page.getByRole('navigation', { name: 'Jobs' }).boundingBox();
		const work = await page.getByRole('main').boundingBox();
		expect(bar && work, view).toBeTruthy();
		if (!bar || !work) continue;
		// One row above the work, folded, and the work across the window.
		expect(bar.y + bar.height, `${view}: the list ends above the work`).toBeLessThanOrEqual(
			work.y + 1
		);
		expect(bar.height, `${view}: the list folded to a row`).toBeLessThan(80);
		expect(work.width, `${view}: the work takes the width`).toBeGreaterThan(600);
		const sideways = await page.evaluate(
			() => document.documentElement.scrollWidth > document.documentElement.clientWidth
		);
		expect(sideways, `${view}: no sideways scrolling`).toBe(false);
	}
});

test('a newer release is told at the sidebar’s foot until hidden from the keyboard', async ({
	page
}) => {
	const sidebar = page.getByRole('navigation', { name: 'Jobs' });
	// A copy the stores do not update, with a newer version in the model list (`?release`).
	await open(page, 'jobs&release');
	await expect(sidebar.getByText(/^Kalamu [\d.]+ is available\./)).toBeVisible();
	await expect(
		sidebar.getByRole('link', { name: /^What’s new, and the download/ })
	).toHaveAttribute('href', /^https:\/\/github\.com\/fmadore\/Kalamu\/releases\/tag\/v[\d.]+$/);
	const hide = sidebar.getByRole('button', { name: 'Hide until the next version' });
	await hide.focus();
	await page.keyboard.press('Enter');
	await expect(sidebar.getByText(/is available\./)).toHaveCount(0);
	// The focus is not lost with the button: it stays in the sidebar.
	expect(await sidebar.evaluate((nav) => nav.contains(document.activeElement))).toBe(true);

	// Hidden for that version across a restart.
	await open(page, 'jobs&release');
	await expect(sidebar.getByRole('button', { name: /^Settings/ })).toBeVisible();
	await expect(sidebar.getByText(/is available\./)).toHaveCount(0);
});
