// Analysing an interview, against the sample data (`?mock`), in both engines: "Analyse this
// interview" offered for a transcription only; its dialog's default path one box and one button,
// with More choices closed; the job started with the researcher's question; the review's cards,
// a quotation played in the transcript at its turn or, without a recording, shown in the text;
// a quotation, an entry and an entry's name changed and put back from the keyboard; the line
// saying what was left out; and the same in French. A description of the research kept for the
// next interviews: not offered on first use, saved as a prompt of the researcher's own that
// Prompts lists and the dialog offers again, chosen to start the job with its prompt, written
// in Prompts as well; and in French. A transcript too long to be read at once: said to be read in
// parts beside the cost; one too long even so (one long turn): said in place of the cost, Start
// disabled until another model is chosen; and in French. A long interview read in parts: the
// review says so in one line, names the part not read by its minutes and offers to read it
// again. A passage selected in the transcript added as the researcher's quotation from its
// menu, marked as theirs, and taken back; and in French.

import { expect, test, type Locator, type Page } from '@playwright/test';
import type { NewJob } from '../src/lib/bindings/NewJob.js';

test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

/** Opens "Use this text in a new job" on a job's page. */
async function useText(page: Page): Promise<Locator> {
	const main = page.getByRole('main');
	await main.locator('summary', { hasText: 'Use this text in a new job' }).click();
	return main.locator('.use-text .menu');
}

/** The answers pane of the sample interview's review, once its cards are drawn. */
async function answers(page: Page, extra = ''): Promise<Locator> {
	await open(page, `review-interview${extra}`);
	const pane = page.getByRole('main').getByRole('region', { name: 'Answers' });
	await expect(pane.getByRole('heading', { name: 'Apprendre en regardant' })).toBeVisible();
	return pane;
}

test('the interview is offered for a transcription, not for a text read from a scan', async ({
	page
}) => {
	await open(page, 'jobs');
	let menu = await useText(page);
	await expect(menu.getByRole('button', { name: /^Analyse it/ })).toBeVisible();
	await expect(menu.getByRole('button', { name: /Analyse this interview/ })).toHaveCount(0);

	await open(page, 'jobs-audio');
	menu = await useText(page);
	await expect(menu.getByRole('button', { name: /Analyse this interview/ })).toBeVisible();
	await expect(menu.getByRole('button', { name: /^Analyse it/ })).toBeVisible();
});

test('the dialog asks one question with one button, and starts the job with it', async ({
	page
}) => {
	await open(page, 'jobs-audio');
	const menu = await useText(page);
	await menu.getByRole('button', { name: /Analyse this interview/ }).click();

	const dialog = page.getByRole('dialog', { name: 'Analyse this interview' });
	const question = dialog.getByRole('textbox', { name: 'What is your research about?' });
	// The box has the focus, says what it is for, and is the only one.
	await expect(question).toBeFocused();
	await expect(question).toHaveAccessibleDescription(/You can leave it empty/);
	await expect(dialog.getByRole('textbox')).toHaveCount(1);
	await expect(dialog.getByText(/^Model: /)).toBeVisible();
	// More choices is closed: no other model, no language, until it is opened.
	const more = dialog.locator('details.more');
	await expect(more).not.toHaveAttribute('open', '');
	await expect(dialog.getByRole('combobox', { name: /^Model/ })).toBeHidden();
	await expect(dialog.getByRole('button', { name: 'Analyse', exact: true })).toBeEnabled();
	await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeVisible();

	await more.locator('summary').click();
	await expect(dialog.getByRole('combobox', { name: /^Model/ })).toBeVisible();
	await expect(dialog.getByRole('radio', { name: 'Same as the document' })).toBeChecked();
	await more.locator('summary').click();

	await question.fill('How women in crafts learn and finance their trade.');
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	const created = await page.evaluate(() => window.__kalamuCreated as NewJob | undefined);
	expect(created?.task).toBe('analysis');
	expect(created?.name).toMatch(/^Interview analysis: /);
	expect(created?.setups).toHaveLength(1);
	expect(created?.setups[0].preset_id).toBe('analysis.interview');
	expect(created?.setups[0].options.question).toBe(
		'How women in crafts learn and finance their trade.'
	);
	// The text sent is the frozen transcript, and the job's page opens.
	expect(created?.sources[0].derived_from).toBeTruthy();
	await expect(page.getByRole('main').getByRole('heading', { level: 1 })).toContainText(
		'atelier-mariam-2024'
	);
});

