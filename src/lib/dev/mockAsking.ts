// What a pretend model does with a request from the review in the browser preview, as the
// engine does it for a passage or a region: an estimate (free on this computer or when answered
// before), an answer after a moment unless stopped (a long moment with the page's `slow` flag),
// kept so the same request again costs nothing. Each kind of request checks, answers and writes
// in its own way (`mockPassages.ts`, `mockRegionReadings.ts`). Development builds only.

import type { ProposalStatus } from '../bindings/ProposalStatus.js';
import type { Timestamp } from '../bindings/Timestamp.js';

/** Refuses with the engine's error `id`. */
export const fail = (id: string): never => {
	throw { id };
};

/** The pretend model's corrections: straight apostrophes put right, a few misreadings mended. */
export function mend(text: string): string {
	return text
		.replace(/(\p{L})'(\p{L})/gu, '$1’$2')
		.replace(/fideles/g, 'fidèles')
		.replace(/Gouvemeur/g, 'Gouverneur');
}

/** What the preview keeps of a request, as the engine stores it. */
type Kept = {
	status: ProposalStatus;
	answer: string | null;
	cost_usd: number | null;
	cached: boolean;
	answered_at: Timestamp | null;
};

/** How one kind of request is answered. */
type Pace<A> = {
	/** What makes two requests the same, so the second costs nothing. */
	key: (ask: A) => string;
	/** What a request costs when it is not free. */
	cost: number;
	/** How long an answer takes, in milliseconds. */
	delay: number;
	/** The page's flag that makes the model take long enough to be stopped. */
	slow: string;
};

export class PretendModel<A extends { id: string; provider: string }, R extends Kept> {
	/** The requests asked, by id. */
	readonly asked = new Map<string, R>();
	/** Answers already given, by what was asked. */
	readonly #answered = new Map<string, string>();
	/** Requests waiting, by id, to stop waiting for. */
	readonly #waiting = new Map<string, () => void>();
	readonly #pace: Pace<A>;

	constructor(pace: Pace<A>) {
		this.#pace = pace;
	}

	/** What `ask` would cost, and whether it is answered already. */
	estimate(ask: A, local: (provider: string) => boolean) {
		const offline = local(ask.provider);
		const cached = this.#answered.has(this.#pace.key(ask));
		return { usd: offline || cached ? null : this.#pace.cost, offline, cached };
	}

	/**
	 * Keeps `kept` as `ask`'s record and answers it: the answer given before, else `read()`'s;
	 * `settle` adds what this kind of request records of how it ended (its cost, its notes).
	 */
	ask(
		ask: A,
		kept: R,
		read: () => string,
		settle: (status: 'answered' | 'cancelled', answer: string, stored?: string) => Partial<R>
	): Promise<R> {
		this.asked.set(ask.id, kept);
		const key = this.#pace.key(ask);
		const stored = this.#answered.get(key);
		const slow = new URLSearchParams(location.search).has(this.#pace.slow);
		return new Promise<R>((resolve) => {
			const done = (status: 'answered' | 'cancelled') => {
				this.#waiting.delete(ask.id);
				clearTimeout(timer);
				const answer = stored ?? read();
				if (status === 'answered') this.#answered.set(key, answer);
				Object.assign(kept, {
					status,
					answer: status === 'answered' ? answer : null,
					...settle(status, answer, stored),
					cached: stored !== undefined,
					answered_at: Date.now()
				});
				resolve({ ...kept });
			};
			const timer = setTimeout(
				() => done('answered'),
				stored !== undefined ? 0 : slow ? 60_000 : this.#pace.delay
			);
			this.#waiting.set(ask.id, () => done('cancelled'));
		});
	}

	/** Stops waiting for request `id`. */
	cancel(id: string): null {
		this.#waiting.get(id)?.();
		return null;
	}

	/** Leaves request `id`'s answer unused. */
	discard(id: string): null {
		const kept = this.asked.get(id);
		if (kept?.status === 'answered') kept.status = 'discarded';
		return null;
	}
}
