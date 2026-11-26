// Servers of the researcher's own in the browser preview, as the shell keeps them
// (`commands/servers.rs`): a list in memory, an Ollama found at any address (four models: one
// that reads pictures, one that reads only text, one run at ollama.com, one with a long name
// whose server says nothing of pictures), each model tried after
// a moment unless stopped, saved only once tried, and merged into the catalogue as providers and
// models the way `Catalog::with_servers` does. Development builds only.
//
// Consent to a server on the network, and the jobs New job makes with a server's model, are
// kept in `mockServerJobs.ts`: a server whose address changes is asked about again, and one
// that a running job uses cannot be moved, lose that job's model or be removed.
//
// Addresses: `nowhere` in the host answers nothing, `locked` asks for a key (and refuses
// `wrong`), `web` is not a server Kalamu knows. `?trial-slow`: a trial takes long enough to be
// stopped.

import { emit } from '@tauri-apps/api/event';
import type { Catalog } from '../bindings/Catalog.js';
import type { FoundServer } from '../bindings/FoundServer.js';
import type { FoundServerModel } from '../bindings/FoundServerModel.js';
import type { ModelInfo } from '../bindings/ModelInfo.js';
import type { ModelTrial } from '../bindings/ModelTrial.js';
import type { OwnModel } from '../bindings/OwnModel.js';
import type { OwnServer } from '../bindings/OwnServer.js';
import type { ProviderInfo } from '../bindings/ProviderInfo.js';
import type { ServerDraft } from '../bindings/ServerDraft.js';
import type { TaskRules } from '../bindings/TaskRules.js';
import type { Task } from '../bindings/Task.js';
import type { TrialModel } from '../bindings/TrialModel.js';
import type { TrialServer } from '../bindings/TrialServer.js';
import { modelsInUse, serverJobCommand, withdraw } from './mockServerJobs.js';

/** Refuses with the shell's error `id` and the values its sentence names. */
const refuse = (id: string, params?: Record<string, string>): never => {
	throw params ? { id, params } : { id };
};

const servers: OwnServer[] = [];
/** Trials made, by address and model. */
const trials = new Map<string, { reads_pages: boolean; fills_forms: boolean; tried_at: string }>();
/** Trials under way, by the interface's id, to stop. */
const running = new Map<string, () => void>();

const trialKey = (address: string, model: string) => JSON.stringify([address, model]);

