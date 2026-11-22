// Correcting a passage selected in the text review with a model, against the sample data
// (`?mock`), in both engines: the menu opened from the selection by a right click and by the
// keyboard, and a visible button; the dialog saying what is sent; the proposal with its
// changes marked, accepted and undone, edited, or discarded; a text changed meanwhile; a
// selection too long; a request stopped. The preview's "model" puts straight apostrophes right.

import { expect, test, type Locator, type Page } from '@playwright/test';

test.describe.configure({ timeout: 60_000 });

const PHRASE = "plus de deux cents fidèles autour de l'imam";
const CORRECTED = 'plus de deux cents fidèles autour de l’imam';

async function openReview(page: Page, extra = ''): Promise<Locator> {
	await page.goto(`/?mock&view=review${extra}`);
	await page.waitForLoadState('networkidle');
	const main = page.getByRole('main');
	const text = main.getByRole('region', { name: 'Reading text' });
	await expect(text).toContainText('deux cents fidèles');
	return text;
}

/** Selects `phrase` in the text shown, across the marks that may cut it, as a pointer would. */
async function select(page: Page, phrase: string): Promise<void> {
	await page.evaluate((phrase) => {
		const root = document.querySelector('.text.numbered');
		if (!root) throw new Error('no text');
		for (const line of root.querySelectorAll('.line .t')) {
			const walker = document.createTreeWalker(line, NodeFilter.SHOW_TEXT);
			const nodes: Text[] = [];
			let all = '';
			for (let n = walker.nextNode(); n; n = walker.nextNode()) {
				nodes.push(n as Text);
				all += n.textContent ?? '';
			}
			const at = all.indexOf(phrase);
			if (at < 0) continue;
			const place = (offset: number): [Text, number] => {
				let seen = 0;
				for (const node of nodes) {
					const length = node.textContent?.length ?? 0;
					if (offset <= seen + length) return [node, offset - seen];
					seen += length;
				}
				const last = nodes[nodes.length - 1];
				return [last, last.textContent?.length ?? 0];
			};
			line.scrollIntoView({ block: 'center' });
			const range = document.createRange();
			range.setStart(...place(at));
			range.setEnd(...place(at + phrase.length));
			const chosen = getSelection();
			chosen?.removeAllRanges();
			chosen?.addRange(range);
			return;
		}
		throw new Error(`not found: ${phrase}`);
	}, phrase);
}

/** Right-clicks in the middle of the selection. */
async function rightClickSelection(page: Page): Promise<void> {
	const box = await page.evaluate(() => {
		const r = getSelection()?.getRangeAt(0).getClientRects()[0];
		return r ? { x: r.left + r.width / 2, y: r.top + r.height / 2 } : null;
	});
	if (!box) throw new Error('no selection on screen');
	await page.mouse.click(box.x, box.y, { button: 'right' });
}

test('a passage is corrected from its menu, its changes shown, accepted and undone', async ({
	page
}) => {
	const text = await openReview(page);
	await select(page, PHRASE);
	// A visible control appears with the selection.
	await expect(page.getByRole('button', { name: 'Correct with a model…' })).toBeVisible();

	await rightClickSelection(page);
	const menu = page.getByRole('menu', { name: 'What to do with the selected text' });
	await expect(menu).toBeVisible();
	await expect(menu.getByRole('menuitem', { name: 'Copy' })).toBeVisible();
	const item = menu.getByRole('menuitem', { name: 'Correct this passage with a model…' });
	await item.click();

	const dialog = page.getByRole('dialog', { name: 'Correct a passage with a model' });
	await expect(dialog).toBeVisible();
	await expect(dialog.locator('mark.passage')).toHaveText(PHRASE);
	// The context sent around it, and to whom.
	await expect(dialog.locator('.context').first()).toContainText('rassemblé');
	await expect(dialog).toContainText('marked as context the model must not change');
	await expect(dialog).toContainText(/About \$0\.000/);

	await dialog.getByRole('button', { name: 'Ask for a proposal' }).click();
	await expect(dialog.getByText('Proposal ready.')).toBeAttached();
	await expect(dialog.locator('.diff ins')).toContainText('’');
	await expect(dialog.locator('.diff del')).toContainText("'");
	// Nothing has changed in the text yet.
	await expect(text).toContainText(PHRASE);

	await dialog.getByRole('button', { name: 'Accept', exact: true }).click();
	await expect(dialog).toBeHidden();
	await expect(text).toContainText(CORRECTED);
	// The focus is back in the text.
	await expect
		.poll(() => page.evaluate(() => document.activeElement?.closest('.page') !== null))
		.toBe(true);

	// Undone from the message, as it was.
	await page.getByRole('button', { name: 'Undo' }).click();
	await expect(text).toContainText(PHRASE);
	await expect(text).not.toContainText(CORRECTED);
});

