// The test build of the app driven against the stand-in provider, flow by flow, as a researcher
// goes: what the Windows test pass did by hand on an installed copy and CI can now do on every
// change to the shell or the engine (docs/windows-test-pass.md marks each step it covers).
// Elements are found by their accessible names, as in e2e/.

import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { expect, test, type Page } from '@playwright/test';
import { App, StandIn, folders, written, type Folders } from './app';

let standIn: StandIn;
let app: App | undefined;

test.beforeEach(async () => {
	standIn = await StandIn.start();
});

test.afterEach(async () => {
	await app?.kill();
	app = undefined;
	await standIn.stop();
});

async function launch(dirs: Folders): Promise<Page> {
	app = await App.launch(dirs, standIn);
	return app.page;
}

/** Quits the app as *Quit now* does, checking that it ends cleanly. */
async function quit(): Promise<void> {
	expect(await app?.quit()).toBe(0);
	app = undefined;
}

/** New job → Extract text with `files`, the model each provider proposes first (Gemini),
 *  started. */
async function startExtraction(page: Page, dirs: Folders, files: string[]): Promise<void> {
	await page.getByRole('button', { name: /^New job/ }).click();
	await page.getByRole('button', { name: /^Extract text/ }).click();
	app?.answerDialogs(files.map((f) => join(dirs.work, f)));
	await page.getByRole('button', { name: 'Add files' }).click();
	for (const f of files) {
		await expect(page.getByRole('button', { name: `Remove ${f}` })).toBeVisible();
	}
	await expect(page.getByRole('combobox', { name: 'Provider' })).toHaveValue(/gemini/i);
	await page
		.getByRole('complementary')
		.getByRole('button', { name: /^Start/ })
		.click();
}

/** The job's page, once it says the job is done. */
async function done(page: Page, job: string, timeout = 60_000): Promise<void> {
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { level: 1, name: job })).toBeVisible();
	await expect(main.getByText('Done', { exact: true })).toBeVisible({ timeout });
}

test('a job from New job to its exported files', async () => {
	const dirs = folders(['daily-nebraskan-1911-issue.pdf']);
	const page = await launch(dirs);

	await startExtraction(page, dirs, ['daily-nebraskan-1911-issue.pdf']);
	await done(page, 'daily-nebraskan-1911-issue.pdf');
	const main = page.getByRole('main');
	await expect(main.getByText('4 of 4 pieces')).toBeVisible();

	// The four pages read once each, by the stand-in, and the text written beside the source in
	// the formats New job ticks by default, with its provenance.
	expect(await standIn.pagesAsked(), JSON.stringify(await standIn.log())).toBe(4);
	const files = [
		'daily-nebraskan-1911-issue.docx',
		'daily-nebraskan-1911-issue.json',
		'daily-nebraskan-1911-issue.provenance.json',
		'daily-nebraskan-1911-issue.txt'
	];
	await expect
		.poll(() => written(dirs.work, 'daily-nebraskan-1911-issue.'))
		.toEqual(expect.arrayContaining(files));
	const written_ = page.getByRole('region', { name: /^Written/ });
	for (const f of files) {
		await expect(written_.getByRole('button', { name: f, exact: true })).toBeVisible();
	}
	const text = readFileSync(join(dirs.work, 'daily-nebraskan-1911-issue.txt'), 'utf8');
	expect(text).toContain('Bamako, le 3 mai 1957');
	const provenance = readFileSync(
		join(dirs.work, 'daily-nebraskan-1911-issue.provenance.json'),
		'utf8'
	);
	expect(provenance).toContain('gemini');
	expect(provenance).not.toContain('stand-in-key');
	await quit();
});

/** Opens the job named `job` from the sidebar. */
async function openJob(page: Page, job: string): Promise<void> {
	await page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('button', { name: new RegExp(`^${job.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}`) })
		.click();
}