/** The address as the shell keeps it, roughly: trimmed, `http(s)://`, no last slash. */
function checked(address: string): string {
	const typed = address.trim();
	if (!typed) refuse('server.address_empty');
	if (!/^https?:\/\//i.test(typed)) refuse('server.address_scheme');
	let url: URL;
	try {
		url = new URL(typed);
	} catch {
		return refuse('server.address_unreadable');
	}
	if (url.username || url.password) refuse('server.address_credentials');
	if (url.search || url.hash) refuse('server.address_query');
	return url.href.replace(/\/+$/, '');
}

const hostOf = (address: string) => new URL(address).hostname.replace(/^\[|\]$/g, '');
const thisComputer = (host: string) =>
	host === 'localhost' || host.startsWith('127.') || host === '::1';

function found(address: string, key: string | null, editing: string | null): FoundServer {
	const host = hostOf(address);
	const shown = new URL(address).host;
	if (host.includes('nowhere')) refuse('server.nothing_answers', { host: shown });
	if (host.includes('web')) refuse('server.not_a_server', { host: shown });
	if (host.includes('locked') && !key) refuse('server.key_needed', { host: shown });
	if (host.includes('locked') && key === 'wrong') refuse('server.key_refused', { host: shown });
	const model = (id: string, rest: Partial<FoundServerModel>): FoundServerModel => ({
		id,
		vision: false,
		audio: false,
		trained_context: 131_072,
		elsewhere: null,
		size_bytes: null,
		digest: `sha256:${id.length.toString(16).padStart(8, '0')}`,
		context: 32_768,
		max_output: 8_192,
		added: false,
		...rest
	});
	const models = [
		model('gemma3:12b', { vision: true, size_bytes: 8_149_190_253 }),
		model('llama3.1:8b', { size_bytes: 4_920_753_328 }),
		model('gpt-oss:120b-cloud', { elsewhere: 'https://ollama.com:443', trained_context: null }),
		// A long name from Hugging Face, of which the server says nothing about pictures.
		model('hf.co/bartowski/Qwen2.5-VL-7B-Instruct-GGUF:Q4_K_M', {
			vision: null,
			size_bytes: 4_683_074_016
		})
	];
	const server = editing
		? servers.find((s) => s.id === editing)
		: servers.find((s) => s.address === address);
	for (const m of models) {
		const added = server?.models.find((o) => o.id === m.id);
		if (added)
			Object.assign(m, { added: true, context: added.context, max_output: added.max_output });
	}
	return {
		dialect: 'ollama',
		address,
		version: '0.40.18',
		host,
		this_computer: thisComputer(host),
		models
	};
}

function tryModel(trial: string, server: TrialServer, model: TrialModel): Promise<ModelTrial> {
	const address = checked(server.address);
	const slow = new URLSearchParams(location.search).has('trial-slow');
	return new Promise((resolve, reject) => {
		const timer = setTimeout(
			() => {
				running.delete(trial);
				const tried_at = new Date().toISOString().slice(0, 10);
				// The text-only model fills the form; the one that reads pictures does both.
				const result = {
					reads_pages: model.vision !== false,
					fills_forms: true,
					tried_at
				};
				trials.set(trialKey(address, model.id), result);
				resolve({
					...result,
					failure: null,
					words: null,
					note: model.vision === false ? `${server.name} says this model reads no pictures.` : null
				});
			},
			slow ? 8000 : 1200
		);
		running.set(trial, () => {
			clearTimeout(timer);
			reject({ id: 'server.trial_stopped', params: { model: model.id } });
		});
	});
}

function save(draft: ServerDraft, key: string | null): OwnServer {
	const address = checked(draft.address);
	const name = draft.name.trim();
	const earlier = draft.id ? servers.find((s) => s.id === draft.id) : undefined;
	if (draft.id && !earlier) refuse('server.not_found', { id: draft.id });
	if (!name) refuse('server.no_name', { server: draft.id ?? '' });
	if (servers.some((s) => s.id !== draft.id && s.name.toLowerCase() === name.toLowerCase()))
		refuse('server.same_name', { name });
	if (draft.asks_key && !key && !earlier?.key) refuse('server.key_missing', { server: name });
	// A running job keeps its server's address, way of speaking and models, as the shell has it.
	const inUse = earlier ? modelsInUse(earlier.id) : [];
	if (
		earlier &&
		inUse.length > 0 &&
		(earlier.address !== address ||
			earlier.dialect !== draft.dialect ||
			inUse.some((model) => !draft.models.some((m) => m.id === model)))
	)
		refuse('server.in_use', { server: earlier.name });
	const models: OwnModel[] = draft.models.map((m) => {
		const stored =
			earlier?.address === address ? earlier.models.find((o) => o.id === m.id) : undefined;
		const trial = trials.get(trialKey(address, m.id)) ?? stored;
		if (!trial) return refuse('server.not_tried', { model: m.id });
		if (!trial.reads_pages && !trial.fills_forms) refuse('server.did_nothing', { model: m.id });
		return {
			id: m.id,
			reads_pages: trial.reads_pages,
			fills_forms: trial.fills_forms,
			context: m.context,
			max_output: m.max_output,
			...(m.input_per_mtok != null ? { input_per_mtok: m.input_per_mtok } : {}),
			...(m.output_per_mtok != null ? { output_per_mtok: m.output_per_mtok } : {}),
			...(trial.tried_at ? { tried_at: trial.tried_at } : {})
		};
	});
	const server: OwnServer = {
		id:
			earlier?.id ??
			`own-${Math.floor(Math.random() * 0xffffffff)
				.toString(16)
				.padStart(8, '0')}`,
		name,
		address,
		dialect: draft.dialect,
		key: draft.asks_key,
		at_once: draft.at_once,
		models
	};
	if (earlier) servers.splice(servers.indexOf(earlier), 1, server);
	else servers.push(server);
	// Moved: asked consent for again.
	if (earlier && earlier.address !== address) withdraw(server.id);
	return server;
}

/** The commands of Settings › Other servers; `undefined` for any other. */
export function serverCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'list_servers':
			return structuredClone(servers);
		case 'find_server':
			return found(
				checked(String(a.address ?? '')),
				(a.key as string | null) ?? null,
				(a.id as string | null) ?? null
			);
		case 'try_server_model':
			return tryModel(String(a.trial), a.server as TrialServer, a.model as TrialModel);
		case 'cancel_server_trial':
			running.get(String(a.trial))?.();
			running.delete(String(a.trial));
			return null;
		case 'save_server': {
			const saved = save(a.server as ServerDraft, (a.key as string | null) ?? null);
			// As the shell does once the engine is handed the server.
			void emit('catalog-changed', null);
			return saved;
		}
		case 'remove_server': {
			const at = servers.findIndex((s) => s.id === a.id);
			if (at < 0) refuse('server.not_found', { id: String(a.id) });
			if (modelsInUse(servers[at].id).length > 0)
				refuse('server.in_use', { server: servers[at].name });
			withdraw(servers[at].id);
			servers.splice(at, 1);
			void emit('catalog-changed', null);
			return null;
		}
		default:
			return serverJobCommand(cmd, a, (id) => servers.find((s) => s.id === id));
	}
}