test('the menu opens from the keyboard and the visible button, and a proposal is edited or discarded', async ({
	page
}) => {
	const text = await openReview(page);
	await select(page, PHRASE);
	await page.locator('.page').first().focus();
	await select(page, PHRASE);
	await page.keyboard.press('Shift+F10');
	const menu = page.getByRole('menu', { name: 'What to do with the selected text' });
	await expect(menu).toBeVisible();
	await expect(menu.getByRole('menuitem').first()).toBeFocused();
	await page.keyboard.press('ArrowDown');
	await page.keyboard.press('Enter');
	const dialog = page.getByRole('dialog', { name: 'Correct a passage with a model' });
	await expect(dialog).toBeVisible();
	await dialog.getByRole('button', { name: 'Ask for a proposal' }).click();
	await expect(dialog.getByText('Proposal ready.')).toBeAttached();
	await dialog.getByRole('button', { name: 'Discard' }).click();
	await expect(dialog).toBeHidden();
	await expect(text).toContainText(PHRASE);

	// From the visible button; asked before, so it costs nothing, and edited before accepting.
	await select(page, PHRASE);
	await page.getByRole('button', { name: 'Correct with a model…' }).click();
	await expect(dialog).toContainText('Asked before');
	await dialog.getByRole('button', { name: 'Ask for a proposal' }).click();
	await dialog.getByRole('button', { name: 'Edit before accepting' }).click();
	const editor = dialog.getByRole('textbox', { name: 'Proposed correction, to edit' });
	await expect(editor).toBeFocused();
	await editor.fill('plus de deux cents fidèles réunis autour de l’imam');
	await dialog.getByRole('button', { name: 'Accept', exact: true }).click();
	await expect(text).toContainText('fidèles réunis autour');
});

test('a proposal is not applied over a text changed meanwhile', async ({ page }) => {
	const text = await openReview(page);
	await select(page, PHRASE);
	await page.getByRole('button', { name: 'Correct with a model…' }).click();
	const dialog = page.getByRole('dialog', { name: 'Correct a passage with a model' });
	await dialog.getByRole('button', { name: 'Ask for a proposal' }).click();
	await expect(dialog.getByText('Proposal ready.')).toBeAttached();
	// The same words corrected elsewhere (another window, say) while the proposal waited.
	await page.evaluate(async () => {
		const ipc = (
			window as unknown as {
				__TAURI_INTERNALS__: { invoke: (c: string, a: unknown) => Promise<unknown> };
			}
		).__TAURI_INTERNALS__;
		const pieces = (await ipc.invoke('source_pieces', {
			jobId: '00000000-0000-0000-0000-0000000000j1',
			sourceId: '00000000-0000-0000-0000-000000000001'
		})) as { setup_id: string; index: number; ai_text: string; reviewed: boolean }[];
		const first = pieces.find((p) => p.index === 0);
		if (!first) throw new Error('no piece');
		await ipc.invoke('save_correction', {
			correction: {
				job_id: '00000000-0000-0000-0000-0000000000j1',
				setup_id: first.setup_id,
				source_id: '00000000-0000-0000-0000-000000000001',
				index: 0,
				text: first.ai_text.replace("l'imam", 'l’imam du quartier'),
				reviewed: first.reviewed,
				updated_at: 0
			}
		});
	});
	await dialog.getByRole('button', { name: 'Accept', exact: true }).click();
	await expect(dialog.getByRole('alert')).toContainText(
		'The text under the selection changed while the model was answering'
	);
	await dialog.getByRole('button', { name: 'Look again' }).click();
	await expect(dialog).toBeHidden();
	await expect(text).not.toContainText(CORRECTED);
});

