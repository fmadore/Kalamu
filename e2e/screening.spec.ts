// Which of a project's documents speak of the researcher's question, against the sample data
// (`?mock`), in both engines: "Which of these speak of my question?" offered for the documents
// chosen, its dialog's default path one box and one button, More choices closed, the documents
// with no text named and left out, the job started with the question over the frozen texts; the
// table filtered by the answer an analysis gave, Keep and Set aside offered only then and kept as
// tags, the analysis exported as one table; nothing of it shown in a project no analysis
// answered; and the same in French.

import { expect, test, type Locator, type Page } from '@playwright/test';
import type { NewJob } from '../src/lib/bindings/NewJob.js';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page, extra = ''): Promise<Locator> {
	await page.goto(`/?mock&view=project${extra}`);
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	await expect(main.getByRole('table')).toBeVisible();
	return main;
}

/** The rows of the documents' table, by their titles' buttons. */
function titles(main: Locator): Locator {
	return main.locator('tbody tr td:nth-child(2) button.link');
}

test('the dialog asks one question, leaves out what has no text, and starts the job', async ({
	page
}) => {
	const main = await open(page);
	await main.getByRole('checkbox', { name: 'Select all documents shown' }).check();
	const bar = main.getByRole('toolbar');
	// Keep and Set aside wait for the table to be filtered by an answer.
	await expect(bar.getByRole('button', { name: 'Keep', exact: true })).toHaveCount(0);
	await bar.getByRole('button', { name: 'Which of these speak of my question?' }).click();

	const dialog = page.getByRole('dialog', {
		name: 'Which of these documents speak of my question?'
	});
	const question = dialog.getByRole('textbox', { name: 'Your research question' });
	await expect(question).toBeFocused();
	await expect(dialog.getByRole('textbox')).toHaveCount(1);
	await expect(dialog.getByText('2 documents will be read.')).toBeVisible();
	await expect(dialog.getByText(/^Left out, not read into text yet: /)).toContainText(
		'lettre-gouverneur-1956'
	);
	await expect(dialog.getByText(/^Model: /)).toBeVisible();
	await expect(dialog.locator('details.more')).not.toHaveAttribute('open', '');
	const start = dialog.getByRole('button', { name: 'Start', exact: true });
	await expect(start).toBeDisabled();

	await question.fill('How associations dealt with the colonial administration');
	await expect(start).toBeEnabled();
	await start.click();
	await expect(dialog).toBeHidden();
	const created = await page.evaluate(() => window.__kalamuCreated as NewJob | undefined);
	expect(created?.task).toBe('analysis');
	expect(created?.name).toMatch(/^My question: How associations/);
	expect(created?.setups[0].preset_id).toBe('analysis.screening');
	expect(created?.setups[0].options.question).toBe(
		'How associations dealt with the colonial administration'
	);
	expect(created?.sources).toHaveLength(2);
	expect(created?.sources.every((s) => s.kind === 'text' && s.derived_from)).toBe(true);
});

test('a document too long for the model even in parts is left out, the others read', async ({
	page
}) => {
	const main = await open(page, '&screening-long');
	await main.getByRole('checkbox', { name: 'Select all documents shown' }).check();
	await main
		.getByRole('toolbar')
		.getByRole('button', { name: 'Which of these speak of my question?' })
		.click();
	const dialog = page.getByRole('dialog', {
		name: 'Which of these documents speak of my question?'
	});
	await expect(dialog.getByText('1 document will be read.')).toBeVisible();
	await expect(dialog.getByText(/^Left out, too long for .* even read in parts: /)).toBeVisible();
	await expect(dialog.getByText(/^Estimated cost: /)).toBeVisible();
	await dialog.getByRole('textbox', { name: 'Your research question' }).fill('Associations');
	await dialog.getByRole('button', { name: 'Start', exact: true }).click();
	await expect(dialog).toBeHidden();
	const created = await page.evaluate(() => window.__kalamuCreated as NewJob | undefined);
	expect(created?.sources).toHaveLength(1);
});

test('the table is filtered by an answer, kept or set aside, and exported', async ({ page }) => {
	const main = await open(page);
	await expect(titles(main)).toHaveCount(4);
	const filter = main.getByRole('combobox', { name: 'Filter by an answer' });
	await filter.selectOption({ label: 'Relevant (1)' });
	await expect(titles(main)).toHaveText(['Bulletin, May 1957']);

	await main.getByRole('checkbox', { name: 'Select all documents shown' }).check();
	await main.getByRole('toolbar').getByRole('button', { name: 'Keep', exact: true }).click();
	await expect(page.getByText('1 document tagged “Kept”')).toBeVisible();
	await expect(main.locator('tbody').getByRole('button', { name: 'Kept' })).toBeVisible();

	// Set aside takes the other tag's place.
	await main.getByRole('checkbox', { name: 'Select all documents shown' }).check();
	await main.getByRole('toolbar').getByRole('button', { name: 'Set aside', exact: true }).click();
	const chips = main.locator('tbody .chip.tag');
	await expect(chips.filter({ hasText: 'Set aside' })).toHaveCount(1);
	await expect(chips.filter({ hasText: 'Kept' })).toHaveCount(0);

	await main.getByRole('button', { name: 'Export this table' }).click();
	await expect(page.getByText('Table exported, as a spreadsheet and as CSV.')).toBeVisible();
	const exported = await page.evaluate(() => window.__kalamuExportedAnswers);
	expect(exported?.job).toBe('00000000-0000-0000-0000-0000000000s1');
	expect(exported?.path).toMatch(/\.xlsx$/);

	await filter.selectOption({ label: 'Any answer' });
	await expect(titles(main)).toHaveCount(4);
	await expect(main.getByRole('button', { name: 'Export this table' })).toHaveCount(0);
});

