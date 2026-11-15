// Records (#171) against the sample data (`?mock`), in both engines: an analysis turned into one
// row per entry in Prompts, its fields becoming the columns; and a directory's rows reviewed
// beside its pages, a cell corrected, a row added and removed, a page not read said so, and a
// page marked reviewed, all from the keyboard.

import { expect, test, type Page } from '@playwright/test';

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

async function primary(page: Page): Promise<'Meta' | 'Control'> {
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	return mac ? 'Meta' : 'Control';
}

test('a page’s rows are corrected, added, removed and reviewed from the keyboard', async ({
	page
}) => {
	await open(page, 'review-records');
	const main = page.getByRole('main');
	const rows = main.getByRole('region', { name: 'Rows on this page' });
	const table = rows.getByRole('table');
	await expect(table.getByRole('row')).toHaveCount(6);
	await expect(rows).toContainText('5 rows');
	// A ditto mark stands as printed.
	await expect(table.getByRole('textbox', { name: 'Address, row 4' })).toHaveValue('"');

	// A cell typed anew is saved when Enter is pressed, the model's rows kept apart.
	const name = table.getByRole('textbox', { name: 'Name, row 1' });
	await name.focus();
	await name.fill('Abadie (Jean)');
	await page.keyboard.press('Enter');
	await expect(main.getByRole('button', { name: 'Back to model output' })).toBeVisible();
	await expect(name).toHaveValue('Abadie (Jean)');

	// A row added after the second takes the focus; one removed leaves the rest in order.
	await table.getByRole('button', { name: 'Add a row after row 2' }).focus();
	await page.keyboard.press('Enter');
	await expect(table.getByRole('row')).toHaveCount(7);
	await expect(table.getByRole('textbox', { name: 'Name, row 3' })).toBeFocused();
	await page.keyboard.type('Ba (Ousmane)');
	await page.keyboard.press('Tab');
	await expect(table.getByRole('textbox', { name: 'Name, row 3' })).toHaveValue('Ba (Ousmane)');
	await table.getByRole('button', { name: 'Remove row 2' }).click();
	await expect(table.getByRole('row')).toHaveCount(6);
	await expect(table.getByRole('textbox', { name: 'Name, row 2' })).toHaveValue('Ba (Ousmane)');

	// Marked reviewed with the shortcut every review takes.
	await name.focus();
	await page.keyboard.press(`${await primary(page)}+Enter`);
	await expect(
		main.getByRole('button', { name: /Mark as not reviewed|Reviewed/ }).first()
	).toBeVisible();

	// The page cut off has no rows, and says so.
	await main.getByRole('button', { name: 'Next', exact: true }).click();
	await expect(rows).toContainText('could not be read');
	await expect(rows.getByRole('table')).toHaveCount(0);

	// The next page brings its own rows.
	await main.getByRole('button', { name: 'Next', exact: true }).click();
	await expect(rows.getByRole('textbox', { name: 'Name, row 1' })).toHaveValue('Diop (Amadou)');
});
