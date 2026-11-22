// What a model missed, added to an analysis's list answers by the researcher (#1119), against
// the sample data (`?mock`) in both engines: by pointer and by keyboard, a name already listed
// pointed out, several pasted at once, a field of choices offering only its choices, the added
// name marked as the researcher's, removed again and taken back with Undo. Elements are found
// by their accessible names, as a screen reader finds them.

import { expect, test, type Locator, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page): Promise<Locator> {
	await page.goto('/?mock&view=review-analysis');
	await page.waitForLoadState('networkidle');
	const answers = page.getByRole('main').getByRole('region', { name: 'Answers' });
	await expect(answers.locator('[data-field="places"]')).toContainText('Dakar');
	return answers;
}

test('names are added by pointer, a name already listed is pointed out, and Undo takes them back', async ({
	page
}) => {
	const answers = await open(page);
	const places = answers.locator('[data-field="places"]');
	const add = places.getByRole('button', { name: 'Add to “Places”' });
	await add.click();
	const field = places.getByRole('textbox', { name: /^Add to “Places”/ });
	await expect(field).toBeFocused();

	// Nothing typed: said, nothing added.
	await places.getByRole('button', { name: 'Add', exact: true }).click();
	await expect(places.getByRole('status')).toHaveText('Type a name to add.');

	await field.fill('Siguiri');
	await places.getByRole('button', { name: 'Add', exact: true }).click();
	const siguiri = places.getByRole('button', { name: 'Siguiri, your correction', exact: true });
	await expect(siguiri).toBeVisible();
	await expect(siguiri).toHaveClass(/yours/);
	// A name a model listed is not marked.
	await expect(places.getByRole('button', { name: 'Dakar', exact: true })).not.toHaveClass(/yours/);
	const toast = page.getByRole('status').filter({ hasText: 'added to the answers' });
	await expect(toast).toContainText('“Siguiri” added to the answers');
	// The field stays open and empty for the next one.
	await expect(field).toHaveValue('');
	await expect(field).toBeFocused();

	// Several pasted at once, a column's lines set between semicolons; the one already listed
	// (in another case) is not added twice.
	await field.evaluate((input) => {
		const data = new DataTransfer();
		data.setData('text/plain', 'siguiri\r\nKouroussa\nKankan\n');
		input.dispatchEvent(
			new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true })
		);
	});
	await expect(field).toHaveValue('siguiri; Kouroussa; Kankan');
	await field.press('Enter');
	await expect(places.getByRole('button', { name: 'Kouroussa, your correction' })).toBeVisible();
	await expect(places.getByRole('button', { name: 'Kankan, your correction' })).toBeVisible();
	await expect(places.getByRole('status')).toHaveText('Already in the list: “siguiri”');
	await expect(places.locator('[data-item]').filter({ hasText: /^Siguiri/i })).toHaveCount(1);
	await expect(toast).toContainText('“Kouroussa”, “Kankan” added to the answers');

	// Undo takes back the last ones; the first stays.
	await toast.getByRole('button', { name: 'Undo' }).click();
	await expect(places.locator('[data-item="Kouroussa"]')).toHaveCount(0);
	await expect(places.locator('[data-item="Kankan"]')).toHaveCount(0);
	await expect(siguiri).toBeVisible();
});