test('the researcher’s own themes wait under More choices and go with the job', async ({
	page
}) => {
	await open(page, 'jobs-audio');
	const menu = await useText(page);
	await menu.getByRole('button', { name: /Analyse this interview/ }).click();
	const dialog = page.getByRole('dialog', { name: 'Analyse this interview' });
	const themes = dialog.getByRole('textbox', { name: 'Your own themes or interview guide' });
	// Not on the default path: the dialog stays one box and one button.
	await expect(themes).toBeHidden();
	await dialog.locator('details.more summary').click();
	await expect(themes).toHaveAccessibleDescription(/One theme per line/);
	await themes.fill('Savings groups\nHelp from the family');
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	const created = await page.evaluate(() => window.__kalamuCreated as NewJob | undefined);
	expect(created?.setups[0].options.themes).toBe('Savings groups\nHelp from the family');
	expect(created?.setups[0].options.question).toBeUndefined();
});

test('each theme says plainly whether it comes from the guide, and a guide theme not found is listed', async ({
	page
}) => {
	const pane = await answers(page);
	const tontine = pane.locator('[data-entry="1"]').first();
	await expect(tontine.locator('.origin')).toHaveText("Origin The researcher's guide");
	await expect(pane.locator('[data-entry="0"]').first().locator('.origin')).toHaveText(
		'Origin The interview'
	);
	await expect(pane.getByRole('heading', { name: 'Guide themes not found' })).toBeVisible();
	await expect(pane.locator('[data-field="guide_themes_not_found"]')).toContainText(
		'Le prix du tissu'
	);
});

/** Opens the dialog on the sample transcript made too long for the models on this computer to
 * read at once, or, `unbroken`, to read at all. */
async function longInterview(page: Page, useText: string, unbroken = true): Promise<Locator> {
	await open(page, `jobs-audio&interview-long${unbroken ? '&interview-unbroken' : ''}`);
	await page.getByRole('main').locator('summary', { hasText: useText }).click();
	await page.getByRole('button', { name: /Analyse this interview|Analyser cet entretien/ }).click();
	return page.getByRole('dialog');
}

/** Chooses a model under More choices, opened, from the menu of models. */
async function chooseModel(dialog: Locator, label: string, name: RegExp): Promise<void> {
	await dialog.getByRole('combobox', { name: new RegExp(`^${label}`) }).click();
	await dialog.page().getByRole('option', { name }).click();
}

test('a transcript too long to be read at once is read in parts, said beside the cost', async ({
	page
}) => {
	const dialog = await longInterview(page, 'Use this text in a new job', false);
	const start = dialog.getByRole('button', { name: 'Analyse', exact: true });
	await expect(dialog.getByText(/^This interview is long/)).toHaveCount(0);
	await dialog.locator('details.more summary').click();
	await chooseModel(dialog, 'Model', /^Gemma 4 E4B/);
	await expect(
		dialog.getByText('This interview is long: it will be read in 13 parts, then brought together.')
	).toBeVisible();
	await expect(dialog.getByText(/^Estimated cost: /)).toBeVisible();
	await expect(dialog.getByRole('alert')).toHaveCount(0);
	await expect(start).toBeEnabled();
	// A model that reads it all at once reads it whole.
	await chooseModel(dialog, 'Model', /^Gemini 4 Hydrogen/);
	await expect(dialog.getByText(/^This interview is long/)).toHaveCount(0);
});

