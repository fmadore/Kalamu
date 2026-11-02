// A sample dictionary for the browser preview (`?mock&view=project`, then Dictionary): names the
// sample analyses might list, a few spelled several ways. The rules are the dictionary crate's,
// simplified. Development builds only.

import type { DictionaryRow } from '../bindings/DictionaryRow.js';
import type { DictionaryView } from '../bindings/DictionaryView.js';
import type { Evidence } from '../bindings/Evidence.js';
import type { Item } from '../bindings/Item.js';
import type { Proposal } from '../bindings/Proposal.js';
import type { ProposalMember } from '../bindings/ProposalMember.js';
import type { ProposalsView } from '../bindings/ProposalsView.js';
import { folded } from '../search.js';

interface MockEntry {
	id: string;
	namespace: string;
	label: string;
	forms: string[];
	reviewed: boolean;
}

/** Every form the sample analyses listed, with how many documents name it. */
const found: [string, string, number][] = [
	['people', 'Moussa Traoré', 3],
	['people', 'Traore Moussa', 1],
	['people', 'El Hadj Oumar Tall', 2],
	['people', 'Oumar Tall', 1],
	['people', 'Père Éric de Rosny', 1],
	['people', 'Imam Diallo', 2],
	['places', 'Bamako', 3],
	['places', 'Kankan', 2],
	['places', 'Côte d’Ivoire', 2],
	['places', 'Ivory Coast', 1],
	['organisations', 'Union culturelle musulmane', 2],
	['organisations', 'UCM', 1]
];
const labels: Record<string, string> = {
	people: 'People',
	places: 'Places',
	organisations: 'Organisations'
};

let entries: MockEntry[] = [
	{
		id: '00000000-0000-0000-0000-0000000000n1',
		namespace: 'places',
		label: 'Bamako',
		forms: ['Bamako'],
		reviewed: true
	}
];
let apart: [string, string][] = [];
let next = 2;

/** What a model answered when asked for merges: groups of forms, with its label and reason. */
const answered: [string, string[], string, string, string][] = [
	[
		'people',
		['Moussa Traoré', 'Traore Moussa'],
		'Moussa Traoré',
		'One name in two orders, the accent dropped in one; both letters are from Bamako.',
		'certain'
	],
	[
		'people',
		['El Hadj Oumar Tall', 'Oumar Tall'],
		'El Hadj Oumar Tall',
		'The same name with and without the title, in documents of the same years.',
		'likely'
	],
	[
		'places',
		['Côte d’Ivoire', 'Ivory Coast'],
		'Côte d’Ivoire',
		'The French and English names of one country.',
		'certain'
	],
	[
		'organisations',
		['Union culturelle musulmane', 'UCM'],
		'Union culturelle musulmane',
		'UCM is the usual abbreviation, and the documents that use it name its Bamako office.',
		'likely'
	]
];
let asked = new Set<string>();
let dismissed = new Set<string>();
const request = '00000000-0000-0000-0000-0000000000r1';
const proposalJob = '00000000-0000-0000-0000-0000000000j9';

function member(namespace: string, form: string): ProposalMember {
	const entry = holder(namespace, form);
	return entry
		? {
				item: { kind: 'entry', id: entry.id },
				key: entry.id,
				label: entry.label,
				forms: entry.forms
			}
		: { item: { kind: 'form', form }, key: `form:${form}`, label: form, forms: [form] };
}

function proposals(): ProposalsView {
	const out: Proposal[] = [];
	for (const [namespace, forms, label, reason, certainty] of answered) {
		if (!asked.has(namespace)) continue;
		const members = [
			...new Map(forms.map((f) => [member(namespace, f).key, member(namespace, f)])).values()
		];
		if (members.length < 2) continue;
		const ids = members.flatMap((mb) => (mb.item.kind === 'entry' ? [mb.item.id] : []));
		if (apart.some(([a, b]) => ids.includes(a) && ids.includes(b))) continue;
		const key = members
			.flatMap((mb) => mb.forms)
			.sort()
			.join('\u001f');
		out.push({
			key,
			namespace,
			request_id: request,
			job_id: proposalJob,
			label,
			reason,
			certainty,
			members,
			dismissed: dismissed.has(key)
		});
	}
	return {
		requests: [...asked].map((namespace) => ({
			id: `${request}-${namespace}`,
			namespace,
			job_id: proposalJob,
			job_name: `Dictionary: merges in ${labels[namespace]}`,
			status: 'completed',
			created_at: Date.now(),
			rows: view().rows.filter((r) => r.namespace === namespace).length,
			groups: 1,
			answered: 1,
			proposals: out.filter((x) => x.namespace === namespace && !x.dismissed).length,
			unknown: 0
		})),
		proposals: out
	};
}

function holder(namespace: string, form: string): MockEntry | undefined {
	return entries.find((e) => e.namespace === namespace && e.forms.includes(form));
}

function documentsOf(namespace: string, form: string): number {
	return found.find(([n, f]) => n === namespace && f === form)?.[2] ?? 0;
}

function keys(form: string): string[] {
	const plain = folded(form)
		.replace(/[^\p{L}\p{N}]+/gu, ' ')
		.trim();
	const words = plain.split(' ');
	return words.length > 1 ? [plain, [...words].sort().join(' ')] : [plain];
}

