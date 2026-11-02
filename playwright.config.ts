// The interface in WebKit, the engine of macOS's webview, against the dev server's sample data
// (`?mock`): CI runs it on a Mac (`npm run test:webkit`), since the maintainers build on
// Windows, where the app runs in Chromium. See docs/development.md.

import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
	testDir: 'e2e',
	globalSetup: './e2e/warm-up.ts',
	forbidOnly: !!process.env.CI,
	retries: process.env.CI ? 1 : 0,
	reporter: process.env.CI ? [['github'], ['list']] : 'list',
	use: { baseURL: 'http://localhost:5173' },
	projects: [{ name: 'webkit', use: { ...devices['Desktop Safari'] } }],
	webServer: {
		command: 'npm run dev -- --port 5173 --strictPort',
		url: 'http://localhost:5173',
		reuseExistingServer: !process.env.CI
	}
});