test('a long interview read in parts says so, names the part not read, and reads it again', async ({
	page
}) => {
	const pane = await answers(page, '&interview-parts');
	await expect(
		pane.getByText('This interview was long: it was read in 3 parts, then brought together.')
	).toBeVisible();
	await expect(
		pane.getByText(
			'The part from 00:04:20 to 00:11:40 could not be read: its themes and quotations are missing.'
		)
	).toBeVisible();
	await expect(pane.getByText('The provider is overloaded. Try again later.')).toBeVisible();
	// The answers are those that brought the parts together, the cards as for any interview.
	await expect(pane.getByRole('heading', { name: 'La tontine comme appui' })).toBeVisible();
	await pane.getByRole('button', { name: 'Read it again' }).click();
	await expect(page.getByText('The part is being read again.')).toBeVisible();
});

test.describe('a long interview in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the review says it plainly, in French', async ({ page }) => {
		await open(page, 'review-interview&interview-parts');
		const pane = page.getByRole('main').getByRole('region', { name: 'Réponses' });
		await expect(
			pane.getByText(
				'Cet entretien était long : il a été lu en 3 morceaux, puis le tout a été rassemblé.'
			)
		).toBeVisible();
		await expect(pane.getByRole('button', { name: 'Le relire' })).toBeVisible();
	});
});

test('a transcript too long for the model chosen even in parts is said in place of the cost, and Start waits', async ({
	page
}) => {
	const dialog = await longInterview(page, 'Use this text in a new job');
	const start = dialog.getByRole('button', { name: 'Analyse', exact: true });
	// The default model reads it all: nothing new on the way.
	await expect(dialog.getByText(/^Estimated cost: /)).toBeVisible();
	await expect(dialog.getByRole('alert')).toHaveCount(0);
	await expect(start).toBeEnabled();

	await dialog.locator('details.more summary').click();
	await chooseModel(dialog, 'Model', /^Gemma 4 E4B/);
	await expect(dialog.getByRole('alert')).toHaveText(
		'This transcript is too long for Gemma 4 E4B: about 100,000 words, where it reads about 11,000 at once. Choose another model under More choices.'
	);
	await expect(dialog.getByText(/^Estimated cost: /)).toHaveCount(0);
	await expect(start).toBeDisabled();

	// Another model clears it.
	await chooseModel(dialog, 'Model', /^Gemini 4 Hydrogen/);
	await expect(dialog.getByRole('alert')).toHaveCount(0);
	await expect(dialog.getByText(/^Estimated cost: /)).toBeVisible();
	await expect(start).toBeEnabled();
});

test.describe('a transcript too long in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the sentence speaks French, its numbers written the French way', async ({ page }) => {
		const dialog = await longInterview(page, 'Utiliser ce texte dans un nouveau traitement');
		await dialog.locator('details.more summary').click();
		await chooseModel(dialog, 'Modèle', /^Gemma 4 E4B/);
		await expect(dialog.getByRole('alert')).toHaveText(
			/^Cette transcription est trop longue pour Gemma 4 E4B : environ 100\s000 mots, alors qu’il en lit environ 11\s000 à la fois\. Choisissez un autre modèle sous Plus de choix\.$/
		);
		await expect(dialog.getByRole('button', { name: 'Analyser', exact: true })).toBeDisabled();
	});
});

test('each theme is a card of quotations with who speaks and when, and what was left out', async ({
	page
}) => {
	const pane = await answers(page);
	const theme = pane.locator('[data-entry="1"]').first();
	await expect(theme.getByRole('heading', { name: 'La tontine comme appui' })).toBeVisible();
	await expect(theme.locator('blockquote')).toHaveText([
		'chaque semaine l’une de nous prend tout',
		'mon tour est venu au bon moment',
		'Quand une femme est malade, les autres tiennent son étal'
	]);
	await expect(theme.getByRole('button', { name: 'Listen: Mariam · 00:02:38' })).toHaveCount(2);
	await expect(pane).toContainText('1 quotation could not be found in the text and was left out.');
	// The passages worth quoting lead with their quotation, why it matters after it.
	await expect(pane.getByText('Sa façon de voir le métier, en une phrase.')).toBeVisible();
});

