// The interface's net for failures nobody expected: an error no code caught, a promise rejected
// with nobody to handle it, a screen that broke as it was drawn (its boundary in `+page.svelte`,
// reported from `hooks.client.ts`).
// Each is sent to the shell's log (`log_interface_error`), which cuts it down to what the log
// may hold, and one message says so for a burst of them, not one per failure. Failures that are
// expected (a file that cannot be read, a provider refusing) never come here: the screens say
// them through `Action` and `describeError`.

import type { InterfaceError } from './bindings/InterfaceError.js';
import type { InterfaceErrorKind } from './bindings/InterfaceErrorKind.js';
import { describeError, isAppError } from './errors.js';
import { messages } from './i18n/index.js';
import { app, toast } from './state.svelte.js';
import { api } from './tauri.js';

/** How much of a message and a stack is sent; the shell keeps less. */
const MESSAGE_MAX = 1000;
const STACK_MAX = 4000;
/** A burst ends once this long passes without a failure: the next one is said again. */
export const BURST_MS = 10_000;
/** Different failures sent in one burst; the shell bounds the log further. */
const SENT_MAX = 20;

/** What is sent of a failure. An error gives its name, message and stack; the core's error
 * only its id, never its detail (a path, a provider's words); anything else thrown only its
 * type, since it may be text from a document. */
export function faultReport(
	kind: InterfaceErrorKind,
	value: unknown,
	screen: string
): InterfaceError {
	if (isAppError(value)) return { kind, screen, name: 'AppError', message: value.id, stack: '' };
	// An error, or what looks like one (from another realm, it fails `instanceof Error`).
	if (value instanceof Error || isErrorLike(value)) {
		return {
			kind,
			screen,
			name: String(value.name),
			message: String(value.message).slice(0, MESSAGE_MAX),
			stack: String(value.stack ?? '').slice(0, STACK_MAX)
		};
	}
	return { kind, screen, name: value === null ? 'null' : typeof value, message: '', stack: '' };
}

function isErrorLike(value: unknown): value is Error {
	if (typeof value !== 'object' || value === null) return false;
	const e = value as Partial<Record<'name' | 'message', unknown>>;
	return typeof e.name === 'string' && typeof e.message === 'string';
}

/** When the last failure came, and the different ones sent since the burst began. */
let last = -Infinity;
const sent = new Set<string>();

/** Whether a failure at `now` begins a burst, said once; every failure keeps it going. */
export function beginsBurst(now: number): boolean {
	const begins = now - last >= BURST_MS;
	last = now;
	if (begins) sent.clear();
	return begins;
}

/** Sends a failure to the log, the same one once a burst; and, at the start of a burst, says
 * that something went wrong, unless a screen's boundary already says so in its place. The core's
 * error, uncaught, is said in its own words. */
export function reportFault(kind: InterfaceErrorKind, value: unknown): void {
	const begins = beginsBurst(Date.now());
	const report = faultReport(kind, value, app.view.name);
	const key = [report.kind, report.name, report.message, report.stack.slice(0, 500)].join('\n');
	if (!sent.has(key) && sent.size < SENT_MAX) {
		sent.add(key);
		void api.logInterfaceError(report).catch(() => undefined);
	}
	if (!begins || kind === 'screen') return;
	const m = messages();
	toast(isAppError(value) ? describeError(value, m) : m.unexpected.message, { kind: 'error' });
}

/** What the webview reports as an error but is none: Chromium's notice that a resize observer
 * left a change for the next frame. */
const harmless = (message: string) => message.startsWith('ResizeObserver loop');

/** Listens on the window for failures no code caught; the function returned stops. */
export function installFaultNet(): () => void {
	const onError = (e: ErrorEvent) => {
		if (!harmless(e.message ?? '')) reportFault('error', e.error ?? null);
	};
	const onRejection = (e: PromiseRejectionEvent) => reportFault('rejection', e.reason);
	window.addEventListener('error', onError);
	window.addEventListener('unhandledrejection', onRejection);
	return () => {
		window.removeEventListener('error', onError);
		window.removeEventListener('unhandledrejection', onRejection);
	};
}
