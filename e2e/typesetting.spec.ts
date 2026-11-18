// Marks that are typesetting, against the sample data (`?mock`), in both engines: a dash that
// stands for an empty value is seen, and a screen reader hears what it stands for instead.

import { expect, test } from '@playwright/test';

test('the Jobs page’s empty cost shows a dash and is read out as none', async ({ page }) => {
	await page.goto('/?mock&view=all-jobs');
	await page.waitForLoadState('networkidle');
	// A correction of existing text, which nothing was paid for.
	const row = page.getByRole('main').getByRole('row', { name: /Fatwas, recueil Diallo/ });
	const cost = row.locator('td.cost');
	await expect(cost.locator('[aria-hidden="true"]')).toHaveText('–');
	await expect(cost.locator('.sr-only')).toHaveText('None');
});
