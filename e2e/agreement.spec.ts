// Agreement between models over a whole analysis, on the job's page, against the sample data
// (`?mock`), in both engines: a real table, found by its name and headers, one row per field
// compared, and kappa explained; a job with one model, or that reads text, shows none.

import { expect, test } from '@playwright/test';

test('a job comparing two models on an analysis shows how far they agree, field by field', async ({
	page
}) => {
	await page.goto('/?mock&view=jobs-analysis');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	const table = main.getByRole('table', { name: 'Agreement between models' });
	await expect(table).toBeVisible();

	for (const header of ['Field', 'Measured by', 'Agreement', 'Kappa', 'Answers compared']) {
		await expect(table.getByRole('columnheader', { name: header, exact: true })).toBeVisible();
	}
	// Two models: no column to name the pair.
	await expect(table.getByRole('columnheader', { name: 'Models' })).toHaveCount(0);

	// The free-text summary is not compared; the lists and the choice are, in the analysis's order.
	const fields = table.getByRole('rowheader');
	await expect(fields).toHaveText(['People', 'Places', 'Tone']);

	const tone = table
		.getByRole('row')
		.filter({ has: page.getByRole('rowheader', { name: 'Tone' }) });
	await expect(tone.getByRole('cell')).toHaveText(['Same choice', '0%', '0.00', '1']);
	const places = table
		.getByRole('row')
		.filter({ has: page.getByRole('rowheader', { name: 'Places' }) });
	await expect(places.getByRole('cell')).toHaveText([
		'Items in common',
		'66.7%',
		'Does not apply',
		'1'
	]);

	await expect(
		main.getByText(/Kappa sets aside the agreement two models would reach/)
	).toBeVisible();
	await expect(
		main.getByRole('button', { name: /Agreement between models \(kappa\)/ })
	).toBeVisible();
});

test('a job that reads text shows no agreement table', async ({ page }) => {
	await page.goto('/?mock&view=jobs');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { level: 1 })).toBeVisible();
	await expect(main.getByRole('table', { name: 'Agreement between models' })).toHaveCount(0);
});
