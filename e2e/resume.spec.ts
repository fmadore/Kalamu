// The job that waits to be resumed, at the sidebar's foot from any screen, against the sample
// data (`?mock`: a job paused half an hour ago and an interview cut short by a quit), in both
// engines: resumed from the keyboard, the focus kept when its row goes, and a refusal said where
// it is seen. Elements are found by their accessible names, as a screen reader finds them.

import { expect, test, type Page } from '@playwright/test';

// WebKit opens the dev server's pages slowly while other workers start.
test.describe.configure({ timeout: 60_000 });

async function open(page: Page, view: string): Promise<void> {
	await page.goto(`/?mock&view=${view}`);
	await page.waitForLoadState('networkidle');
}

test('the job stopped last is offered first, the others a click away, none that runs or ended', async ({
	page
}) => {
	await open(page, 'settings');
	const foot = page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('region', { name: /^To resume/ });
	const paused = foot.getByRole('button', { name: 'Resume “Rapports de cercle, Dori”' });
	await expect(paused).toBeVisible();
	await expect(foot).toContainText('Paused · 5 of 12 pieces');
	const interrupted = foot.getByRole('button', { name: 'Resume “Entretiens Ouagadougou”' });
	await expect(interrupted).toHaveCount(0);

	const more = foot.getByRole('button', { name: '1 more' });
	await expect(more).toHaveAttribute('aria-expanded', 'false');
	await more.click();
	await expect(more).toHaveAttribute('aria-expanded', 'true');
	await expect(interrupted).toBeVisible();
	await expect(foot).toContainText('Interrupted · 2 of 4 pieces');
	// Finished, cancelled or failed jobs are never offered.
	await expect(foot.getByRole('button', { name: /^Resume/ })).toHaveCount(2);
});

test('a job resumed from the keyboard leaves the foot and the focus goes on', async ({ page }) => {
	await open(page, 'jobs');
	const sidebar = page.getByRole('navigation', { name: 'Jobs' });
	const foot = sidebar.getByRole('region', { name: /^To resume/ });
	const paused = foot.getByRole('button', { name: 'Resume “Rapports de cercle, Dori”' });
	await paused.focus();
	await page.keyboard.press('Enter');

	// Its row goes; the next job to resume takes the focus.
	await expect(paused).toHaveCount(0);
	const interrupted = foot.getByRole('button', { name: 'Resume “Entretiens Ouagadougou”' });
	await expect(interrupted).toBeFocused();
	await expect(page.getByText('“Rapports de cercle, Dori” resumed.')).toBeVisible();

	// The last one: the foot goes, and the focus goes to the job in the list, running now.
	await page.keyboard.press('Enter');
	await expect(foot).toHaveCount(0);
	const row = sidebar.getByRole('button', { name: /^Entretiens Ouagadougou Running/ });
	await expect(row).toBeFocused();
});

test('a job the engine will not resume says why at the foot and keeps the focus', async ({
	page
}) => {
	// The engine refuses (`?resume-fails`), as it does for a provider without its key.
	await open(page, 'jobs&resume-fails');
	const foot = page
		.getByRole('navigation', { name: 'Jobs' })
		.getByRole('region', { name: /^To resume/ });
	const paused = foot.getByRole('button', { name: 'Resume “Rapports de cercle, Dori”' });
	await paused.focus();
	await page.keyboard.press('Enter');
	// Said under the job, in a region read out at once (the toast keeps the window's one alert).
	const said = foot.getByText(/^No API key is saved for this provider\. Add one in Settings\./);
	await expect(said).toBeVisible();
	await expect(said).toHaveAttribute('aria-live', 'assertive');
	await expect(paused).toBeFocused();
	await expect(paused).toHaveAccessibleDescription(/No API key is saved/);
});
