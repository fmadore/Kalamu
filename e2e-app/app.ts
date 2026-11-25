// The built app, started and driven: the test build of Kalamu (`--features test-overrides`), in a
// data folder of its own, every cloud provider reached at the stand-in provider, and its WebView2
// window driven over the Chrome DevTools protocol that WebView2 opens when the test build asks
// it to (KALAMU_TEST_DEVTOOLS_PORT), which is what Edge's WebDriver itself drives. Windows
// only: WKWebView offers neither. See docs/decisions/2026-11-25-app-driven-in-ci.md.

import { execFileSync, spawn, type ChildProcess } from 'node:child_process';
import {
	copyFileSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readdirSync,
	readFileSync,
	writeFileSync
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import type { Server } from 'node:http';
import { chromium, type Browser, type Page } from '@playwright/test';
// @ts-expect-error: a plain Node script without types.
import { start as startStandIn } from '../scripts/app-driven/stand-in.mjs';

const repo = resolve(import.meta.dirname, '..');

/** The test build's executable: KALAMU_APP, else the debug build in the workspace's target. */
export function executable(): string {
	const exe = process.env.KALAMU_APP ?? join(repo, 'target', 'debug', 'kalamu.exe');
	if (!existsSync(exe)) {
		throw new Error(
			`${exe} not found: build it with \`npm run build\` then ` +
				'`cargo build -p kalamu --features custom-protocol,test-overrides`'
		);
	}
	return exe;
}

/** The stand-in provider, started once per test file on a free port. */
export class StandIn {
	private constructor(
		private server: Server,
		readonly url: string
	) {}

	static async start(): Promise<StandIn> {
		const server: Server = await startStandIn(0);
		const address = server.address();
		const port = typeof address === 'object' && address ? address.port : 0;
		return new StandIn(server, `http://127.0.0.1:${port}`);
	}

	async log(): Promise<{ method: string; path: string; at: number }[]> {
		return (await fetch(`${this.url}/__log`)).json();
	}

	/** How many page answers were asked for so far. */
	async pagesAsked(): Promise<number> {
		return (await this.log()).filter((r) => r.path.endsWith(':generateContent')).length;
	}

	async delay(ms: number): Promise<void> {
		await fetch(`${this.url}/__delay?ms=${ms}`, { method: 'POST' });
	}

	async releaseBatches(): Promise<void> {
		await fetch(`${this.url}/__batches/release`, { method: 'POST' });
	}

	stop(): Promise<void> {
		return new Promise((done) => this.server.close(() => done()));
	}
}

/** A test run's folders: the app's data, and the sources it reads and writes beside. */
export interface Folders {
	data: string;
	work: string;
}

/** A data folder with English chosen, the model list never refreshed, and consent to send to
 *  Gemini already given (the consent dialog is native, out of a webview driver's reach); and a
 *  work folder holding copies of the named sample files. */
export function folders(samples: string[]): Folders {
	const root = mkdtempSync(join(tmpdir(), 'kalamu-app-'));
	const data = join(root, 'data');
	const work = join(root, 'work');
	mkdirSync(data, { recursive: true });
	mkdirSync(work, { recursive: true });
	writeFileSync(
		join(data, 'settings.json'),
		JSON.stringify({
			language: 'en',
			refresh_catalog: false,
			ai_consent: ['gemini'],
			concurrency: 1
		})
	);
	for (const name of samples) copyFileSync(join(repo, 'samples', name), join(work, name));
	return { data, work };
}

/** The files in the work folder whose name starts with `stem`. */
export function written(work: string, stem: string): string[] {
	return readdirSync(work)
		.filter((name) => name.startsWith(stem))
		.sort();
}

/** The file in the data folder the test build's dialogs take their answers from, in order
 *  (`src-tauri/src/test_overrides.rs`). */
const DIALOG_ANSWERS = 'dialog-answers.json';

/** One launch of the app: its process, the browser connection and the window's page. */
export class App {
	private constructor(
		private process: ChildProcess,
		private browser: Browser,
		readonly page: Page,
		private exited: Promise<number | null>,
		private data: string,
		private port: number
	) {}

	/** Waits until WebView2's browser process has let go of the port, and with it of the data
	 *  folder's webview profile, which the next launch on the same folder opens. */
	private async released(): Promise<void> {
		const deadline = Date.now() + 30_000;
		while (Date.now() < deadline) {
			try {
				await fetch(`http://127.0.0.1:${this.port}/json/version`);
			} catch {
				return;
			}
			await new Promise((r) => setTimeout(r, 500));
		}
	}

	/** Starts the app on `dirs.data`, its providers at `standIn`, and connects to its window.
	 *  Each launch takes a port of its own: WebView2's browser process outlives its app by a few
	 *  seconds, and a second launch on the same port would reach the old one. */
	static async launch(dirs: Folders, standIn: StandIn): Promise<App> {
		const port = nextPort++;
		writeFileSync(join(dirs.data, DIALOG_ANSWERS), '[]');
		const child = spawn(executable(), [], {
			env: {
				...process.env,
				KALAMU_TEST_DATA: dirs.data,
				KALAMU_TEST_PROVIDERS: standIn.url,
				KALAMU_TEST_DEVTOOLS_PORT: String(port),
				RUST_LOG: 'kalamu=info,kalamu_lib=info,kalamu_engine=info,warn'
			},
			stdio: ['ignore', 'inherit', 'inherit']
		});
		const exited = new Promise<number | null>((done) => child.once('exit', (code) => done(code)));
		const browser = await connect(port, dirs.data, exited);
		const page = await windowPage(browser);
		await page.getByRole('navigation', { name: 'Jobs' }).waitFor();
		return new App(child, browser, page, exited, dirs.data, port);
	}

	/** Queues the answers the next native dialogs give, which the test build reads in place of
	 *  opening a window no webview driver reaches: a path or a list of paths for a file picker, a
	 *  path or null for a save dialog, true or false for a question. */
	answerDialogs(...answers: unknown[]): void {
		const file = join(this.data, DIALOG_ANSWERS);
		const queued: unknown[] = existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : [];
		writeFileSync(file, JSON.stringify([...queued, ...answers]));
	}

	/** The answers queued and not yet taken. */
	dialogsLeft(): unknown[] {
		return JSON.parse(readFileSync(join(this.data, DIALOG_ANSWERS), 'utf8'));
	}

	/** A command of the app's, called as the interface calls it. */
	invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
		return this.page.evaluate(
			([c, a]) =>
				(
					window as unknown as {
						__TAURI_INTERNALS__: { invoke: (c: string, a: unknown) => Promise<unknown> };
					}
				).__TAURI_INTERNALS__.invoke(c as string, a) as Promise<never>,
			[cmd, args] as const
		);
	}

	/** Quits as *Quit now* does (running jobs paused, pieces sent waited for) and waits for the
	 *  process to end; its exit code. */
	async quit(): Promise<number | null> {
		// The call does not return: the process ends under it.
		this.invoke('quit_app').catch(() => {});
		const code = await Promise.race([
			this.exited,
			new Promise<'timeout'>((r) => setTimeout(() => r('timeout'), 60_000))
		]);
		await this.browser.close().catch(() => {});
		if (code === 'timeout') {
			this.process.kill();
			await this.released();
			throw new Error('the app did not end within a minute of being quit');
		}
		await this.released();
		return code;
	}

	/** Ends the process however it stands, for a test's clean-up. */
	async kill(): Promise<void> {
		await this.browser.close().catch(() => {});
		if (this.process.exitCode === null && this.process.signalCode === null) this.process.kill();
		await this.exited;
		await this.released();
	}
}

