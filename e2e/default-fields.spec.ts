// The default prompts' fields in the interface language (#210), against the sample data
// (`?mock`), in both engines: the interview's answers and a directory's columns named in French
// in the French interface and in English in the English one, from the same job, whose fields
// are stored in English.

import { expect, test, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

test('the interview’s fields and a directory’s columns are named in English', async ({ page }) => {
	await open(page, 'review-interview');
	const answers = page.getByRole('main').getByRole('region', { name: 'Answers' });
	for (const name of ['Summary', 'Themes', 'Keywords', 'Quotations']) {
		await expect(answers.getByRole('heading', { name, exact: true })).toBeVisible();
	}
	await expect(answers.getByRole('group', { name: 'Themes', exact: true })).toBeVisible();

	await open(page, 'review-records');
	const main = page.getByRole('main');
	for (const name of ['Name', 'Occupation', 'Address']) {
		await expect(main.getByRole('columnheader', { name, exact: true })).toBeVisible();
	}
});

test.describe('in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the same fields are named in French', async ({ page }) => {
		await open(page, 'review-interview');
		const answers = page.getByRole('main').getByRole('region', { name: 'Réponses' });
		for (const name of ['Résumé', 'Thèmes', 'Mots-clés', 'Citations']) {
			await expect(answers.getByRole('heading', { name, exact: true })).toBeVisible();
		}
		await expect(answers.getByRole('heading', { name: 'Themes' })).toHaveCount(0);
		await expect(answers.getByRole('group', { name: 'Thèmes', exact: true })).toBeVisible();
		// The prompt's name in the review's header, stored in English, is shown in French too.
		const header = page.getByRole('main');
		await expect(header.getByText(/^Entretien\s:\sthèmes et citations$/)).toBeVisible();
		await expect(header.getByText('Interview: themes and quotations')).toHaveCount(0);

		await open(page, 'review-records');
		const main = page.getByRole('main');
		for (const name of ['Nom', 'Profession', 'Adresse']) {
			await expect(main.getByRole('columnheader', { name, exact: true })).toBeVisible();
		}
		await expect(main.getByRole('columnheader', { name: 'Occupation' })).toHaveCount(0);
	});
});
