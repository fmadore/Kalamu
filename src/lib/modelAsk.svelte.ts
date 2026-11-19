// Asking a model from the review, as the passage dialog and the region dialog both do it: the
// model chosen (provider, model, prompt preset; whether its key is there and it is installed)
// and the request's life (what it would cost, the consent, the wait that can be stopped, the
// answer with its changes marked, edited first, then used or discarded). Each dialog keeps what
// is its own: what it sends, where an answer goes, and how it is drawn. Nothing here changes a
// text: the engine records the request and its answer, and only the researcher's choice writes.
import type { DiffSpan } from './bindings/DiffSpan.js';
import type { PassageEstimate } from './bindings/PassageEstimate.js';
import type { Preset } from './bindings/Preset.js';
import type { ProposalStatus } from './bindings/ProposalStatus.js';
import { providerName as providerNameOf } from './catalogText.js';
import { askConsent } from './dialogs.js';
import { describeError, isAppError } from './errors.js';
import { messages } from './i18n/index.js';
import { rulesFor } from './newJob.js';
import { findModel, modelsFor, providerDefault } from './newJobModels.js';
import { app, isInstalled } from './state.svelte.js';
import { api } from './tauri.js';

/** A model chosen to ask, as it is remembered between requests. */
export type Setup = { provider: string; model: string; preset: string | null };

/** A remembered set-up, if `v` is one. */
export function readSetup(v: unknown): Setup | null {
	const s = v as Partial<Setup> | null;
	return typeof s?.provider === 'string' && typeof s.model === 'string'
		? { provider: s.provider, model: s.model, preset: s.preset ?? null }
		: null;
}

/** The tasks a review asks a model to do over a part of what it shows. */
export type AskTask = 'text_correction' | 'text_extraction';

/** The provider, model and prompt preset a request goes to, and whether it can go. */
export class ModelChoice {
	readonly task: AskTask;
	readonly inputs: string[];
	readonly #usable: (provider: string) => boolean;
	provider = $state('');
	model = $state('');
	preset = $state('');
	presets = $state<Preset[]>([]);

	readonly catalog = $derived(app.catalog);
	readonly rule = $derived.by(() => rulesFor(app.rules, this.task));
	/** The providers with a model that does the task over these inputs. */
	readonly providers = $derived(
		(this.catalog?.providers ?? []).filter(
			(pr) => this.catalog && modelsFor(this.catalog, this.rule, pr.id, this.inputs).length > 0
		)
	);
	readonly models = $derived.by(() =>
		this.catalog ? modelsFor(this.catalog, this.rule, this.provider, this.inputs) : []
	);
	readonly providerName = $derived.by(() => {
		const found = this.catalog?.providers.find((pr) => pr.id === this.provider);
		return found ? providerNameOf(found) : this.provider;
	});
	readonly modelName = $derived(
		(this.catalog ? findModel(this.catalog, this.provider, this.model)?.name : undefined) ??
			this.model
	);
	readonly local = $derived(
		this.catalog?.providers.find((pr) => pr.id === this.provider)?.local === true
	);
	/** The provider can be asked: its key is saved (or, where `usable` says so, it is local). */
	readonly keyed = $derived.by(() => this.#usable(this.provider));
	readonly installed = $derived(isInstalled(this.provider, this.model));

	/**
	 * Starts from the first of `candidates` still offered, else the first usable provider (or the
	 * first at all) and its usual model; `preset` stands where the start names none.
	 */
	constructor(
		task: AskTask,
		inputs: string[],
		usable: (provider: string) => boolean,
		candidates: (Setup | null)[],
		preset: string
	) {
		this.task = task;
		this.inputs = inputs;
		this.#usable = usable;
		const start = candidates.find((s) => s !== null && this.offers(s)) ?? this.#fallback();
		this.provider = start.provider;
		this.model = start.model;
		this.preset = start.preset ?? preset;
	}

	/** Whether `s`'s model is still offered for the task. */
	offers(s: Setup): boolean {
		return (
			this.catalog !== null &&
			modelsFor(this.catalog, this.rule, s.provider, this.inputs).some((x) => x.id === s.model)
		);
	}

	#fallback(): Setup {
		const first = this.providers.find((pr) => this.#usable(pr.id)) ?? this.providers[0];
		const id = first?.id ?? '';
		return { provider: id, model: this.#usualModel(id), preset: null };
	}

	#usualModel(provider: string): string {
		return this.catalog ? providerDefault(this.catalog, this.rule, provider, this.inputs) : '';
	}

	/** Another provider chosen: its usual model with it. */
	providerChanged(): void {
		if (this.catalog) this.model = this.#usualModel(this.provider);
	}

	/** Reads the task's prompt presets, for the prompt picker. */
	loadPresets(): void {
		void api
			.listPresets(this.task)
			.then((list) => (this.presets = list))
			.catch(() => (this.presets = []));
	}
}

/** What the engine keeps of a request: enough to tell how it ended. */
export type Answered = {
	ask: { id: string };
	status: ProposalStatus;
	answer: string | null;
	error: string | null;
};

/** What a dialog says as a request changes. */
export type RequestTexts = {
	waiting: (model: string) => string;
	ready: string;
	stopped: string;
	failed: (reason: string) => string;
	changed: string;
};

/** The engine's commands for one kind of request, once it is asked. */
export type RequestCalls<B> = {
	cancel: (id: string) => Promise<unknown>;
	accept: (id: string, text: string) => Promise<B>;
	discard: (id: string) => Promise<unknown>;
	/** The engine's error when the text under the request changed meanwhile. */
	changed: string;
};

/** Asking, waiting for the answer, the answer shown, or the text found changed on use. */
export type Stage = 'ask' | 'waiting' | 'answer' | 'changed';

/** One request's life, from its cost to its answer used or left. */
export class ModelRequest<A extends Answered, B> {
	stage = $state<Stage>('ask');
	estimate = $state<PassageEstimate | null>(null);
	/** Why the request would be refused, from the engine: a text changed since, say. */
	estimateError = $state<string | null>(null);
	error = $state<string | null>(null);
	/** The researcher stopped waiting for the last request. */
	stopped = $state(false);
	answer = $state<A | null>(null);
	/** The answer's changes against the words it would replace; none when it replaces none. */
	spans = $state<DiffSpan[]>([]);
	editing = $state(false);
	draft = $state('');
	/** Read out as it changes. */
	said = $state('');
	/** The request being waited for. */
	#asking: string | null = null;
	readonly #m = messages();
	readonly #texts: RequestTexts;
	readonly #calls: RequestCalls<B>;