test('a quotation plays in the transcript, from its turn', async ({ page, browserName }) => {
	const pane = await answers(page);
	await pane.getByRole('button', { name: 'Listen: Mariam · 00:08:47' }).click();
	const main = page.getByRole('main');
	// The transcript's review, on its second piece of six minutes, which holds 00:08:47.
	await expect(main.getByRole('region', { name: 'Reading text' })).toContainText('Les téléphones.');
	await expect(main.locator('select.pages')).toHaveValue('1');
	// The player is sent to the turn: 2 minutes 47 into the piece, past the sample tone's end, so
	// the player stops at its end. Playwright's WebKit on Windows has no media engine to tell.
	if (browserName === 'chromium') {
		await expect
			.poll(() => main.locator('audio').evaluate((a: HTMLAudioElement) => a.currentTime))
			.toBeGreaterThan(2);
	}
});

test('without the recording, a quotation is shown where it stands in the text', async ({
	page
}) => {
	const pane = await answers(page, '&interview-gone');
	await expect(pane.getByRole('button', { name: /^Listen/ })).toHaveCount(0);
	// After an Arabic phrase and an emoji in the text: the place is counted in characters.
	await pane.getByRole('button', { name: 'Show in the text: Mariam · 00:08:47' }).click();
	await expect(page.locator('mark.quoted')).toHaveText(
		'les clientes m’envoient la photo d’un modèle'
	);
	await pane.getByRole('button', { name: 'Show in the text: Mariam · 00:02:38' }).last().click();
	await expect(page.locator('mark.quoted')).toHaveText('mon tour est venu au bon moment');
});

/** Selects `words` in the transcript beside the answers. */
async function select(page: Page, words: string): Promise<void> {
	await page.evaluate((words) => {
		const sheet = document.querySelector('.source .desk.document');
		const walker = document.createTreeWalker(sheet!, NodeFilter.SHOW_TEXT);
		for (let node = walker.nextNode(); node; node = walker.nextNode()) {
			const at = node.textContent?.indexOf(words) ?? -1;
			if (at < 0) continue;
			const range = document.createRange();
			range.setStart(node, at);
			range.setEnd(node, at + words.length);
			getSelection()?.removeAllRanges();
			getSelection()?.addRange(range);
			return;
		}
		throw new Error(`${words} is not in the transcript`);
	}, words);
}

test('a passage selected in the transcript is added as the researcher’s quotation, and taken back', async ({
	page
}) => {
	const pane = await answers(page, '&interview-gone');
	const first = pane.locator('[data-entry="0"]').first();
	await expect(first.locator('blockquote')).toHaveCount(2);
	await select(page, 'Les premiers mois, mes clientes étaient mes voisines');
	await page.keyboard.press('Shift+F10');
	const menu = page.getByRole('menu', { name: 'Add as a quotation' });
	await expect(menu).toBeVisible();
	// Every entry it can go to, each field's under its name.
	await expect(menu.getByRole('menuitem')).toHaveCount(5);
	await menu.getByRole('menuitem', { name: 'Apprendre en regardant' }).click();

	const quotes = first.locator('blockquote');
	await expect(quotes).toHaveCount(3);
	await expect(quotes.last()).toHaveText('Les premiers mois, mes clientes étaient mes voisines');
	// Marked as the researcher's, with the speaker and time of its turn.
	const mine = first.locator('[data-quote="2"]');
	await expect(mine).toHaveClass(/hand/);
	await expect(mine.getByText('Added by you')).toBeVisible();
	await expect(mine).toContainText('Mariam · 00:01:09');
	const toast = page
		.getByRole('status')
		.filter({ hasText: 'Quotation added to “Apprendre en regardant”' });
	await toast.getByRole('button', { name: 'Undo' }).click();
	await expect(quotes).toHaveCount(2);
});

test.describe('a quotation added in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the menu and the mark speak French', async ({ page }) => {
		await open(page, 'review-interview&interview-gone');
		const pane = page.getByRole('main').getByRole('region', { name: 'Réponses' });
		await expect(pane.getByRole('heading', { name: 'Apprendre en regardant' })).toBeVisible();
		await select(page, 'Les premiers mois, mes clientes étaient mes voisines');
		await page.keyboard.press('Shift+F10');
		const menu = page.getByRole('menu', { name: 'Ajouter comme citation' });
		await menu.getByRole('menuitem', { name: 'La tontine comme appui' }).click();
		await expect(pane.getByText('Ajoutée par vous')).toBeVisible();
	});
});

