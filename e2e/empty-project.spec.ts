// A first project, before anything is in it (issue #212): its documents say what to do first and
// that the rest fills itself, the other sections say in a line what will come there, and all of
// it goes once a document is added. A project with documents and results keeps its table.

import { expect, test, type Page } from '@playwright/test';

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

async function newProject(page: Page, name: string): Promise<void> {
	await open(page, 'projects');
	const main = page.getByRole('main');
	await main.getByRole('button', { name: 'New project' }).click();
	await main.getByRole('textbox', { name: 'Name' }).fill(name);
	await main.getByRole('button', { name: 'Create' }).click();
	await expect(main.getByRole('heading', { level: 1, name })).toBeVisible();
}

test('a new project says what to do first, and the orientation leaves once a document is added', async ({
	page
}) => {
	await newProject(page, 'Letters from Kayes');
	const main = page.getByRole('main');
	const start = main.getByRole('region', { name: 'Start by adding your documents' });
	await expect(start).toBeVisible();
	// It names the buttons that are there, above it.
	await expect(start).toContainText('“Add files” or “Add a folder”');
	await expect(start).toContainText('“Add an existing job”');
	await expect(start).toContainText('Nothing else has to be filled in.');
	await expect(main.getByRole('button', { name: 'Add files' })).toBeVisible();
	await expect(main.getByRole('button', { name: 'Add an existing job' })).toBeVisible();
	// Nothing to dismiss.
	await expect(start.getByRole('button')).toHaveCount(0);

	// The other sections say in a line what will come there.
	const sections = main.getByRole('navigation', { name: 'Collections' });
	await sections.getByRole('button', { name: 'Search' }).click();
	await expect(main).toContainText('Nothing has been read in this project yet.');
	await sections.getByRole('button', { name: 'To check' }).click();
	await expect(main).toContainText('Nothing has been read here yet. Once a job has read');
	await sections.getByRole('button', { name: 'Dictionary' }).click();
	await expect(main).toContainText('No analysis of these documents lists names yet.');
	await sections.getByRole('button', { name: /^Activity/ }).click();
	await expect(main).toContainText('No job has read these documents yet.');

	// Files added: the table takes its place, and the orientation is gone for good.
	await sections.getByRole('button', { name: /^All documents/ }).click();
	await expect(start).toBeVisible();
	await main.getByRole('button', { name: 'Add files' }).click();
	await expect(main.getByRole('table')).toBeVisible();
	// The sample picker gives two recordings for a project, whose picker takes every kind.
	await expect(main.getByRole('button', { name: 'imam-2024-03', exact: true })).toBeVisible();
	await expect(start).toBeHidden();
	await expect(main).not.toContainText('Start by adding your documents');
});

test('a project with documents and results keeps its table, without the orientation', async ({
	page
}) => {
	await open(page, 'project');
	const main = page.getByRole('main');
	await expect(main.getByRole('table')).toBeVisible();
	await expect(main).not.toContainText('Start by adding your documents');
	await main
		.getByRole('navigation', { name: 'Collections' })
		.getByRole('button', { name: 'Search' })
		.click();
	await expect(main.getByRole('searchbox', { name: 'Words to find' })).toBeVisible();
	await expect(main).not.toContainText('Nothing has been read in this project yet.');
});

test.describe('in French', () => {
	test.use({ locale: 'fr-FR' });
	test('a new project says what to do first, in French', async ({ page }) => {
		await open(page, 'projects');
		const main = page.getByRole('main');
		await main.getByRole('button', { name: 'Nouveau projet' }).click();
		await main.getByRole('textbox', { name: 'Nom' }).fill('Lettres de Kayes');
		await main.getByRole('button', { name: 'Créer' }).click();
		const start = main.getByRole('region', { name: 'Commencez par ajouter vos documents' });
		await expect(start).toBeVisible();
		await expect(start).toContainText('Ajouter des fichiers');
		await expect(start).toContainText('Ajouter un traitement existant');
	});
});