/** The DevTools port of the next launch. */
let nextPort = 9333;

/** The port WebView2 says it listens on (the `DevToolsActivePort` file its browser process
 *  writes in its profile, under the data folder), when it wrote one. */
function activePort(data: string): number | undefined {
	for (const dir of [join(data, 'EBWebView'), data]) {
		const file = join(dir, 'DevToolsActivePort');
		if (!existsSync(file)) continue;
		const port = Number(readFileSync(file, 'utf8').split(/\r?\n/)[0]);
		if (Number.isInteger(port) && port > 0) return port;
	}
	return undefined;
}

/** What the machine shows when the webview cannot be reached, for the log of a failed run:
 *  WebView2's version, its processes with the arguments they were started with, the ports they
 *  listen on, and what the data folder holds. */
function diagnosis(data: string): string {
	const script = [
		"$ErrorActionPreference = 'SilentlyContinue'",
		"'WebView2: ' + (Get-ItemProperty 'HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}').pv",
		"'Policy: ' + ((Get-ItemProperty 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Edge' | Out-String).Trim())",
		"'Policy (WebView2): ' + ((Get-ChildItem 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Edge\\WebView2' -Recurse | Out-String).Trim())",
		"$p = Get-CimInstance Win32_Process | Where-Object { $_.Name -in 'msedgewebview2.exe','kalamu.exe' }",
		"$p | ForEach-Object { '' + $_.ProcessId + ' ' + $_.Name + ' ' + $_.CommandLine }",
		"Get-NetTCPConnection -State Listen | Where-Object { $p.ProcessId -contains $_.OwningProcess } | ForEach-Object { 'listening: ' + $_.LocalAddress + ':' + $_.LocalPort + ' pid ' + $_.OwningProcess }"
	].join('; ');
	let machine: string;
	try {
		machine = execFileSync('powershell', ['-NoProfile', '-Command', script], {
			encoding: 'utf8',
			timeout: 60_000
		});
	} catch (e) {
		machine = `the machine could not be asked: ${String(e)}`;
	}
	const list = (dir: string) => (existsSync(dir) ? readdirSync(dir).join(', ') : 'absent');
	return [
		machine.trim(),
		`data folder: ${list(data)}`,
		`EBWebView: ${list(join(data, 'EBWebView'))}`,
		`DevToolsActivePort: ${activePort(data) ?? 'not written'}`
	].join('\n');
}