test('a name is added from the keyboard, Escape closes the field, and the name is removed again', async ({
	page
}) => {
	const answers = await open(page);
	const people = answers.locator('[data-field="people"]');
	// The names are one tab stop; Tab goes from them to Add. On macOS, WebKit's Tab stops at
	// fields only unless the system's keyboard navigation is on, and Option+Tab stops at every
	// control, buttons included.
	await people.locator('[data-item][tabindex="0"]').focus();
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	await page.keyboard.press(mac ? 'Alt+Tab' : 'Tab');
	const add = people.getByRole('button', { name: 'Add to “People”' });
	await expect(add).toBeFocused();
	await page.keyboard.press('Enter');
	const field = people.getByRole('textbox', { name: /^Add to “People”/ });
	await expect(field).toBeFocused();
	await page.keyboard.type('Almamy  Samory ');
	await page.keyboard.press('Enter');
	const samory = people.getByRole('button', { name: 'Almamy Samory, your correction' });
	await expect(samory).toBeVisible();
	await expect(field).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(field).toHaveCount(0);
	await expect(add).toBeFocused();

	// The name added is the researcher's correction like any other: its menu removes it.
	await samory.focus();
	await page.keyboard.press('Enter');
	const menu = page.getByRole('menu', { name: 'What to do with “Almamy Samory”' });
	await expect(menu.getByRole('menuitem', { name: 'Rename' })).toBeFocused();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	await expect(samory).toHaveCount(0);
	await expect(page.getByRole('status').filter({ hasText: 'removed' })).toContainText(
		'“Almamy Samory” removed from the answers'
	);
});

test('a field of choices offers only the choices not yet listed, and a choice is only removed', async ({
	page
}) => {
	const answers = await open(page);
	const themes = answers.locator('[data-field="themes"]');
	await themes.getByRole('button', { name: 'Add to “Themes”' }).click();
	const choice = themes.getByRole('combobox', { name: 'Choice to add to “Themes”' });
	await expect(choice).toBeFocused();
	// Education, Pilgrimage and Mosques are listed: Trade is the one choice left.
	await expect(choice.locator('option')).toHaveText(['Trade']);
	await choice.press('Enter');
	const trade = themes.getByRole('button', { name: 'Trade, your correction', exact: true });
	await expect(trade).toBeVisible();
	await expect(themes.getByText('Every choice is already listed.')).toBeVisible();
	await page.keyboard.press('Escape');
	await expect(themes.getByRole('button', { name: 'Add to “Themes”' })).toBeFocused();

	// A choice cannot be renamed, only removed.
	await trade.click();
	const menu = page.getByRole('menu', { name: 'What to do with “Trade”' });
	await expect(menu.getByRole('menuitem')).toHaveText(['Remove from the answers']);
	await menu.getByRole('menuitem', { name: 'Remove from the answers' }).click();
	await expect(trade).toHaveCount(0);
});

test('a name added by hand is found in the project search as written by the researcher', async ({
	page
}) => {
	const answers = await open(page);
	const places = answers.locator('[data-field="places"]');
	await places.getByRole('button', { name: 'Add to “Places”' }).click();
	await places.getByRole('textbox', { name: /^Add to “Places”/ }).fill('Siguiri');
	await places.getByRole('button', { name: 'Add', exact: true }).click();
	await expect(places.getByRole('button', { name: 'Siguiri, your correction' })).toBeVisible();

	await page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('button', { name: /^Missions in French West Africa/ })
		.click();
	await page
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Search' })
		.click();
	const search = page.getByRole('search');
	const main = page.getByRole('main');
	await main.getByRole('checkbox', { name: /^Also search what models wrote about them/ }).check();
	const box = search.getByRole('searchbox', { name: 'Words to find' });
	const result = main.locator('.result');

	await box.fill('Siguiri');
	await search.getByRole('button', { name: 'Search' }).click();
	// Added where both set-ups' names are shown together, it went into each one's answers.
	await expect(result).toHaveCount(2);
	for (const one of await result.all()) {
		await expect(one.locator('mark')).toHaveText('Siguiri');
		await expect(one.getByText('Written by you')).toBeVisible();
		await expect(one.getByText('Written by a model')).toHaveCount(0);
	}

	// A name the model listed in the same answers is still said to be the model's.
	await box.fill('Dakar');
	await search.getByRole('button', { name: 'Search' }).click();
	const analysis = result.filter({ hasText: 'Places' });
	await expect(analysis.first().getByText('Written by a model')).toBeVisible();
	await expect(analysis.first().getByText('Written by you')).toHaveCount(0);
});