test('a quotation, an entry and an entry’s name are changed and put back from the keyboard', async ({
	page
}) => {
	const pane = await answers(page);
	const first = pane.locator('[data-entry="0"]').first();
	const quotes = first.locator('blockquote');
	await expect(quotes).toHaveCount(2);

	// Tab goes from a quotation's Listen to its Remove. On macOS, WebKit's Tab stops at fields
	// only unless the system's keyboard navigation is on, and Option+Tab stops at every control.
	await first.getByRole('button', { name: 'Listen: Mariam · 00:00:15' }).focus();
	const mac = await page.evaluate(() => /Macintosh|Mac OS X/.test(navigator.userAgent));
	await page.keyboard.press(mac ? 'Alt+Tab' : 'Tab');
	const remove = first.getByRole('button', {
		name: 'Remove the quotation “Pendant trois ans, je balayais l’atelier…”'
	});
	await expect(remove).toBeFocused();
	await page.keyboard.press('Enter');
	await expect(quotes).toHaveCount(1);
	// The focus goes to the quotation that took its place.
	await expect(first.getByRole('button', { name: 'Listen: Mariam · 00:06:11' })).toBeFocused();
	const toast = page.getByRole('status').filter({ hasText: 'Quotation removed from the answers' });
	await toast.getByRole('button', { name: 'Undo' }).focus();
	await page.keyboard.press('Enter');
	await expect(quotes).toHaveCount(2);
	await expect(first.getByRole('button', { name: 'Listen: Mariam · 00:00:15' })).toBeFocused();

	// An entry removed: the next one takes its place and the focus.
	await pane.getByRole('button', { name: 'Remove “La tontine comme appui”' }).focus();
	await page.keyboard.press('Enter');
	await expect(pane.getByRole('heading', { name: 'La tontine comme appui' })).toHaveCount(0);
	await expect(
		pane.getByRole('button', { name: 'Edit “Le téléphone et les clientes”' })
	).toBeFocused();
	const removed = page
		.getByRole('status')
		.filter({ hasText: '“La tontine comme appui” removed from the answers' });
	await removed.getByRole('button', { name: 'Undo' }).click();
	await expect(pane.getByRole('heading', { name: 'La tontine comme appui' })).toBeVisible();

	// A theme's name typed anew, saved, then put back.
	await pane.getByRole('button', { name: 'Edit “Apprendre en regardant”' }).focus();
	await page.keyboard.press('Enter');
	const name = pane.getByRole('textbox', { name: 'Name' });
	await expect(name).toBeFocused();
	await name.fill('Apprendre par l’observation');
	await pane.getByRole('button', { name: 'Save' }).click();
	await expect(pane.getByRole('heading', { name: 'Apprendre par l’observation' })).toBeVisible();
	await expect(
		pane.getByRole('button', { name: 'Edit “Apprendre par l’observation”' })
	).toBeFocused();
	await page.getByRole('status').getByRole('button', { name: 'Undo' }).click();
	await expect(pane.getByRole('heading', { name: 'Apprendre en regardant' })).toBeVisible();
	// No quotation's words can be typed.
	await expect(pane.locator('blockquote[contenteditable]')).toHaveCount(0);
});

test('two set-ups show their cards each under its siglum', async ({ page }) => {
	const pane = await answers(page, '&interview-two');
	const themes = pane.getByRole('group', { name: /^Themes · / });
	await expect(themes).toHaveCount(2);
	await expect(themes.nth(1)).toContainText('claude-sonnet-5-5');
	await expect(themes.nth(1).getByRole('heading', { name: 'Apprendre en regardant' })).toHaveCount(
		0
	);
});

