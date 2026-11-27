// Settings › Other servers against the sample data (`?mock`, `src/lib/dev/mockServers.ts`): any
// address finds an Ollama with a model that reads pictures, one that reads only text and one run
// at ollama.com; a host with `nowhere` answers nothing, `locked` asks for a key and refuses
// `wrong`. A server added is a provider like the others in New job, each model offered for
// what its trial showed.

import { expect, test, type Locator, type Page } from '@playwright/test';

async function open(page: Page, query = ''): Promise<Locator> {
	await page.goto(`/?mock&view=settings-servers${query}`);
	await page.waitForLoadState('networkidle');
	return page.getByRole('main');
}

async function connect(main: Locator, name: string, address: string): Promise<void> {
	await main.getByRole('button', { name: 'Add a server' }).click();
	await main.getByRole('textbox', { name: 'Name' }).fill(name);
	await main.getByRole('textbox', { name: 'Address' }).fill(address);
	await main.getByRole('button', { name: 'Connect' }).click();
}

test('a server is added in three steps, and New job offers each model for what it did', async ({
	page
}) => {
	const main = await open(page);
	await expect(main.getByRole('tab', { name: 'Other servers' })).toHaveAttribute(
		'aria-selected',
		'true'
	);
	await expect(main.getByText('No server added yet.')).toBeVisible();
	await expect(main.getByRole('link', { name: 'How to add a server' })).toHaveAttribute(
		'href',
		'https://kalamu.frederickmadore.com/guide/other-servers.html'
	);

	await connect(main, 'Lab Ollama', 'http://192.168.1.20:11434');
	await expect(main.getByText('Ollama 0.40.18, on the network at 192.168.1.20.')).toBeVisible();
	// Plain http to another computer: said before anything is sent.
	await expect(main.getByText(/your files travel over the network unencrypted/)).toBeVisible();
	// A model run at ollama.com cannot be ticked, and says why.
	await expect(main.getByRole('checkbox', { name: /gpt-oss:120b-cloud/ })).toBeDisabled();
	await expect(
		main.getByText('Runs at ollama.com, not on this server, so it cannot be added.')
	).toBeVisible();

	await main.getByRole('checkbox', { name: /gemma3:12b/ }).check();
	await main.getByRole('checkbox', { name: /llama3\.1:8b/ }).check();
	await main.getByRole('button', { name: 'Try and add' }).click();
	const gemma = main.getByRole('listitem').filter({ hasText: 'gemma3:12b' });
	const llama = main.getByRole('listitem').filter({ hasText: 'llama3.1:8b' });
	await expect(gemma.getByText(/^Trying…/)).toBeVisible();
	await expect(main.getByText('“Lab Ollama” is saved.', { exact: false })).toBeVisible();
	// Both results, each with what follows from it.
	await expect(gemma.getByText('offered for text extraction', { exact: true })).toBeVisible();
	await expect(
		gemma.getByText('With both, it is offered for tables of contents too.')
	).toBeVisible();
	await expect(llama.getByText('not offered for text extraction')).toBeVisible();
	await expect(
		llama.getByText('offered for analyses, corrections and translations', { exact: true })
	).toBeVisible();
	await main.getByRole('button', { name: 'Done' }).click();

	// Listed, with where it is and what each model is offered for.
	const listed = main
		.getByRole('listitem')
		.filter({ has: page.getByRole('heading', { name: 'Lab Ollama' }) });
	await expect(listed.getByText('On the network')).toBeVisible();
	await expect(listed.getByText('192.168.1.20:11434')).toBeVisible();
	await expect(
		listed.getByText(/^Offered for analyses, corrections and translations/)
	).toBeVisible();
	// Its key is not asked for among the providers' keys.
	await main.getByRole('tab', { name: 'API keys' }).click();
	await expect(main.getByText('Lab Ollama')).toHaveCount(0);

	// New job: the model that reads pages is offered for text extraction, under the servers'
	// heading with what serves it; the one that reads only text is not.
	await page
		.getByRole('navigation')
		.getByRole('button', { name: /^New job/ })
		.click();
	const setups = main.getByRole('region', { name: /^Model and prompt/ });
	await setups.getByRole('button', { name: 'Tier', exact: true }).click();
	await setups.getByRole('combobox', { name: 'Model' }).first().click();
	const list = page.getByRole('listbox', { name: 'Model' });
	const servers = list.getByRole('group', { name: 'Other servers' });
	await expect(servers.getByRole('option', { name: /^gemma3:12b/ })).toContainText(
		'Ollama on the network at 192.168.1.20'
	);
	await expect(list.getByRole('option', { name: /^llama3\.1:8b/ })).toHaveCount(0);
	await servers.getByRole('option', { name: /^gemma3:12b/ }).click();
	await expect(setups.getByRole('combobox', { name: 'Model' }).first()).toHaveText(
		/gemma3:12b\s*Lab Ollama/
	);
	// No key to ask for, no price set, no economy mode.
	await expect(main.getByText(/No Lab Ollama key yet/)).toHaveCount(0);
	await expect(main.getByRole('heading', { name: 'Speed and price' })).toHaveCount(0);

	// An analysis: the model that reads only text is offered.
	await main.getByRole('button', { name: /^Analyse texts/ }).click();
	await setups.getByRole('combobox', { name: 'Model' }).first().click();
	await expect(list.getByRole('option', { name: /^llama3\.1:8b/ })).toBeVisible();
});