test('a correction saved is found after a restart, and written to the files', async () => {
	const dirs = folders(['bulletin-page.jpg']);
	let page = await launch(dirs);
	await startExtraction(page, dirs, ['bulletin-page.jpg']);
	await done(page, 'bulletin-page.jpg');

	await page.getByRole('button', { name: 'Review', exact: true }).click();
	await page.getByRole('button', { name: 'Correct', exact: true }).click();
	const editor = page.getByRole('textbox', { name: /^Correct / });
	await editor.fill('Bamako, le 4 mai 1957\nMonsieur le Gouverneur,');
	await page.getByRole('button', { name: /^Save correction/ }).click();
	const reading = page.getByRole('region', { name: 'Reading text' });
	await expect(reading).toContainText('le 4 mai 1957');
	await quit();
	// The files a correction left out of date are written by the time the app has ended.
	const text = () => readFileSync(join(dirs.work, 'bulletin-page.txt'), 'utf8');
	expect(text()).toContain('le 4 mai 1957');

	page = await launch(dirs);
	await openJob(page, 'bulletin-page.jpg');
	await done(page, 'bulletin-page.jpg');
	await page.getByRole('button', { name: 'Review', exact: true }).click();
	await expect(page.getByRole('region', { name: 'Reading text' })).toContainText('le 4 mai 1957');
	// Nothing was asked of the provider again.
	expect(await standIn.pagesAsked(), JSON.stringify(await standIn.log())).toBe(1);
	await quit();
});

test('a paused job, the app quit and opened again, resumes without reading a page twice', async () => {
	const dirs = folders(['daily-nebraskan-1911-issue.pdf']);
	let page = await launch(dirs);
	// Each page's answer takes a while, so the job can be paused part way.
	await standIn.delay(5_000);
	await startExtraction(page, dirs, ['daily-nebraskan-1911-issue.pdf']);
	const main = page.getByRole('main');
	const job = (state: string) =>
		page
			.getByRole('navigation', { name: 'Jobs' })
			.getByRole('button', { name: new RegExp(`^daily-nebraskan-1911-issue\\.pdf ${state}`) });
	await expect(job('Running · [1-3] of 4 pieces')).toBeVisible({ timeout: 30_000 });
	await main.getByRole('button', { name: 'Pause', exact: true }).click();
	// The page already sent is stored, then the job waits.
	await expect(job('Paused')).toBeVisible({ timeout: 30_000 });
	const before = await standIn.pagesAsked();
	expect(before, JSON.stringify(await standIn.log())).toBeLessThan(4);
	await quit();

	await standIn.delay(0);
	page = await launch(dirs);
	const sidebar = page.getByRole('navigation', { name: 'Jobs' });
	await sidebar
		.getByRole('region', { name: /^To resume/ })
		.getByRole('button', { name: 'Resume “daily-nebraskan-1911-issue.pdf”' })
		.click();
	await openJob(page, 'daily-nebraskan-1911-issue.pdf');
	await done(page, 'daily-nebraskan-1911-issue.pdf');
	await expect(page.getByRole('main').getByText('4 of 4 pieces')).toBeVisible();
	// Each page read once in all: none read before the pause was sent again.
	expect(await standIn.pagesAsked(), JSON.stringify(await standIn.log())).toBe(4);
	await expect
		.poll(() => written(dirs.work, 'daily-nebraskan-1911-issue.'))
		.toContain('daily-nebraskan-1911-issue.txt');
	await quit();
});