test.describe('in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the dialog and the cards speak French', async ({ page }) => {
		const pane = await (async () => {
			await open(page, 'review-interview');
			const p = page.getByRole('main').getByRole('region', { name: 'Réponses' });
			await expect(p.getByRole('heading', { name: 'Apprendre en regardant' })).toBeVisible();
			return p;
		})();
		await expect(pane.getByRole('button', { name: 'Écouter : Mariam · 00:00:15' })).toBeVisible();
		await expect(pane).toContainText(
			'1 citation n’a pas été retrouvée dans le texte et a été laissée de côté.'
		);

		await open(page, 'jobs-audio');
		await page
			.getByRole('main')
			.locator('summary', { hasText: 'Utiliser ce texte dans un nouveau traitement' })
			.click();
		await page.getByRole('button', { name: /Analyser cet entretien/ }).click();
		const dialog = page.getByRole('dialog', { name: 'Analyser cet entretien' });
		await expect(
			dialog.getByRole('textbox', { name: 'Sur quoi porte votre recherche ?' })
		).toBeFocused();
		await expect(dialog.getByText('Plus de choix')).toBeVisible();
		await expect(dialog.getByRole('button', { name: 'Analyser', exact: true })).toBeVisible();
	});
});

/** Opens "Analyse this interview" from the audio job's page, already shown. */
async function interviewDialog(page: Page, name = 'Analyse this interview'): Promise<Locator> {
	const menu = await useText(page);
	await menu.getByRole('button', { name: /Analyse this interview/ }).click();
	return page.getByRole('dialog', { name });
}

const created = (page: Page) => page.evaluate(() => window.__kalamuCreated as NewJob | undefined);

/** Back on the audio job's page from anywhere in the app, without loading the page again (the
 * preview keeps prompts saved in memory). */
async function backToTheRecording(page: Page): Promise<void> {
	await page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('button', { name: /^Entretiens Ouagadougou/ })
		.first()
		.click();
	await expect(page.getByRole('main').getByRole('heading', { level: 1 })).toContainText(
		'Entretiens Ouagadougou'
	);
}

test('on first use nothing is saved to choose from, and keeping is offered once something is typed', async ({
	page
}) => {
	await open(page, 'jobs-audio');
	const dialog = await interviewDialog(page);
	await expect(dialog.getByRole('textbox', { name: 'What is your research about?' })).toBeFocused();
	await expect(dialog.getByRole('combobox', { name: 'Your research' })).toHaveCount(0);
	const keep = dialog.getByRole('checkbox', { name: 'Keep this for my next interviews' });
	await expect(keep).toHaveCount(0);

	await dialog.getByRole('textbox').fill('How traders find credit.');
	await expect(keep).not.toBeChecked();
	await expect(dialog.getByRole('textbox')).toHaveCount(1);
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	// Not kept: the ready-made prompt, with the question typed.
	expect((await created(page))?.setups[0].preset_id).toBe('analysis.interview');
});

test('a description kept for the next interviews is a prompt in Prompts and is offered again', async ({
	page
}) => {
	await open(page, 'jobs-audio');
	let dialog = await interviewDialog(page);
	const words = 'How women in crafts learn and finance their trade.';
	await dialog.getByRole('textbox', { name: 'What is your research about?' }).fill(words);
	await dialog.getByRole('checkbox', { name: 'Keep this for my next interviews' }).check();
	// A name to find it again, from the description's first words, which can be changed.
	const name = dialog.getByRole('textbox', { name: 'Name' });
	await expect(name).toHaveValue('How women in crafts learn and finance…');
	await name.fill('Crafts');
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	const first = (await created(page))?.setups[0];
	expect(first?.preset_id).toMatch(/^user-/);
	expect(first?.options.question).toBe(words);

	// In Prompts, among the analysis prompts, as the researcher's own, the description in its box.
	await page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('button', { name: 'Prompts' })
		.click();
	const main = page.getByRole('main');
	await main.getByRole('button', { name: /^Crafts Yours$/ }).click();
	await expect(main.getByRole('textbox', { name: 'What is your research about?' })).toHaveValue(
		words
	);

	// The dialog now offers it, chosen as the one used last, its words shown in place of the box.
	await backToTheRecording(page);
	dialog = await interviewDialog(page);
	const research = dialog.getByRole('combobox', { name: 'Your research' });
	await expect(research).toBeFocused();
	await expect(research.locator('option:checked')).toHaveText('Crafts');
	await expect(research.locator('option')).toHaveText(['Crafts', 'A new description']);
	await expect(dialog.getByText(words)).toBeVisible();
	await expect(dialog.getByRole('textbox')).toHaveCount(0);
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	const again = (await created(page))?.setups[0];
	expect(again?.preset_id).toBe(first?.preset_id);
	expect(again?.options.question).toBeUndefined();
});

