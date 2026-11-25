// The built app driven against the stand-in provider (`e2e-app/`), on Windows: the test build
// started for each flow and its WebView2 window driven over CDP. One flow at a time, since each
// starts the app. See docs/development.md and docs/decisions/2026-11-25-app-driven-in-ci.md.
//
//   npm run build
//   cargo build -p kalamu --features custom-protocol,test-overrides
//   npm run test:app

import { defineConfig } from '@playwright/test';

export default defineConfig({
	testDir: 'e2e-app',
	forbidOnly: !!process.env.CI,
	workers: 1,
	fullyParallel: false,
	retries: process.env.CI ? 1 : 0,
	// A flow starts the app up to three times, and a debug build is slow to start.
	timeout: 300_000,
	expect: { timeout: 20_000 },
	reporter: process.env.CI ? [['github'], ['list']] : 'list',
	use: { trace: 'retain-on-failure' },
	outputDir: 'test-results/app'
});
