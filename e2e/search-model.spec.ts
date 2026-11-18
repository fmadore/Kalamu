// The search by meaning's model on this computer, in Settings › Offline models, against the
// sample data (`?mock`): listed under a heading of its own, downloaded, then removable.

import { expect, test } from '@playwright/test';

test('the search by meaning model is listed apart and its download runs', async ({ page }) => {
	// `idle`: no other download running, so this one can start.
	await page.goto('/?mock&idle&view=settings');
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await main.getByRole('tab', { name: 'Offline models' }).click();

	const heading = main.getByRole('heading', { name: 'Search by meaning', level: 3 });
	await heading.scrollIntoViewIfNeeded();
	await expect(heading).toBeVisible();
	const row = main.locator('.model', { hasText: 'EmbeddingGemma 2' });
	await expect(row).toContainText('Not downloaded');
	await expect(row).toContainText('310 MB');

	await row.getByRole('button', { name: 'Download', exact: true }).click();
	await expect(row).toContainText('Downloading…');
	await expect(row.getByRole('button', { name: 'Cancel', exact: true })).toBeVisible();

	// Done: downloaded, and it can be removed again.
	await expect(row).toContainText('Downloaded', { timeout: 5000 });
	await expect(row.getByRole('button', { name: 'Remove', exact: true })).toBeEnabled();
});