test('each row shows its answer, reason and passages, kept or set aside by keyboard', async ({
	page
}) => {
	const main = await open(page);
	// Nothing of it before the table is filtered.
	await expect(main.getByRole('button', { name: /^Keep “/ })).toHaveCount(0);
	await main
		.getByRole('combobox', { name: 'Filter by an answer' })
		.selectOption({ label: 'Relevant (1)' });
	const row = main.locator('tbody tr').filter({ hasText: 'Bulletin, May 1957' });
	await expect(row.getByText('Relevant', { exact: true })).toBeVisible();
	await expect(
		row.getByText('The bulletin reports the association asking the administration')
	).toBeVisible();
	const passages = row.getByRole('list', { name: 'Passages quoted from “Bulletin, May 1957”' });
	await expect(passages.getByRole('listitem')).toHaveCount(2);
	await expect(passages.getByRole('listitem').first()).toContainText('Page 2');

	// Keep reached from the keyboard: the button before it, then Tab, then Enter.
	const keep = row.getByRole('button', { name: 'Keep “Bulletin, May 1957”' });
	await row.getByRole('button', { name: 'Bulletin, May 1957', exact: true }).focus();
	for (let i = 0; i < 6 && !(await keep.evaluate((b) => b === document.activeElement)); i++)
		await page.keyboard.press('Tab');
	await expect(keep).toBeFocused();
	await page.keyboard.press('Enter');
	await expect(page.getByText('1 document tagged “Kept”')).toBeVisible();
	await expect(keep).toHaveAttribute('aria-pressed', 'true');
	// Set aside takes its place; pressed again, it is taken off.
	const aside = row.getByRole('button', { name: 'Set aside “Bulletin, May 1957”' });
	await aside.click();
	await expect(aside).toHaveAttribute('aria-pressed', 'true');
	await expect(keep).toHaveAttribute('aria-pressed', 'false');
	await aside.click();
	await expect(page.getByText('Tag “Set aside” taken off')).toBeVisible();
	await expect(aside).toHaveAttribute('aria-pressed', 'false');
});

test('the question is asked again of the documents added since, as it was asked', async ({
	page
}) => {
	// Every document read against it: nothing to ask again.
	let main = await open(page);
	await expect(
		main.getByRole('button', { name: 'Read the new documents against this question' })
	).toHaveCount(0);

	main = await open(page, '&screening-new');
	await expect(
		main.getByText(/^1 document has not been read against “How Catholic and Muslim/)
	).toBeVisible();
	await main.getByRole('button', { name: 'Read the new documents against this question' }).click();
	const dialog = page.getByRole('dialog', {
		name: 'Which of these documents speak of my question?'
	});
	await expect(dialog.getByRole('textbox', { name: 'Your research question' })).toHaveValue(
		'How Catholic and Muslim associations dealt with the colonial administration'
	);
	await expect(dialog.getByText('1 document will be read.')).toBeVisible();
	await dialog.getByRole('button', { name: 'Start', exact: true }).click();
	await expect(dialog).toBeHidden();
	const created = await page.evaluate(() => window.__kalamuCreated as NewJob | undefined);
	expect(created?.sources).toHaveLength(1);
	expect(created?.setups[0]).toMatchObject({
		provider: 'gemini',
		model: 'gemini-4-carbon',
		preset_id: 'analysis.screening',
		options: {
			question: 'How Catholic and Muslim associations dealt with the colonial administration'
		}
	});
	expect(created?.settings.languages).toEqual({ mode: 'explicit', languages: ['fr'] });
});

test('a project no analysis answered shows no filter by an answer', async ({ page }) => {
	const main = await open(page, '&no-screening');
	await expect(main.getByRole('combobox', { name: 'Filter by an answer' })).toHaveCount(0);
});

test.describe('in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the filter shows the answers in French and the dialog asks in French', async ({ page }) => {
		const main = await open(page);
		const filter = main.getByRole('combobox', { name: 'Filtrer selon une réponse' });
		await expect(filter.locator('option')).toHaveText([
			'Toutes les réponses',
			'Pertinent (1)',
			'En partie (1)',
			'Non pertinent (1)',
			'Incertain (1)'
		]);
		await filter.selectOption({ label: 'Incertain (1)' });
		await expect(titles(main)).toHaveText(['Interview with the imam']);
		await main.getByRole('checkbox', { name: 'Sélectionner tous les documents affichés' }).check();
		const bar = main.getByRole('toolbar');
		await expect(bar.getByRole('button', { name: 'Garder', exact: true })).toBeVisible();
		await expect(bar.getByRole('button', { name: 'Écarter', exact: true })).toBeVisible();
		await bar.getByRole('button', { name: 'Lesquels parlent de ma question ?' }).click();
		const dialog = page.getByRole('dialog', {
			name: 'Lesquels de ces documents parlent de ma question ?'
		});
		await expect(
			dialog.getByRole('textbox', { name: 'Votre question de recherche' })
		).toBeFocused();
		await expect(dialog.getByText('1 document sera lu.')).toBeVisible();
		await expect(dialog.getByRole('button', { name: 'Lancer', exact: true })).toBeDisabled();
	});
});
