// Where Kalamu opens, against the sample data (`?mock`) in both engines: on Settings with a
// welcome while there is neither a key nor anything installed offline, and on New job as soon as
// there is either, so a researcher who works offline is never sent to the API keys.

import { expect, test, type Page } from '@playwright/test';

async function launch(page: Page, query: string): Promise<void> {
	await page.goto(`/?mock&${query}`);
	await page.waitForLoadState('networkidle');
}

const welcome = (page: Page) => page.getByRole('region', { name: 'Welcome to Kalamu' });

test('with no key and nothing offline, Settings opens on a welcome offering both ways', async ({
	page
}) => {
	await launch(page, 'keys=none&offline=none');
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
	await expect(welcome(page)).toBeVisible();
	await expect(welcome(page).getByRole('link', { name: /Read the guide/ })).toHaveAttribute(
		'href',
		'https://kalamu.frederickmadore.com/guide/'
	);
	// Nothing installed: the sidebar does not say models are ready.
	await expect(page.getByText('Models ready on this computer')).toHaveCount(0);

	// Each way opens its tab.
	await welcome(page).getByRole('button', { name: 'Use a model on this computer' }).click();
	await expect(main.getByRole('tab', { name: 'Offline models' })).toHaveAttribute(
		'aria-selected',
		'true'
	);
	await welcome(page).getByRole('button', { name: 'Add a key from an online provider' }).click();
	await expect(main.getByRole('tab', { name: 'API keys' })).toHaveAttribute(
		'aria-selected',
		'true'
	);

	// A key saved: the welcome goes by itself.
	const field = main.getByRole('textbox', { name: /Paste the key/ }).first();
	await field.fill('sk-test');
	await field.locator('xpath=..').getByRole('button', { name: 'Save', exact: true }).click();
	await expect(welcome(page)).toBeHidden();
});

test('with no key and one offline model, New job opens and the sidebar says so', async ({
	page
}) => {
	await launch(page, 'keys=none&offline=one');
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { name: 'New job', level: 1 })).toBeVisible();
	await expect(page.getByText('Models ready on this computer, no internet needed')).toBeVisible();
	await expect(welcome(page)).toHaveCount(0);
});

test('with a key saved, New job opens and Settings shows no welcome', async ({ page }) => {
	await launch(page, 'offline=none');
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { name: 'New job', level: 1 })).toBeVisible();
	await page.getByRole('button', { name: 'Settings', exact: true }).click();
	await expect(main.getByRole('heading', { name: 'Settings', level: 1 })).toBeVisible();
	await expect(welcome(page)).toHaveCount(0);
});

test('the guide opens in the interface language', async ({ page }) => {
	await launch(page, 'view=glossary');
	await expect(
		page.getByRole('main').getByRole('link', { name: /Read the guide/ })
	).toHaveAttribute('href', 'https://kalamu.frederickmadore.com/guide/');
});

test.describe('in French', () => {
	test.use({ locale: 'fr-FR' });
	test('the guide opens in French', async ({ page }) => {
		await launch(page, 'view=glossary');
		await expect(
			page.getByRole('main').getByRole('link', { name: /Lire le guide/ })
		).toHaveAttribute('href', 'https://kalamu.frederickmadore.com/guide/fr/');
	});
});
