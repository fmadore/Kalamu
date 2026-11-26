import { readFileSync } from 'node:fs';
import { afterEach, describe, expect, it } from 'vitest';
import type { Catalog } from './bindings/Catalog.js';
import type { FoundServer } from './bindings/FoundServer.js';
import type { OwnServer } from './bindings/OwnServer.js';
import type { ProviderInfo } from './bindings/ProviderInfo.js';
import { staysHere } from './catalogText.js';
import { en, setLocale } from './i18n/index.js';
import { fr } from './i18n/fr.js';
import {
	consentMessage,
	draftOfSaved,
	foundWords,
	keptModel,
	offeredFor,
	priceOf,
	serverDraft,
	serverSummary,
	unencrypted
} from './servers.js';

const bundled = JSON.parse(readFileSync('crates/catalog/catalog.json', 'utf8')) as Catalog;

function server(id: string, name: string, host: string, here: boolean): ProviderInfo {
	return {
		id,
		name,
		key_url: '',
		key_prefix: null,
		local: false,
		own: {
			address: `http://${host}:11434`,
			host,
			this_computer: here,
			key: false,
			dialect: 'ollama'
		}
	};
}

const catalog: Catalog = {
	...bundled,
	providers: [
		...bundled.providers,
		server('own-0000000a', 'Desk', 'localhost', true),
		server('own-0000000b', 'Lab', '192.168.1.20', false)
	]
};

const found: FoundServer = {
	dialect: 'ollama',
	address: 'http://192.168.1.20:11434',
	version: '0.40.18',
	host: '192.168.1.20',
	this_computer: false,
	models: []
};

const saved: OwnServer = {
	id: 'own-0000000b',
	name: 'Lab',
	address: 'http://192.168.1.20:11434',
	dialect: 'ollama',
	key: true,
	at_once: 2,
	models: [
		{
			id: 'gemma3:12b',
			reads_pages: true,
			fills_forms: true,
			context: 32768,
			max_output: 8192,
			input_per_mtok: 0.1,
			tried_at: '2026-11-26'
		},
		{ id: 'llama3.1:8b', reads_pages: false, fills_forms: true, context: 16384, max_output: 4096 }
	]
};

afterEach(() => setLocale('en'));

describe('where a provider’s files go', () => {
	it('stays on this computer for an engine inside Kalamu and a server here, never otherwise', () => {
		expect(staysHere(catalog, 'tesseract')).toBe(true);
		expect(staysHere(catalog, 'own-0000000a')).toBe(true);
		expect(staysHere(catalog, 'own-0000000b')).toBe(false);
		expect(staysHere(catalog, 'anthropic')).toBe(false);
		expect(staysHere(catalog, 'unknown')).toBe(false);
		expect(staysHere(null, 'tesseract')).toBe(false);
	});

	it('names what a server is and where it is', () => {
		expect(foundWords(en, found)).toBe('Ollama 0.40.18, on the network at 192.168.1.20.');
		expect(
			foundWords(en, { ...found, dialect: 'open_ai', version: null, this_computer: true })
		).toBe('An OpenAI-style server, on this computer.');
		expect(serverSummary(en, catalog.providers.at(-2)!.own!)).toBe('Ollama on this computer');
		expect(serverSummary(fr, catalog.providers.at(-1)!.own!)).toContain('sur le réseau');
	});

	it('warns of plain http only for another computer', () => {
		expect(unencrypted(found)).toBe(true);
		expect(unencrypted({ ...found, address: 'https://uni.example/v1' })).toBe(false);
		expect(unencrypted({ address: 'http://localhost:11434', this_computer: true })).toBe(false);
	});
});

describe('the consent asked', () => {
	it('keeps the providers’ sentence as it was when no server is among them', () => {
		const { title, message } = consentMessage(['anthropic'], catalog, en, 'POLICY');
		expect(title).toBe(en.consent.title);
		expect(message).toBe(en.consent.ask('Anthropic Claude', 'POLICY'));
	});

	it('says a server on the network is run by someone under their own rules, named and placed', () => {
		const { title, message } = consentMessage(['own-0000000b'], catalog, en, 'POLICY');
		expect(title).toBe(en.consent.titleServer);
		expect(message).toContain('the server “Lab” at 192.168.1.20');
		expect(message).toContain('their own rules');
		expect(message).not.toContain('under your own account and API key');
	});

	it('reads right for a mix of providers and servers', () => {
		setLocale('en');
		const { title, message } = consentMessage(['anthropic', 'own-0000000b'], catalog, en, 'P');
		expect(title).toBe(en.consent.titleBoth);
		expect(message).toContain('Anthropic Claude, and to the server “Lab” at 192.168.1.20');
		expect(message).toContain('under your own account and API key');
		const french = consentMessage(['own-0000000b'], catalog, fr, 'P').message;
		expect(french).toContain('vos fichiers au serveur «\u00a0Lab\u00a0» à l’adresse 192.168.1.20');
	});
});

describe('a server saved', () => {
	it('keeps a model’s stored trial only for the server changed, at the same address and dialect', () => {
		expect(keptModel(saved, found, 'gemma3:12b')?.reads_pages).toBe(true);
		expect(keptModel(saved, found, 'qwen3-vl:8b')).toBeUndefined();
		expect(keptModel(null, found, 'gemma3:12b')).toBeUndefined();
		expect(
			keptModel(saved, { ...found, address: 'http://192.168.1.21:11434' }, 'gemma3:12b')
		).toBeUndefined();
		expect(keptModel(saved, { ...found, dialect: 'open_ai' }, 'gemma3:12b')).toBeUndefined();
	});

	it('says what a model is offered for from its trial', () => {
		expect(offeredFor(en, { reads_pages: true, fills_forms: false })).toBe('text extraction');
		expect(offeredFor(en, { reads_pages: false, fills_forms: true })).toBe(
			'analyses, corrections and translations'
		);
		expect(offeredFor(en, { reads_pages: true, fills_forms: true })).toContain(
			'tables of contents'
		);
		expect(offeredFor(en, { reads_pages: false, fills_forms: false })).toBe('');
	});

	it('reads a price typed, empty or not a number being none', () => {
		expect(priceOf('')).toBeNull();
		expect(priceOf(null)).toBeNull();
		expect(priceOf('0,25')).toBe(0.25);
		expect(priceOf(3)).toBe(3);
		expect(priceOf('abc')).toBeNull();
	});

	it('drafts a server with its requests at once kept from 1 to 8', () => {
		const draft = serverDraft({
			editing: null,
			name: '  Lab ',
			found,
			asksKey: false,
			atOnce: 20,
			models: []
		});
		expect(draft).toMatchObject({ id: null, name: 'Lab', at_once: 8, dialect: 'ollama' });
		expect(draft.address).toBe(found.address);
	});

	it('saves a server again as it is, leaving out a model that now does nothing', () => {
		const draft = draftOfSaved(saved, ['llama3.1:8b']);
		expect(draft).toMatchObject({ id: saved.id, asks_key: true, at_once: 2 });
		expect(draft.models).toEqual([
			{
				id: 'gemma3:12b',
				context: 32768,
				max_output: 8192,
				input_per_mtok: 0.1,
				output_per_mtok: null
			}
		]);
	});
});