	constructor(texts: RequestTexts, calls: RequestCalls<B>) {
		this.#texts = texts;
		this.#calls = calls;
	}

	/** The text an answer used would write: the answer, or the researcher's edit of it. */
	get text(): string {
		return this.editing ? this.draft : (this.answer?.answer ?? '');
	}

	/**
	 * Asks what the request would cost, and whether it is answered already, while it is being
	 * asked; nothing when `load` is null. For an effect: returns what forgets a late reply.
	 */
	estimateWith(load: (() => Promise<PassageEstimate>) | null): (() => void) | undefined {
		this.estimateError = null;
		if (this.stage !== 'ask' || !load) {
			this.estimate = null;
			return;
		}
		let gone = false;
		load()
			.then((e) => {
				if (!gone) this.estimate = e;
			})
			.catch((e) => {
				if (gone) return;
				this.estimate = null;
				this.estimateError = describeError(e, this.#m);
			});
		return () => (gone = true);
	}

	/**
	 * Asks, once the provider is consented to: `remember` keeps the choice, `send` sends the
	 * request under its new id, and an answer is marked against `base` where it replaces words.
	 */
	async ask(o: {
		provider: string;
		modelName: string;
		remember: () => void;
		send: (id: string) => Promise<A>;
		base: string | null;
	}): Promise<void> {
		this.error = null;
		this.stopped = false;
		if (!(await askConsent([o.provider], app.catalog))) return;
		o.remember();
		const id = crypto.randomUUID();
		this.#asking = id;
		this.stage = 'waiting';
		this.said = this.#texts.waiting(o.modelName);
		try {
			const answer = await o.send(id);
			if (answer.status === 'answered' && answer.answer !== null) {
				this.answer = answer;
				this.draft = answer.answer;
				this.spans =
					o.base !== null ? await api.diffTexts(o.base, answer.answer).catch(() => []) : [];
				this.stage = 'answer';
				this.said = this.#texts.ready;
			} else if (answer.status === 'cancelled') {
				this.stage = 'ask';
				this.stopped = true;
				this.said = this.#texts.stopped;
			} else {
				this.stage = 'ask';
				this.error = this.#texts.failed(answer.error ?? '');
				this.said = '';
			}
		} catch (e) {
			this.stage = 'ask';
			this.error = describeError(e, this.#m);
			this.said = '';
		} finally {
			this.#asking = null;
		}
	}

	/** Stops waiting; the engine keeps an answer that still comes. */
	stop(): void {
		if (this.#asking) void this.#calls.cancel(this.#asking);
	}

	/** Writes the answer, or its edit, and hands on what it replaced. */
	async accept(onaccepted: (id: string, before: B) => void): Promise<void> {
		const answer = this.answer;
		if (!answer) return;
		this.error = null;
		try {
			const before = await this.#calls.accept(answer.ask.id, this.text);
			onaccepted(answer.ask.id, before);
		} catch (e) {
			if (isAppError(e) && e.id === this.#calls.changed) {
				this.stage = 'changed';
				this.said = this.#texts.changed;
			} else {
				this.error = describeError(e, this.#m);
			}
		}
	}

	/** Leaves the answer unused, and closes. */
	discard(onclose: () => void): void {
		if (this.answer) void this.#calls.discard(this.answer.ask.id).catch(() => undefined);
		onclose();
	}
}
