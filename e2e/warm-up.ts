// The dev server bundles the dependencies on the first visit and reloads the page once they are
// ready, logging errors meanwhile. One visit before the tests, so they see the app as it runs.

import { webkit, type FullConfig } from '@playwright/test';

export default async function warmUp(config: FullConfig): Promise<void> {
	const baseURL = config.projects[0]?.use.baseURL ?? 'http://localhost:5173';
	const browser = await webkit.launch();
	const page = await browser.newPage();
	for (const view of ['', '&view=review-audio', '&view=settings']) {
		await page.goto(`${baseURL}/?mock${view}`);
		await page.waitForLoadState('networkidle');
	}
	await page.waitForTimeout(2000);
	await browser.close();
}
