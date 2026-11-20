// A sample first (#215), against the sample data (src/lib/dev/mockSample.ts): `?long` gives New
// job a PDF of 40 pages, which suggests trying three pages first, alone or with a second model;
// `?sample` lists a sample of three pages whose page runs the rest with its model, priced with
// the pages already read at no cost, or compares another model on the same pages.

import { expect, test, type Page } from '@playwright/test';

type Asked = { cmd: string; args: Record<string, unknown> }[];

const asked = (page: Page) =>
	page.evaluate(() => (window as unknown as { __kalamuSample?: Asked }).__kalamuSample ?? []);

async function openLong(page: Page): Promise<void> {
	await page.goto('/?mock&view=new&long');
	await page.waitForLoadState('networkidle');
	await page.getByRole('main').getByRole('button', { name: 'Add files' }).click();
}

test('a long document suggests three pages first, with two models in one click', async ({
	page
}) => {
	await openLong(page);
	const main = page.getByRole('main');
	const offer = main.getByRole('region', { name: 'Try on a few pages first' });
	await expect(offer).toBeVisible();
	await expect(offer).toContainText('40 pages is a lot to read at once.');
	// Nothing is chosen until the researcher says so: every page, one model.
	await expect(main.getByRole('checkbox', { name: /Try a sample first/ })).not.toBeChecked();

	await offer.getByRole('button', { name: 'Try on 3 pages with two models' }).click();
	await expect(offer).toContainText('with both models');
	await expect(main.getByRole('checkbox', { name: /Try a sample first/ })).toBeChecked();
	await expect(
		main.getByText('3 pages spread through each document', { exact: true })
	).toBeVisible();
	await expect(main.getByRole('button', { name: 'Remove', exact: true })).toHaveCount(2);

	await page.getByRole('button', { name: /^Start(?! over)/ }).click();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText(/bulletin-1957-06/);
	const created = (await asked(page)).find((a) => a.cmd === 'create_job');
	const job = created?.args.job as {
		settings: { sample_pages: number | null; sample_spread?: boolean };
		setups: { model: string }[];
	};
	expect(job.settings.sample_pages).toBe(3);
	expect(job.settings.sample_spread).toBe(true);
	expect(job.setups).toHaveLength(2);
	expect(job.setups[0].model).not.toBe(job.setups[1].model);
});

test('the suggestion is taken back with Read every page, and a short document has none', async ({
	page
}) => {
	await openLong(page);
	const main = page.getByRole('main');
	const offer = main.getByRole('region', { name: 'Try on a few pages first' });
	await offer.getByRole('button', { name: 'Try on 3 pages', exact: true }).click();
	await expect(main.getByRole('button', { name: 'Remove', exact: true })).toHaveCount(0);
	await expect(offer).toContainText('spread through each document, not the first ones');
	await offer.getByRole('button', { name: 'Read every page' }).click();
	await expect(main.getByRole('checkbox', { name: /Try a sample first/ })).not.toBeChecked();
	await expect(offer.getByRole('button', { name: 'Try on 3 pages', exact: true })).toBeVisible();

	// The sample data's own files are three pages and one: nothing is suggested.
	await page.goto('/?mock&view=new');
	await page.waitForLoadState('networkidle');
	await page.getByRole('main').getByRole('button', { name: 'Add files' }).click();
	await expect(
		page.getByRole('main').getByRole('button', { name: 'Remove', exact: true })
	).toHaveCount(0);
	await expect(page.getByText('Try on a few pages first')).toHaveCount(0);
});

async function openSample(page: Page): Promise<void> {
	await page.goto('/?mock&view=new&sample');
	await page.waitForLoadState('networkidle');
	await page
		.getByRole('button', { name: /Bulletin, January 1958/ })
		.first()
		.click();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText('Bulletin, January 1958');
}

test('from a sample, the rest runs with its model, the pages read at no cost', async ({ page }) => {
	await openSample(page);
	const main = page.getByRole('main');
	await expect(main.getByText('A sample: 3 pages spread through each document.')).toBeVisible();
	await expect(main.getByText(/^37 pages left, about \$0\.040?\./)).toBeVisible();
	await expect(main.getByText('The 3 pages already read cost nothing.')).toBeVisible();

	await main.getByRole('button', { name: /^Run the rest with Gemini/ }).click();
	await expect(page.getByRole('heading', { level: 1 })).toHaveText(
		'Bulletin, January 1958 (every page)'
	);
	const rest = (await asked(page)).find((a) => a.cmd === 'rest_of_sample');
	expect(rest?.args).toMatchObject({
		job: '00000000-0000-0000-0000-0000000000s1',
		setup: '00000000-0000-0000-0000-00000000000a'
	});
	// The job of every page is no sample: nothing more to offer there.
	await expect(main.getByRole('button', { name: /^Run the rest/ })).toHaveCount(0);
});

test('from a sample, another model reads the same pages beside it', async ({ page }) => {
	await openSample(page);
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'Compare with another model' }).click();
	const dialog = page.getByRole('dialog', { name: 'Compare with another model' });
	await expect(dialog).toBeVisible();
	await expect(dialog).toContainText('Only the new model is paid for');
	await dialog.getByRole('button', { name: 'Start' }).click();
	await expect(dialog).toHaveCount(0);
	await expect(page.getByRole('heading', { level: 1 })).toHaveText(
		'Bulletin, January 1958 (compared)'
	);
	const compared = (await asked(page)).find((a) => a.cmd === 'compare_sample');
	expect(compared?.args.job).toBe('00000000-0000-0000-0000-0000000000s1');
	expect(compared?.args.model).not.toBe('gemini-3.8-flash');
});