test('a batch sent in the cheaper mode is collected after a restart', async () => {
	const dirs = folders(['daily-nebraskan-1911-issue.pdf']);
	let page = await launch(dirs);
	await page.getByRole('button', { name: /^Extract text/ }).click();
	app?.answerDialogs([join(dirs.work, 'daily-nebraskan-1911-issue.pdf')]);
	await page.getByRole('button', { name: 'Add files' }).click();
	await page.getByRole('radio', { name: /^Cheaper/ }).check();
	await page
		.getByRole('complementary')
		.getByRole('button', { name: /^Start/ })
		.click();
	const main = page.getByRole('main');
	await expect(main.getByText(/with the provider’s batch service/)).toBeVisible({
		timeout: 30_000
	});
	await quit();

	// The provider answers while the app is closed.
	await standIn.releaseBatches();
	page = await launch(dirs);
	await openJob(page, 'daily-nebraskan-1911-issue.pdf');
	// Collected at launch; the job ends at the collection's next turn, a minute later at most.
	await done(page, 'daily-nebraskan-1911-issue.pdf', 120_000);
	await expect
		.poll(() => written(dirs.work, 'daily-nebraskan-1911-issue.'))
		.toContain('daily-nebraskan-1911-issue.txt');
	const log = await standIn.log();
	// One batch made, its pages never sent one by one.
	expect(log.filter((r) => r.path.endsWith(':batchGenerateContent'))).toHaveLength(1);
	expect(log.filter((r) => r.path.endsWith(':generateContent'))).toHaveLength(0);
	await quit();
});

test('a project saved to a file is opened again as a project of its own', async () => {
	const dirs = folders(['bulletin-page.jpg']);
	const page = await launch(dirs);
	await startExtraction(page, dirs, ['bulletin-page.jpg']);
	await done(page, 'bulletin-page.jpg');

	const sidebar = page.getByRole('navigation', { name: 'Jobs' });
	await sidebar.getByRole('button', { name: 'Projects', exact: true }).click();
	await page.getByRole('button', { name: 'New project' }).click();
	const form = page.getByRole('form', { name: 'New project' });
	await form.getByRole('textbox', { name: 'Name' }).fill('Bulletins');
	await form.getByRole('button', { name: 'Create' }).click();
	const main = page.getByRole('main');
	await expect(main.getByRole('heading', { level: 1, name: 'Bulletins' })).toBeVisible();
	// Empty, it says what to do first, and its sections what will come there (the pass's A9).
	await expect(main.getByRole('region', { name: 'Start by adding your documents' })).toBeVisible();
	const sections = main.getByRole('navigation', { name: 'Collections' });
	await sections.getByRole('button', { name: 'Search' }).click();
	await expect(main).toContainText('Nothing has been read in this project yet.');
	await sections.getByRole('button', { name: 'To check' }).click();
	await expect(main).toContainText('Nothing has been read here yet. Once a job has read');
	await sections.getByRole('button', { name: /^All documents/ }).click();
	await main.getByRole('button', { name: 'Add an existing job' }).click();
	const attach = page.getByRole('dialog', { name: 'Add an existing job' });
	await attach.getByRole('combobox', { name: 'Job' }).selectOption({ index: 1 });
	await expect(attach.getByRole('combobox', { name: 'bulletin-page.jpg' })).toBeVisible();
	await attach.getByRole('button', { name: 'Add', exact: true }).click();
	await expect(main.getByRole('button', { name: /^All documents 1/ })).toBeVisible();

	const file = join(dirs.work, 'Bulletins.kalamu');
	await main.getByRole('button', { name: 'Save to a file…' }).click();
	const save = page.getByRole('dialog', { name: 'Save this project to a file' });
	app?.answerDialogs(file);
	await save.getByRole('button', { name: 'Save…' }).click();
	await expect.poll(() => written(dirs.work, 'Bulletins.kalamu')).toEqual(['Bulletins.kalamu']);
	await save.getByRole('button', { name: 'Close', exact: true }).click();

	await sidebar.getByRole('button', { name: 'Projects', exact: true }).click();
	app?.answerDialogs(file);
	await main.getByRole('button', { name: 'Open a project file…' }).click();
	// Opened beside the first, under a name of its own.
	await expect(
		page.getByText(/^“Bulletins \(.+\)” is open, with 1 document and 1 job\./)
	).toBeVisible();
	const projects = page.getByRole('list', { name: 'Projects' });
	await expect(projects.getByRole('button', { name: /^Bulletins/ })).toHaveCount(2);
	await quit();
});
