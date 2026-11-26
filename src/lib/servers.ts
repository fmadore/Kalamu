// Settings › Other servers, kept out of the components so it can be tested on its own: how a
// server is named in words (what it is, where it is), what a model's trial means for what it is
// offered, which ticked models must be tried and which keep the trial stored with the server,
// the draft saved, and the consent asked before files go to a server on the network. What a
// model does is never decided here: its trial says it, and the shell keeps it.

import type { Catalog } from './bindings/Catalog.js';
import type { Dialect } from './bindings/Dialect.js';
import type { FoundServer } from './bindings/FoundServer.js';
import type { ModelDraft } from './bindings/ModelDraft.js';
import type { ModelTrial } from './bindings/ModelTrial.js';
import type { OwnInfo } from './bindings/OwnInfo.js';
import type { OwnServer } from './bindings/OwnServer.js';
import type { ServerDraft } from './bindings/ServerDraft.js';
import { providerNameOf } from './catalogText.js';
import { listNames, type Messages } from './i18n/index.js';

/** The most requests a server is sent at the same time, as the shell allows (`MOST_AT_ONCE`). */
export const MOST_AT_ONCE = 8;

/** Where a server is, in words: on this computer, or on the network at its host. */
export function whereWords(m: Messages, at: { this_computer: boolean; host: string }): string {
	return at.this_computer ? m.servers.thisComputer : m.servers.network(at.host);
}

/** What a server found is, in one line: Ollama and its version, or an OpenAI-style server. */
export function foundWords(m: Messages, found: FoundServer): string {
	const where = whereWords(m, found);
	return found.dialect === 'ollama'
		? m.servers.foundOllama(found.version ?? '', where)
		: m.servers.foundOpenAi(where);
}

/** A server's model, in the model lists, which give it no description: what serves it, where. */
export function serverSummary(
	m: Messages,
	own: Pick<OwnInfo, 'dialect' | 'this_computer' | 'host'>
): string {
	return summaryOf(m, own.dialect, whereWords(m, own));
}

function summaryOf(m: Messages, dialect: Dialect, where: string): string {
	return dialect === 'ollama' ? m.servers.summaryOllama(where) : m.servers.summaryOpenAi(where);
}

/** Over plain `http://` to another computer: the files, and a key, travel unencrypted. */
export function unencrypted(found: Pick<FoundServer, 'address' | 'this_computer'>): boolean {
	return !found.this_computer && /^http:\/\//i.test(found.address);
}

/** The host a model runs at when not on the server (`https://ollama.com:443` → `ollama.com`). */
export function hostOf(address: string): string {
	try {
		return new URL(address).hostname || address;
	} catch {
		return address;
	}
}

/** What the server says of a model's pictures. */
export function visionWords(m: Messages, vision: boolean | null): string {
	return vision === true
		? m.servers.readsImages
		: vision === false
			? m.servers.noImages
			: m.servers.imagesUnknown;
}

/** What a model is offered for, from what its trial showed; empty when it did neither. */
export function offeredFor(
	m: Messages,
	does: { reads_pages: boolean; fills_forms: boolean }
): string {
	const o = m.servers.offered;
	return does.reads_pages && does.fills_forms
		? o.both
		: does.reads_pages
			? o.pages
			: does.fills_forms
				? o.forms
				: '';
}

/** A trial answered, and the model read the picture or filled the form: it can be saved. */
export function didSomething(trial: ModelTrial): boolean {
	return trial.failure === null && (trial.reads_pages || trial.fills_forms);
}

/**
 * Whether the trials stored with the server still hold for what was found: the server is being
 * changed, at the same address, spoken to the same way. The shell keeps them then, and a model
 * already added need not be tried again.
 */
export function storedTrialsHold(editing: OwnServer | null, found: FoundServer): boolean {
	return editing !== null && editing.address === found.address && editing.dialect === found.dialect;
}

/** A model already added whose stored trial holds: kept as it was, not tried again. */
export function keptModel(
	editing: OwnServer | null,
	found: FoundServer,
	id: string
): OwnServer['models'][number] | undefined {
	return storedTrialsHold(editing, found) ? editing?.models.find((o) => o.id === id) : undefined;
}

/** What *More choices* holds for a model, as typed. */
export interface ModelChoices {
	context: number;
	max_output: number;
	/** Empty or none for no price. */
	input_price: number | string | null;
	output_price: number | string | null;
}

/** A price typed: none when empty or not a number. */
export function priceOf(typed: string | number | null | undefined): number | null {
	if (typed === null || typed === undefined) return null;
	const text = String(typed).trim().replace(',', '.');
	if (text === '') return null;
	const n = Number(text);
	return Number.isFinite(n) ? n : null;
}

/** A model as the draft saves it. */
export function modelDraft(id: string, c: ModelChoices): ModelDraft {
	return {
		id,
		context: Math.round(Number(c.context)),
		max_output: Math.round(Number(c.max_output)),
		input_per_mtok: priceOf(c.input_price),
		output_per_mtok: priceOf(c.output_price)
	};
}

/** The server as Settings saves it. */
export function serverDraft(o: {
	editing: OwnServer | null;
	name: string;
	found: FoundServer;
	asksKey: boolean;
	atOnce: number;
	models: ModelDraft[];
}): ServerDraft {
	return {
		id: o.editing?.id ?? null,
		name: o.name.trim(),
		address: o.found.address,
		dialect: o.found.dialect,
		asks_key: o.asksKey,
		at_once: Math.min(MOST_AT_ONCE, Math.max(1, Math.round(Number(o.atOnce)) || 1)),
		models: o.models
	};
}

/** A saved server as a draft, each model as it is: to save it again after one is tried again. */
export function draftOfSaved(server: OwnServer, leaveOut: string[] = []): ServerDraft {
	return {
		id: server.id,
		name: server.name,
		address: server.address,
		dialect: server.dialect,
		asks_key: server.key,
		at_once: server.at_once,
		models: server.models
			.filter((model) => !leaveOut.includes(model.id))
			.map((model) => ({
				id: model.id,
				context: model.context,
				max_output: model.max_output,
				input_per_mtok: model.input_per_mtok ?? null,
				output_per_mtok: model.output_per_mtok ?? null
			}))
	};
}

/** The consent asked before files go to providers and servers, in words that fit the mix. */
export function consentMessage(
	needed: string[],
	catalog: Catalog | null,
	m: Messages,
	policy: string
): { title: string; message: string } {
	const own = needed.flatMap((id) => {
		const provider = catalog?.providers.find((p) => p.id === id);
		return provider?.own && !provider.own.this_computer
			? [m.consent.serverNamed(provider.name, provider.own.host)]
			: [];
	});
	const cloud = needed
		.filter((id) => {
			const provider = catalog?.providers.find((p) => p.id === id);
			return !(provider?.own && !provider.own.this_computer);
		})
		.map((id) => (catalog ? providerNameOf(catalog, id) : id));
	if (own.length === 0)
		return { title: m.consent.title, message: m.consent.ask(listNames(cloud), policy) };
	if (cloud.length === 0)
		return {
			title: m.consent.titleServer,
			message: m.consent.askServers(listNames(own), policy)
		};
	return {
		title: m.consent.titleBoth,
		message: m.consent.askBoth(listNames(cloud), listNames(own), policy)
	};
}
