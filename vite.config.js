import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { sveltekit } from '@sveltejs/kit/vite';
// vitest's re-export, not vite's: only this one accepts the `test` block below.
import { defineConfig } from 'vitest/config';

const host = process.env.TAURI_DEV_HOST;

// Fontsource lists a `.woff` fallback after every `.woff2`; WebView2 always takes the `.woff2`.
// Dropping the fallbacks keeps them out of the bundle (from Live Translation).
/** @returns {import('vite').Plugin} */
function woff2Only() {
	return {
		name: 'woff2-only',
		enforce: 'pre',
		transform(code, id) {
			if (!/[\/]@fontsource[\/].+\.css$/.test(id)) return;
			return code.replace(/,\s*url\([^)]+\.woff\)\s*format\('woff'\)/g, '');
		}
	};
}

export default defineConfig({
	plugins: [
		woff2Only(),
		sveltekit({
			preprocess: vitePreprocess(),
			// Tauri serves a static bundle; no Node server at run time.
			adapter: adapter({ fallback: 'index.html' }),
			// A desktop app is updated by its installer; nothing to poll for.
			version: { pollInterval: 0 }
		})
	],
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		hmr: host ? { protocol: 'ws', host, port: 5183 } : undefined,
		watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**'] }
	},
	envPrefix: ['VITE_', 'TAURI_ENV_'],
	test: {
		environment: 'node',
		include: ['src/**/*.test.ts']
	}
});
