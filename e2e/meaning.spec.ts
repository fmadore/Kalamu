// The search by meaning and the map, against the sample data (`?mock`), in both engines: a model
// chosen, the consent asked before anything is sent, the estimate, a preparation followed and a
// question whose passage opens the review; coverage said with the results; the map's list and
// points reached by keyboard; and the screens with no model or too few documents. Elements are
// found by their accessible names, as a screen reader finds them.

import { expect, test, type Page } from '@playwright/test';

async function open(page: Page, query: string): Promise<void> {
	await page.goto(`/?mock&${query}`);
	await page.waitForLoadState('networkidle');
}

async function meaning(page: Page) {
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Meaning', exact: true }).click();
	await expect(main.getByRole('button', { name: 'Meaning', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);
	return main;
}

test('a cloud model asks first, shows what preparing will send, and a passage found by meaning opens the review', async ({
	page
}) => {
	await open(page, 'view=search');
	const main = await meaning(page);
	const setup = main.getByRole('region', { name: 'Set-up' });
	// Nothing prepared: the question waits.
	await expect(main.getByRole('searchbox', { name: 'Question or description' })).toBeDisabled();

	await setup.getByRole('combobox', { name: 'Model' }).selectOption({ label: 'Gemini Embedding' });
	await expect(setup).toContainText('Send this project’s text to');
	await expect(setup).toContainText('each question you type in the search by meaning is sent');
	await expect(setup.getByRole('button', { name: 'Prepare the text' })).toHaveCount(0);

	await setup.getByRole('button', { name: 'Agree to send it' }).click();
	await expect(setup).toContainText('What preparing will do');
	await expect(setup).toContainText('passages to send');
	await expect(setup).toContainText('An estimate');
	await setup.getByRole('button', { name: 'Prepare the text' }).click();
	await expect(setup.getByRole('button', { name: 'Pause' })).toBeVisible();
	// The preparation goes on to the end, and the set-up folds away.
	await expect(main.getByRole('button', { name: 'Show the set-up' })).toBeVisible({
		timeout: 15_000
	});

	const box = main.getByRole('searchbox', { name: 'Question or description' });
	await box.fill('the mosque of Bamako and its imam');
	await main.getByRole('search').getByRole('button', { name: 'Search' }).click();
	await expect(main).toContainText(/closest documents? of \d+ compared/);
	await expect(main).toContainText(/Compared the text of \d+ of \d+ documents/);
	await expect(main).toContainText('A passage not listed does not mean');
	await expect(main).toContainText('It is not a probability');

	await main.getByRole('button', { name: 'Open Page 2 in the review' }).click();
	await expect(page.getByRole('main').getByRole('region', { name: 'Reading text' })).toContainText(
		'mosquée de Bamako'
	);
});

test('a preparation can be paused, and the record keeps what it cost', async ({ page }) => {
	await open(page, 'view=search');
	const main = await meaning(page);
	const setup = main.getByRole('region', { name: 'Set-up' });
	await setup.getByRole('combobox', { name: 'Model' }).selectOption({ label: 'Gemini Embedding' });
	await setup.getByRole('button', { name: 'Agree to send it' }).click();
	await setup.getByRole('button', { name: 'Prepare the text' }).click();
	await setup.getByRole('button', { name: 'Pause' }).click();
	await expect(setup.getByRole('button', { name: 'Continue' })).toBeVisible();
	await setup.getByText('Preparations').click();
	await expect(setup).toContainText('Paused');
	await expect(setup).toContainText('Recorded for this project so far');
});

test('a model on this computer not yet downloaded points to Offline models', async ({ page }) => {
	await open(page, 'view=search');
	const main = await meaning(page);
	const setup = main.getByRole('region', { name: 'Set-up' });
	await setup.getByRole('combobox', { name: 'Model' }).selectOption({ label: 'EmbeddingGemma' });
	await expect(setup).toContainText('On this computer: nothing is sent');
	await expect(setup).not.toContainText('Send this project’s text');
	await setup.getByRole('button', { name: 'Open Offline models' }).click();
	await expect(page.getByRole('main')).toContainText('Offline models');
});

test('the map’s list and points are reached by keyboard, each showing its nearest documents', async ({
	page
}) => {
	await open(page, 'view=map&docs=12&meaning=ready');
	const main = page.getByRole('main');
	await expect(main).toContainText('Positions are approximate and the axes mean nothing');
	await expect(main).toContainText('A long document’s point is the average of everything in it');
	await expect(main).toContainText(/11 of 12 documents placed/);
	await expect(main).toContainText('1 with nothing read yet');

	// The list: a document chosen with the keyboard shows the documents nearest to it.
	const listed = main.getByRole('button', { name: /^journal-1958-03/ }).last();
	await listed.focus();
	await page.keyboard.press('Enter');
	const nearest = main.getByRole('list', { name: 'Nearest documents' });
	await expect(nearest.getByRole('listitem')).toHaveCount(10);
	await expect(main.getByRole('button', { name: 'Open the document' })).toBeVisible();

	// The points: one reached with Tab, the arrows moving on, Enter opening the document.
	const map = main.getByRole('group', { name: /Map of the documents/ });
	const point = map.getByRole('button', { name: /^journal-1958-03/ });
	await expect(point).toHaveAttribute('tabindex', '0');
	await point.focus();
	for (const key of ['ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight']) {
		await page.keyboard.press(key);
	}
	const focused = map.locator('[role="button"]:focus');
	await expect(focused).toHaveCount(1);
	const title = (await focused.getAttribute('aria-label'))?.split(',')[0] ?? '';
	await expect(main.getByRole('region', { name: 'Selected' })).toContainText(title);
	await page.keyboard.press('Enter');
	await expect(page.getByRole('main').getByRole('heading', { name: title })).toBeVisible();
});

test('the map says why it is not drawn: no model, or too few documents placed', async ({
	page
}) => {
	await open(page, 'view=map');
	const main = page.getByRole('main');
	await expect(main).toContainText('Choose a model and prepare the text in Search');
	await main.getByRole('button', { name: 'Open the search by meaning' }).click();
	await expect(main.getByRole('button', { name: 'Meaning', exact: true })).toHaveAttribute(
		'aria-pressed',
		'true'
	);

	await open(page, 'view=map&meaning=ready');
	await expect(page.getByRole('main')).toContainText(
		'A map needs at least five documents placed, and only 3 are'
	);
	await expect(
		page.getByRole('main').getByRole('group', { name: /Map of the documents/ })
	).toHaveCount(0);
});
