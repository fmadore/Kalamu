// The dev server bundles the dependencies on the first visit and reloads the page once they are
// ready, logging errors meanwhile. One visit before the tests, so they see the app as it runs.
// Any engine warms the server: the one installed on this machine is used (WebKit on CI's Mac,
// Chromium on its Linux machine).

import { chromium, webkit, type Browser, type FullConfig } from '@playwright/test';

async function anyBrowser(): Promise<Browser> {
	try {
		return await webkit.launch();
	} catch {
		return chromium.launch();
	}
}

export default async function warmUp(config: FullConfig): Promise<void> {
	const baseURL = config.projects[0]?.use.baseURL ?? 'http://localhost:5173';
	const browser = await anyBrowser();
	const page = await browser.newPage();
	for (const view of ['', '&view=review-audio', '&view=settings']) {
		await page.goto(`${baseURL}/?mock${view}`);
		await page.waitForLoadState('networkidle');
	}
	await page.waitForTimeout(2000);
	await browser.close();
}
