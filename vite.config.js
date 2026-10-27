import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
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

// pdf.js loads its decoders (JPEG 2000, JBIG2), colour profiles, character maps and standard
// fonts at run time from URLs it is given. Without them, scans in those formats render blank.
// Served at /pdfjs/<folder>/ in development and copied into the bundle on build.
const PDFJS_DATA = ['wasm', 'cmaps', 'standard_fonts', 'iccs'];
const pdfjsRoot = path.dirname(fileURLToPath(import.meta.resolve('pdfjs-dist/package.json')));

/** @returns {import('vite').Plugin} */
function pdfjsData() {
	return {
		name: 'pdfjs-data',
		configureServer(server) {
			server.middlewares.use('/pdfjs', (req, res, next) => {
				const [folder, file, ...rest] = (req.url ?? '').split('?')[0].split('/').filter(Boolean);
				if (!PDFJS_DATA.includes(folder) || !file || rest.length) return next();
				const full = path.join(pdfjsRoot, folder, decodeURIComponent(file));
				if (path.dirname(full) !== path.join(pdfjsRoot, folder) || !fs.existsSync(full)) {
					return next();
				}
				if (full.endsWith('.wasm')) res.setHeader('Content-Type', 'application/wasm');
				fs.createReadStream(full).pipe(res);
			});
		},
		generateBundle() {
			if (this.environment.name !== 'client') return;
			for (const folder of PDFJS_DATA) {
				for (const file of fs.readdirSync(path.join(pdfjsRoot, folder))) {
					this.emitFile({
						type: 'asset',
						fileName: `pdfjs/${folder}/${file}`,
						source: fs.readFileSync(path.join(pdfjsRoot, folder, file))
					});
				}
			}
		}
	};
}

export default defineConfig({
	plugins: [
		woff2Only(),
		pdfjsData(),
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