test('a selection too short or too long is refused with the reason, and a request can be stopped', async ({
	page
}) => {
	await openReview(page, '&passage-slow');
	const dialog = page.getByRole('dialog', { name: 'Correct a passage with a model' });

	// Two letters: too short to send.
	await select(page, 'de');
	await page.getByRole('button', { name: 'Correct with a model…' }).click();
	await expect(dialog.getByRole('alert')).toContainText('Select at least 3 letters or figures');
	await expect(dialog.getByRole('button', { name: 'Ask for a proposal' })).toBeDisabled();
	await page.keyboard.press('Escape');
	await expect(dialog).toBeHidden();

	// A request stopped while it waits.
	await select(page, PHRASE);
	await page.getByRole('button', { name: 'Correct with a model…' }).click();
	await dialog.getByRole('button', { name: 'Ask for a proposal' }).click();
	await expect(dialog).toContainText('Waiting for');
	await dialog.getByRole('button', { name: 'Stop waiting' }).click();
	await expect(dialog.getByRole('status').last()).toContainText('Stopped waiting');
	await expect(dialog.getByRole('button', { name: 'Ask for a proposal' })).toBeEnabled();
	await dialog.getByRole('button', { name: 'Cancel' }).click();
	await expect(dialog).toBeHidden();

	// A page corrected by hand to more than one call takes, then selected whole.
	await page.locator('header').getByRole('button', { name: 'Correct', exact: true }).click();
	const editor = page.locator('textarea.editor');
	await editor.fill(`Une longue page.
${'Le commandant de cercle assistait à la séance. '.repeat(100)}`);
	await page.getByRole('button', { name: /Save correction/ }).click();
	await expect(editor).toBeHidden();
	await page.evaluate(() => {
		const root = document.querySelector('.text.numbered');
		if (!root) throw new Error('no text');
		const range = document.createRange();
		range.selectNodeContents(root);
		getSelection()?.removeAllRanges();
		getSelection()?.addRange(range);
	});
	await page.getByRole('button', { name: 'Correct with a model…' }).click();
	await expect(dialog.getByRole('alert')).toContainText('At most 4000 are sent at once');
	await expect(dialog.getByRole('button', { name: 'Ask for a proposal' })).toBeDisabled();
});

test('a right click where nothing is selected leaves the system’s menu alone', async ({ page }) => {
	const text = await openReview(page);
	await page.evaluate(() => getSelection()?.removeAllRanges());
	const prevented = await text.evaluate((el) => {
		const e = new MouseEvent('contextmenu', {
			bubbles: true,
			cancelable: true,
			clientX: 10,
			clientY: 10
		});
		el.querySelector('.line .t')?.dispatchEvent(e);
		return e.defaultPrevented;
	});
	expect(prevented).toBe(false);
	await expect(page.getByRole('menu')).toHaveCount(0);
});

test('a page read as handwriting starts its passage’s correction on the handwritten prompt', async ({
	page
}) => {
	for (const [extra, prompt] of [
		['&handwritten', 'Handwritten document or manuscript'],
		['', 'Printed document']
	]) {
		await openReview(page, extra);
		await select(page, PHRASE);
		await page.getByRole('button', { name: 'Correct with a model…' }).click();
		const dialog = page.getByRole('dialog', { name: 'Correct a passage with a model' });
		const picked = dialog.getByRole('combobox', { name: 'Prompt' }).locator('option:checked');
		await expect(picked).toHaveText(prompt);
	}
});