function view(): DictionaryView {
	const rows: DictionaryRow[] = [];
	const row = (
		key: string,
		entry: string | null,
		namespace: string,
		label: string,
		forms: string[],
		reviewed: boolean
	): DictionaryRow => {
		const views = forms.map((form) => ({ form, documents: documentsOf(namespace, form) }));
		const documents = views.reduce((n, f) => n + f.documents, 0);
		return {
			key,
			entry,
			namespace,
			label,
			forms: views,
			documents,
			listed: documents,
			read: documents + (label.startsWith('Oumar') ? 1 : 0),
			reviewed,
			unseen: documents === 0,
			apart: entry
				? apart.filter((p) => p.includes(entry)).map(([a, b]) => (a === entry ? b : a))
				: [],
			similar: []
		};
	};
	for (const e of entries) rows.push(row(e.id, e.id, e.namespace, e.label, e.forms, e.reviewed));
	for (const [namespace, form] of found)
		if (!holder(namespace, form))
			rows.push(row(`form:${form}`, null, namespace, form, [form], false));
	for (const r of rows) {
		const mine = r.forms.flatMap((f) => keys(f.form));
		r.similar = rows
			.filter(
				(o) =>
					o !== r &&
					o.namespace === r.namespace &&
					!r.apart.includes(o.key) &&
					o.forms.some((f) => keys(f.form).some((k) => mine.includes(k)))
			)
			.map((o) => o.key);
	}
	rows.sort((a, b) => a.namespace.localeCompare(b.namespace) || a.label.localeCompare(b.label));
	return {
		namespaces: Object.entries(labels).map(([key, label]) => ({
			key,
			label,
			rows: rows.filter((r) => r.namespace === key).length,
			to_review: rows.filter((r) => r.namespace === key && !r.reviewed).length
		})),
		rows,
		documents_read: 3,
		documents: 4
	};
}

function materialise(namespace: string, item: Item): MockEntry {
	if (item.kind === 'entry') {
		const entry = entries.find((e) => e.id === item.id);
		if (!entry) throw { id: 'dictionary.no_entry' };
		return entry;
	}
	const held = holder(namespace, item.form);
	if (held) return held;
	const entry = {
		id: `00000000-0000-0000-0000-0000000000n${next++}`,
		namespace,
		label: item.form,
		forms: [item.form],
		reviewed: false
	};
	entries.push(entry);
	return entry;
}

export function dictionaryCommand(cmd: string, a: Record<string, unknown>): unknown {
	const namespace = String(a.namespace ?? '');
	switch (cmd) {
		case 'dictionary_view':
			return view();
		case 'dictionary_evidence':
			return (a.forms as string[]).map((form, i): Evidence => ({
				document_id: '00000000-0000-0000-0000-0000000000d1',
				document: 'Bulletin, May 1957',
				job_id: '00000000-0000-0000-0000-0000000000j3',
				job_name: 'Bulletin, people and places',
				source_id: '00000000-0000-0000-0000-0000000000s1',
				setup_label: i === 0 ? 'gemini-3.8-flash' : 'claude-sonnet-5-5',
				created_at: Date.now(),
				forms: [form]
			}));
		case 'dictionary_merge': {
			const chosen = (a.items as Item[]).map((item) => materialise(namespace, item));
			if (chosen.some((x) => chosen.some((y) => apart.some(([p, q]) => p === x.id && q === y.id))))
				throw { id: 'dictionary.kept_apart' };
			const [kept, ...rest] = chosen;
			kept.forms = [...new Set(chosen.flatMap((e) => e.forms))];
			kept.label = String(a.label).trim();
			kept.reviewed = true;
			entries = entries.filter((e) => !rest.includes(e));
			return kept;
		}
		case 'dictionary_split': {
			const entry = entries.find((e) => e.id === a.entry);
			if (entry) entry.forms = entry.forms.filter((f) => !(a.forms as string[]).includes(f));
			entries = entries.filter((e) => e.forms.length > 0);
			return null;
		}
		case 'dictionary_relabel': {
			const entry = entries.find((e) => e.id === a.entry);
			if (entry) entry.label = String(a.label).trim();
			return entry;
		}
		case 'dictionary_set_reviewed':
			for (const item of a.items as Item[]) {
				if (!a.reviewed && item.kind === 'form') continue;
				materialise(namespace, item).reviewed = Boolean(a.reviewed);
			}
			return null;
		case 'dictionary_keep_apart': {
			const x = materialise(namespace, a.a as Item);
			const y = materialise(namespace, a.b as Item);
			if (x === y) throw { id: 'dictionary.same_entry' };
			// Once, whichever way given, as the store keeps it.
			if (!apart.some(([p, q]) => [p, q].includes(x.id) && [p, q].includes(y.id)))
				apart.push([x.id, y.id]);
			return null;
		}
		case 'dictionary_undo_apart':
			apart = apart.filter(([p, q]) => !([a.a, a.b].includes(p) && [a.a, a.b].includes(q)));
			return null;
		case 'export_dictionary':
			return null;
		case 'dictionary_plan': {
			const rows = view().rows.filter((r) => r.namespace === namespace).length;
			if (rows < 2) throw { id: 'dictionary.nothing_to_compare' };
			return {
				sources: [
					{
						id: '00000000-0000-0000-0000-0000000000s9',
						path: `C:/Kalamu/snapshots/${namespace}.txt`,
						file_name: `${namespace}.txt`,
						sha256: namespace,
						size_bytes: rows * 120,
						modified_ms: null,
						kind: 'text',
						page_count: null,
						duration_ms: null,
						split_spreads: false,
						even_out_volume: false
					}
				],
				rows
			};
		}
		case 'dictionary_propose':
			asked.add(namespace);
			return { id: proposalJob, name: String(a.name) };
		case 'dictionary_proposals':
			return proposals();
		case 'dictionary_dismiss':
			if (a.dismissed) dismissed.add(String(a.key));
			else dismissed.delete(String(a.key));
			return null;
		case 'dictionary_forget_request':
			asked = new Set([...asked].filter((n) => `${request}-${n}` !== a.request));
			return null;
		default:
			return undefined;
	}
}