test('a saved description is chosen, and a new one gives the box back', async ({ page }) => {
	await open(page, 'jobs-audio&research');
	const dialog = await interviewDialog(page);
	const research = dialog.getByRole('combobox', { name: 'Your research' });
	await expect(research.locator('option')).toHaveText([
		'Crafts apprenticeship',
		'Market traders',
		'A new description'
	]);
	await expect(research.locator('option:checked')).toHaveText('Crafts apprenticeship');
	await expect(dialog.getByText(/^How young people learn a craft/)).toBeVisible();
	await expect(dialog.getByText('You can change or delete it in Prompts.')).toBeVisible();

	await research.selectOption({ label: 'A new description' });
	await expect(dialog.getByRole('textbox', { name: 'What is your research about?' })).toBeVisible();
	await research.selectOption({ label: 'Market traders' });
	await expect(dialog.getByRole('textbox')).toHaveCount(0);
	await dialog.getByRole('button', { name: 'Analyse', exact: true }).click();
	await expect(dialog).toBeHidden();
	const setup = (await created(page))?.setups[0];
	expect(setup?.preset_id).toBe('user-research-1');
	expect(setup?.options.question).toBeUndefined();
});

test('the ready-made prompt duplicated in Prompts with a description is offered by the dialog', async ({
	page
}) => {
	await open(page, 'prompts');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: /^Interview: themes and quotations/ }).click();
	await main.getByRole('button', { name: 'Duplicate' }).click();
	await expect(main.getByRole('textbox', { name: 'Name', exact: true }).first()).toHaveValue(
		'Interview: themes and quotations (copy)'
	);
	await main.getByRole('textbox', { name: 'Name', exact: true }).first().fill('Port workers');
	await main
		.getByRole('textbox', { name: 'What is your research about?' })
		.fill('How dock workers find a day’s work.');
	await main.getByRole('button', { name: 'Save' }).click();
	await expect(main.getByRole('button', { name: /^Port workers Yours$/ })).toBeVisible();

	await backToTheRecording(page);
	const dialog = await interviewDialog(page);
	await expect(
		dialog.getByRole('combobox', { name: 'Your research' }).locator('option:checked')
	).toHaveText('Port workers');
	await expect(dialog.getByText('How dock workers find a day’s work.')).toBeVisible();
});

test.describe('kept descriptions in French', () => {
	test.use({ locale: 'fr-FR' });

	test('the choice and the box to keep speak French', async ({ page }) => {
		await open(page, 'jobs-audio&research');
		await page
			.getByRole('main')
			.locator('summary', { hasText: 'Utiliser ce texte dans un nouveau traitement' })
			.click();
		await page.getByRole('button', { name: /Analyser cet entretien/ }).click();
		const dialog = page.getByRole('dialog', { name: 'Analyser cet entretien' });
		const research = dialog.getByRole('combobox', { name: 'Votre recherche' });
		await expect(research.locator('option').last()).toHaveText('Une nouvelle description');
		await expect(
			dialog.getByText('Vous pouvez le modifier ou le supprimer dans Prompts.')
		).toBeVisible();
		await research.selectOption({ label: 'Une nouvelle description' });
		await dialog
			.getByRole('textbox', { name: 'Sur quoi porte votre recherche ?' })
			.fill('Les commerçantes et le crédit.');
		await dialog
			.getByRole('checkbox', { name: 'Garder ce texte pour mes prochains entretiens' })
			.check();
		await expect(dialog.getByRole('textbox', { name: 'Nom' })).toHaveValue(
			'Les commerçantes et le crédit.'
		);
	});
});