/** Connects over CDP once WebView2 listens, or fails when the app ended first. The port asked
 *  for is tried at each of the loopback's addresses, then the one WebView2 wrote down. */
async function connect(
	port: number,
	data: string,
	exited: Promise<number | null>
): Promise<Browser> {
	let ended: number | null | undefined;
	void exited.then((code) => (ended = code));
	const deadline = Date.now() + 90_000;
	let last: unknown;
	while (Date.now() < deadline) {
		if (ended !== undefined) throw new Error(`the app ended at launch (exit code ${ended})`);
		const written = activePort(data);
		const addresses = [`127.0.0.1:${port}`, `[::1]:${port}`];
		if (written !== undefined && written !== port) addresses.push(`127.0.0.1:${written}`);
		for (const address of addresses) {
			try {
				return await chromium.connectOverCDP(`http://${address}`, { timeout: 5_000 });
			} catch (e) {
				last = e;
			}
		}
		await new Promise((r) => setTimeout(r, 500));
	}
	throw new Error(
		`could not reach the app's webview on port ${port}: ${String(last)}\n${diagnosis(data)}`
	);
}

/** The main window's page, once it shows the app (not WebView2's blank start). */
async function windowPage(browser: Browser): Promise<Page> {
	const deadline = Date.now() + 60_000;
	while (Date.now() < deadline) {
		for (const context of browser.contexts()) {
			for (const page of context.pages()) {
				if (/^(https?:\/\/tauri\.localhost|tauri:\/\/localhost)/.test(page.url())) return page;
			}
		}
		await new Promise((r) => setTimeout(r, 250));
	}
	throw new Error('the app opened no window');
}
