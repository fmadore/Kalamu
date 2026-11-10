// The interface against the dev server's sample data (`?mock`), in the two engines it runs in:
// WebKit, the engine of macOS's webview, which CI runs on a Mac (`npm run test:webkit`) since the
// maintainers build on Windows; and Chromium, the engine of Windows' WebView2
// (`npm run test:chromium`). See docs/development.md.

import { defineConfig, devices } from '@playwright/test';

// KALAMU_E2E_PORT serves another port, so checkouts side by side do not test each other's server.
const port = Number(process.env.KALAMU_E2E_PORT ?? 5173);

export default defineConfig({
	testDir: 'e2e',
	globalSetup: './e2e/warm-up.ts',
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	reporter: process.env.CI ? [['github'], ['list']] : 'list',
	// English whatever the machine's language: the specs find elements by their English names.
	use: { baseURL: `http://localhost:${port}`, locale: 'en-US' },
	projects: [
		{ name: 'webkit', use: { ...devices['Desktop Safari'] } },
		{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }
	],
	webServer: {
		command: `npm run dev -- --port ${port} --strictPort`,
		url: `http://localhost:${port}`,
		reuseExistingServer: !process.env.CI
	}
});