/** `catalog` with the servers added, as `Catalog::with_servers` adds them: each server a
 *  provider, each model offered for what its trial showed, one page a call. */
export function withServers(catalog: Catalog): Catalog {
	const template = catalog.models.find((m) => m.tasks.includes('text_extraction'));
	if (!template || servers.length === 0) return catalog;
	const providers: ProviderInfo[] = servers.map((s) => {
		const host = hostOf(s.address);
		return {
			id: s.id,
			name: s.name,
			key_url: '',
			key_prefix: null,
			local: false,
			own: {
				address: s.address,
				host,
				this_computer: thisComputer(host),
				key: s.key,
				dialect: s.dialect
			}
		};
	});
	const models: ModelInfo[] = servers.flatMap((s) =>
		s.models
			.filter((m) => m.reads_pages || m.fills_forms)
			.map((m) => {
				const model: ModelInfo = {
					...template,
					provider: s.id,
					id: m.id,
					name: m.id,
					description: '',
					tasks: [
						...(m.reads_pages ? (['text_extraction'] as const) : []),
						...(m.fills_forms ? (['analysis'] as const) : [])
					],
					tier: 'balanced',
					inputs: [
						...(m.fills_forms ? ['text/plain'] : []),
						...(m.reads_pages ? ['image/png', 'image/jpeg'] : [])
					],
					recommended_for: [],
					provider_default_for: [],
					costly: false,
					pricing: {
						input_per_mtok: m.input_per_mtok ?? null,
						output_per_mtok: m.output_per_mtok ?? null,
						per_minute: null,
						per_page: null,
						note: null
					},
					limits: {
						max_request_mb: null,
						max_pages: 1,
						max_audio_minutes: null,
						max_output_tokens: m.max_output,
						max_input_tokens: m.context - m.max_output
					}
				};
				delete model.summary;
				delete model.reasoning;
				delete model.page_thinking;
				delete model.translations;
				delete model.successor;
				return model;
			})
	);
	return {
		...catalog,
		providers: [...catalog.providers, ...providers],
		models: [...catalog.models, ...models]
	};
}

/** Whether `provider` is a server that asks for no key: always usable, as the shell says. */
export const keyless = (provider: string): boolean =>
	servers.some((s) => s.id === provider && !s.key);

/** The tasks a server's model is offered for, as the engine's rules read its entry: reading
 *  pages, text extraction; filling a form, analyses, corrections and translations; both, tables
 *  of contents too. */
function tasksOf(m: OwnModel): Task[] {
	return [
		...(m.reads_pages ? (['text_extraction'] as const) : []),
		...(m.fills_forms ? (['analysis', 'text_correction', 'translation'] as const) : []),
		...(m.reads_pages && m.fills_forms ? (['contents'] as const) : [])
	];
}

/** `rules` with the servers' models in them, as the shell's `task_rules` reads the merged
 *  catalogue: each for what its trial showed, a model that fills forms combining a table of
 *  contents' pages. */
export function withServerRules(rules: TaskRules[]): TaskRules[] {
	if (servers.length === 0) return rules;
	const ref = (provider: string, model: string) => ({
		provider,
		model,
		recommended: false,
		provider_default: false
	});
	return rules.map((rule) => {
		const models = servers.flatMap((s) =>
			s.models.filter((m) => tasksOf(m).includes(rule.task)).map((m) => ref(s.id, m.id))
		);
		const merges =
			rule.task === 'contents'
				? servers.flatMap((s) => s.models.filter((m) => m.fills_forms).map((m) => ref(s.id, m.id)))
				: [];
		return {
			...rule,
			models: [...rule.models, ...models],
			merge_models: [...rule.merge_models, ...merges]
		};
	});
}

/** A server's model, which has no batch service and costs only what the researcher set. */
export function serverPrice(provider: string, model: string): number | null | undefined {
	const server = servers.find((s) => s.id === provider);
	if (!server) return undefined;
	const found = server.models.find((m) => m.id === model);
	if (found?.input_per_mtok == null || found.output_per_mtok == null) return null;
	// 120 pages of about 1,500 tokens read and 600 written, as the sample job counts them.
	return (120 * (1500 * found.input_per_mtok + 600 * found.output_per_mtok)) / 1e6;
}