test('what kept a server from being found is said in plain words', async ({ page }) => {
	const main = await open(page);
	await connect(main, 'Office', 'http://nowhere.lan:11434');
	const alert = main.getByRole('alert');
	await expect(alert).toContainText('Nothing answers at nowhere.lan:11434.');
	await expect(alert).toContainText('is running on that computer');
	await expect(alert).not.toContainText('internet');

	await main.getByRole('textbox', { name: 'Address' }).fill('http://locked.lan');
	await main.getByRole('button', { name: 'Connect' }).click();
	await expect(alert).toContainText('The server at locked.lan asks for a key.');
	await main.getByRole('checkbox', { name: 'This server asks for a key' }).check();
	await main.getByLabel(/^Key/).fill('wrong');
	await main.getByRole('button', { name: 'Connect' }).click();
	await expect(alert).toContainText('The server at locked.lan refused the key.');

	await main.getByRole('textbox', { name: 'Address' }).fill('localhost:11434');
	await main.getByRole('button', { name: 'Connect' }).click();
	await expect(alert).toHaveText('The address must start with http:// or https://.');
});

test('a slow trial can be stopped, and a server removed', async ({ page }) => {
	const main = await open(page, '&trial-slow');
	await connect(main, 'Desk', 'http://localhost:11434');
	await expect(main.getByText('Ollama 0.40.18, on this computer.')).toBeVisible();
	await expect(main.getByText(/unencrypted/)).toHaveCount(0);
	await main.getByRole('checkbox', { name: /gemma3:12b/ }).check();
	await main.getByRole('button', { name: 'Try and add' }).click();
	await main.getByRole('button', { name: 'Stop' }).click();
	await expect(main.getByText('Stopped, so it is not added.')).toBeVisible();
	await expect(main.getByText(/The server is not saved/)).toBeVisible();
	await main.getByRole('button', { name: 'Cancel' }).click();
	await expect(main.getByText('No server added yet.')).toBeVisible();

	// Added at last (the trial takes its time), then removed.
	await page.goto('/?mock&view=settings-servers');
	await page.waitForLoadState('networkidle');
	await connect(main, 'Desk', 'http://localhost:11434');
	await main.getByRole('checkbox', { name: /llama3\.1:8b/ }).check();
	await main.getByRole('button', { name: 'Try and add' }).click();
	await main.getByRole('button', { name: 'Done' }).click();
	const listed = main
		.getByRole('listitem')
		.filter({ has: page.getByRole('heading', { name: 'Desk' }) });
	await expect(listed.getByText('On this computer')).toBeVisible();
	page.once('dialog', (dialog) => {
		expect(dialog.message()).toContain('Its jobs and results stay.');
		void dialog.accept();
	});
	await listed.getByRole('button', { name: 'Remove' }).click();
	await expect(main.getByText('No server added yet.')).toBeVisible();
});

/** Adds the server `name` at `address` with gemma3:12b, which reads pages, from Settings. */
async function addServer(page: Page, main: Locator, name: string, address: string) {
	await connect(main, name, address);
	await main.getByRole('checkbox', { name: /gemma3:12b/ }).check();
	await main.getByRole('button', { name: 'Try and add' }).click();
	await main.getByRole('button', { name: 'Done' }).click();
	await expect(main.getByRole('heading', { name })).toBeVisible();
}

/** Starts a text extraction with the server's model, agreeing to send it the files; the job's
 *  page opens, the job running (slowly, in the sample data). */
