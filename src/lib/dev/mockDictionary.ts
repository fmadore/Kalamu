// A sample dictionary for the browser preview (`?mock&view=project`, then Dictionary): names the
// sample analyses might list, a few spelled several ways. The rules are the dictionary crate's,
// simplified. Development builds only.

import type { DictionaryRow } from '../bindings/DictionaryRow.js';
import type { DictionaryView } from '../bindings/DictionaryView.js';
import type { Evidence } from '../bindings/Evidence.js';
import type { Item } from '../bindings/Item.js';
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
		default:
			return undefined;
	}
}
