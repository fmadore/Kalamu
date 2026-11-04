// What a screen does when the researcher asks it to change something: the work runs, the
// screen reads again what it changed, and what went wrong is said in the interface language,
// in the screen's own banner or passed up to the one that shows it. Every screen that changes
// stored things goes through one of these, so each says busy and fails the same way.

import { describeError } from './errors.js';
import { messages } from './i18n/index.js';

export interface ActionOptions {
	/** Runs after each work that succeeded, while still busy: reading again what it changed. */
	after?: () => unknown;
	/**
	 * Where the sentence goes when something fails, for a part whose screen shows the errors.
	 * Without it, the sentence is kept in `error`, cleared as the next work starts.
	 */
	report?: (message: string) => void;
}

export class Action {
	/** True while a work and what follows it run. */
	busy = $state(false);
	/** What went wrong last, as a sentence; null once the next work starts. */
	error = $state<string | null>(null);
	readonly #after: ((result: unknown) => unknown) | undefined;
	readonly #report: ((message: string) => void) | undefined;

	constructor({ after, report }: ActionOptions = {}) {
		this.#after = after;
		this.#report = report;
	}

	/**
	 * Runs `work`, then what follows it: `after` when given (null for nothing), else the one the
	 * action was made with. Returns whether both succeeded; a failure is said, never thrown.
	 */
	async run<T>(work: () => Promise<T>, after?: ((result: T) => unknown) | null): Promise<boolean> {
		this.busy = true;
		if (!this.#report) this.error = null;
		try {
			const result = await work();
			await (after === undefined ? this.#after : after)?.(result);
			return true;
		} catch (e) {
			this.fail(e);
			return false;
		} finally {
			this.busy = false;
		}
	}

	/** Says what went wrong, as the core's error or as given. */
	fail(e: unknown): void {
		const message = describeError(e, messages());
		if (this.#report) this.#report(message);
		else this.error = message;
	}
}
