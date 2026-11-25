// The stand-in provider: a local HTTP server that answers the test build of Kalamu (`cargo build
// --features test-overrides`, KALAMU_TEST_PROVIDERS pointing here) with the recorded answers the
// providers' own tests already hold (crates/providers/tests/fixtures), so the app can be driven
// in CI without a key or a real service. Node with no package, so CI starts it on Windows and
// macOS with the Node the front end already needs. See
// docs/decisions/2026-11-25-app-driven-in-ci.md.
//
//   node scripts/app-driven/stand-in.mjs [--port 4310]
//
// Each cloud provider is reached under /<provider id>. What it serves:
//
// - Gemini, page by page: POST /gemini/models/<model>:generateContent answers gemini_ok.json.
// - Gemini, in a batch (economy mode): POST /gemini/models/<model>:batchGenerateContent makes a
//   batch held as pending, in the reference's shape, until POST /__batches/release; then
//   GET /gemini/batches/<id> answers it succeeded, each request with gemini_ok.json inline, as
//   crates/providers/tests/batches.rs reads it.
// - OpenAI's /responses (openai_ok.json), Anthropic's /messages (anthropic_ok.json) and Mistral's
//   /ocr (mistral_ocr.json), page by page, for flows that choose them.
//
// And for the flows, under /__ (never a provider's path):
//
// - GET /__log: every provider request so far, `{ method, path, at }`;
// - POST /__delay?ms=<n>: each page's answer waits so long, so a job can be paused part way;
// - POST /__batches/release: every batch made so far answers as succeeded;
// - POST /__reset: the log, the delay and the batches forgotten.
//
// Anything else is a 404 naming the path, so a request the stand-in does not know fails the flow
// rather than passing unnoticed. It listens on 127.0.0.1 only.

import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const fixtures = join(here, '..', '..', 'crates', 'providers', 'tests', 'fixtures');

/** A recorded answer from the providers' tests, read once. */
function fixture(name) {
	return JSON.parse(readFileSync(join(fixtures, name), 'utf8'));
}

const recorded = {
	gemini: fixture('gemini_ok.json'),
	openai: fixture('openai_ok.json'),
	anthropic: fixture('anthropic_ok.json'),
	mistralOcr: fixture('mistral_ocr.json')
};

/** What one stand-in remembers: each server started has its own. */
function fresh() {
	return {
		log: [],
		delayMs: 0,
		/** @type {Map<string, { keys: string[], released: boolean }>} */
		batches: new Map(),
		made: 0
	};
}

function send(res, status, body) {
	const text = JSON.stringify(body);
	res.writeHead(status, {
		'content-type': 'application/json; charset=utf-8',
		'content-length': Buffer.byteLength(text)
	});
	res.end(text);
}

function readBody(req) {
	return new Promise((resolve, reject) => {
		const chunks = [];
		req.on('data', (c) => chunks.push(c));
		req.on('end', () => resolve(Buffer.concat(chunks).toString('utf8')));
		req.on('error', reject);
	});
}

const wait = (ms) => new Promise((r) => setTimeout(r, ms));

/** The keys a batch's inline requests were sent under (`metadata.key`), in order. */
function batchKeys(body) {
	try {
		const parsed = JSON.parse(body);
		const requests = parsed?.batch?.input_config?.requests?.requests ?? [];
		return requests.map((r, i) => r?.metadata?.key ?? `p${i}`);
	} catch {
		return [];
	}
}

/** A batch in the reference's shape: pending, or succeeded with its answers inline. */
function batchStatus(name, batch) {
	if (!batch.released) {
		return {
			name,
			metadata: {
				state: 'BATCH_STATE_PENDING',
				batchStats: { requestCount: String(batch.keys.length) }
			}
		};
	}
	return {
		name,
		metadata: {
			state: 'BATCH_STATE_SUCCEEDED',
			output: {
				inlinedResponses: {
					inlinedResponses: batch.keys.map((key) => ({
						response: recorded.gemini,
						metadata: { key }
					}))
				}
			}
		},
		done: true
	};
}

async function provider(state, req, res, path, body) {
	state.log.push({ method: req.method, path, at: Date.now() });
	const post = req.method === 'POST';
	let m;
	if (post && (m = path.match(/^\/gemini\/models\/([^/:]+):generateContent$/))) {
		if (state.delayMs > 0) await wait(state.delayMs);
		return send(res, 200, recorded.gemini);
	}
	if (post && (m = path.match(/^\/gemini\/models\/([^/:]+):batchGenerateContent$/))) {
		state.made += 1;
		const name = `batches/stand-in-${state.made}`;
		const batch = { keys: batchKeys(body), released: false };
		state.batches.set(name, batch);
		return send(res, 200, batchStatus(name, batch));
	}
	if (req.method === 'GET' && (m = path.match(/^\/gemini\/(batches\/[^/]+)$/))) {
		const batch = state.batches.get(m[1]);
		if (!batch) return send(res, 404, { error: { code: 404, message: `${m[1]} not found` } });
		return send(res, 200, batchStatus(m[1], batch));
	}
	if (req.method === 'DELETE' && /^\/gemini\/files\/[^/]+$/.test(path)) {
		return send(res, 200, {});
	}
	if (post && path === '/openai/responses') {
		if (state.delayMs > 0) await wait(state.delayMs);
		return send(res, 200, recorded.openai);
	}
	if (post && path === '/anthropic/messages') {
		if (state.delayMs > 0) await wait(state.delayMs);
		return send(res, 200, recorded.anthropic);
	}
	if (post && path === '/mistral/ocr') {
		if (state.delayMs > 0) await wait(state.delayMs);
		return send(res, 200, recorded.mistralOcr);
	}
	return send(res, 404, { error: { code: 404, message: `the stand-in does not serve ${path}` } });
}

function control(state, req, res, url) {
	const path = url.pathname;
	if (req.method === 'GET' && path === '/__log') return send(res, 200, state.log);
	if (req.method === 'POST' && path === '/__delay') {
		state.delayMs = Math.max(0, Number(url.searchParams.get('ms') ?? 0) || 0);
		return send(res, 200, { delayMs: state.delayMs });
	}
	if (req.method === 'POST' && path === '/__batches/release') {
		for (const batch of state.batches.values()) batch.released = true;
		return send(res, 200, { released: state.batches.size });
	}
	if (req.method === 'POST' && path === '/__reset') {
		state.log = [];
		state.delayMs = 0;
		state.batches.clear();
		return send(res, 200, {});
	}
	return send(res, 404, { error: `no control at ${path}` });
}

/** Starts the stand-in on `port` (0 for any free one); resolves to the server once it listens. */
export function start(port = 4310) {
	const state = fresh();
	/** One request answered; a failure is a 500 naming it. */
	async function answer(req, res) {
		try {
			const url = new URL(req.url ?? '/', 'http://127.0.0.1');
			const body = await readBody(req);
			if (url.pathname.startsWith('/__')) return control(state, req, res, url);
			return await provider(state, req, res, url.pathname, body);
		} catch (e) {
			send(res, 500, { error: { code: 500, message: String(e) } });
		}
	}
	const server = createServer((req, res) => void answer(req, res));
	return new Promise((resolve, reject) => {
		server.once('error', reject);
		server.listen(port, '127.0.0.1', () => resolve(server));
	});
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
	const at = process.argv.indexOf('--port');
	const port = at > 0 ? Number(process.argv[at + 1]) : 4310;
	const server = await start(port);
	const address = server.address();
	console.log(`stand-in provider at http://127.0.0.1:${address.port}`);
}