async function startOnServer(page: Page, main: Locator, name: string, host: string) {
	await page
		.getByRole('navigation')
		.getByRole('button', { name: /^New job/ })
		.click();
	await main.getByRole('button', { name: 'Add files' }).click();
	const setups = main.getByRole('region', { name: /^Model and prompt/ });
	await setups.getByRole('button', { name: 'Tier', exact: true }).click();
	await setups.getByRole('combobox', { name: 'Model' }).first().click();
	await page
		.getByRole('listbox', { name: 'Model' })
		.getByRole('group', { name: 'Other servers' })
		.getByRole('option', { name: /^gemma3:12b/ })
		.click();
	page.once('dialog', (dialog) => {
		expect(dialog.message()).toContain(`the server “${name}” at ${host}`);
		void dialog.accept();
	});
	await page.getByRole('button', { name: /^Start(?! over)/ }).click();
	await expect(main.getByRole('button', { name: 'Pause' })).toBeVisible();
}

/** Settings › Other servers, reached from the sidebar so the sample data is kept. */
async function toServers(page: Page, main: Locator) {
	await page.getByRole('navigation').getByRole('button', { name: 'Settings' }).click();
	await main.getByRole('tab', { name: 'Other servers' }).click();
}

/** Changes the server's address in its form, its model tried again there. */
async function move(main: Locator, name: string, address: string) {
	const listed = main
		.getByRole('listitem')
		.filter({ has: main.page().getByRole('heading', { name }) });
	await listed.getByRole('button', { name: 'Edit' }).click();
	await main.getByRole('textbox', { name: 'Address' }).fill(address);
	await main.getByRole('button', { name: 'Connect' }).click();
	await main.getByRole('button', { name: 'Try and add' }).click();
}

test('a job resumed after its server moved asks again, naming the new host; declined, it waits', async ({
	page
}) => {
	const main = await open(page);
	await addServer(page, main, 'Lab Ollama', 'http://192.168.1.20:11434');
	await startOnServer(page, main, 'Lab Ollama', '192.168.1.20');
	await main.getByRole('button', { name: 'Pause' }).click();
	await expect(main.getByRole('button', { name: 'Resume' })).toBeVisible();
	const job = (await page.getByRole('heading', { level: 1 }).textContent())?.trim() ?? '';

	// Moved while the job waits: allowed, and its consent withdrawn.
	await toServers(page, main);
	await move(main, 'Lab Ollama', 'http://10.0.0.5:11434');
	await expect(main.getByText('“Lab Ollama” is saved.', { exact: false })).toBeVisible();
	await main.getByRole('button', { name: 'Done' }).click();

	// Resumed from the sidebar: asked again, about the new address; declined, nothing starts.
	const foot = page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('region', { name: /^To resume/ });
	const resume = foot.getByRole('button', { name: `Resume “${job}”` });
	let asked = '';
	page.once('dialog', (dialog) => {
		asked = dialog.message();
		void dialog.dismiss();
	});
	await resume.click();
	await expect.poll(() => asked).toContain('the server “Lab Ollama” at 10.0.0.5');
	expect(asked).not.toContain('192.168.1.20');
	await expect(resume).toBeVisible();
	await expect(foot).toContainText('Paused');
	await expect(page.getByText(`“${job}” resumed.`)).toHaveCount(0);

	// Agreed to, it runs again.
	page.once('dialog', (dialog) => void dialog.accept());
	await resume.click();
	await expect(page.getByText(`“${job}” resumed.`)).toBeVisible();
});

test('a server a running job uses cannot be moved or removed until the job is paused', async ({
	page
}) => {
	const main = await open(page);
	await addServer(page, main, 'Lab Ollama', 'http://192.168.1.20:11434');
	await startOnServer(page, main, 'Lab Ollama', '192.168.1.20');
	const busy =
		'“Lab Ollama” is in use by a running job. Pause the job, or let it finish, then change or remove the server.';

	await toServers(page, main);
	const listed = main
		.getByRole('listitem')
		.filter({ has: page.getByRole('heading', { name: 'Lab Ollama' }) });
	page.once('dialog', (dialog) => {
		expect(dialog.message()).toContain('cannot be finished');
		void dialog.accept();
	});
	await listed.getByRole('button', { name: 'Remove' }).click();
	await expect(page.getByText(busy)).toBeVisible();
	await expect(main.getByRole('heading', { name: 'Lab Ollama' })).toBeVisible();

	await move(main, 'Lab Ollama', 'http://10.0.0.5:11434');
	await expect(main.getByRole('alert')).toHaveText(busy);
});
