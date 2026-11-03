// Export profiles for the browser preview (`?mock&view=project`, then Export descriptions): one
// saved profile for an Omeka S import, kept in memory. The check mirrors Rust's
// (`ExportProfile::check`) closely enough to show its messages. Development builds only.

import type { ExportProfile } from '../bindings/ExportProfile.js';
import type { ProfileColumn } from '../bindings/ProfileColumn.js';
import type { ProfileError } from '../bindings/ProfileError.js';
import { FIELDS } from '../described.js';

const EVERY: ProfileColumn[] = [
	{ heading: 'document_id', value: { kind: 'document_id' } },
	{ heading: 'title', value: { kind: 'title' } },
	{ heading: 'file_name', value: { kind: 'file_name' } },
	{ heading: 'path', value: { kind: 'path' } },
	{ heading: 'collections', value: { kind: 'collections' } },
	{ heading: 'tags', value: { kind: 'tags' } },
	...FIELDS.flatMap((field): ProfileColumn[] => [
		{ heading: field, value: { kind: 'part', field } },
		{ heading: `${field}_from`, value: { kind: 'from', field } }
	])
];

const DUBLIN_CORE: ProfileColumn[] = [
	{ heading: 'dcterms:identifier', value: { kind: 'part', field: 'shelfmark' } },
	{ heading: 'dcterms:title', value: { kind: 'title' } },
	{ heading: 'dcterms:creator', value: { kind: 'part', field: 'creator' } },
	{ heading: 'dcterms:date', value: { kind: 'part', field: 'date' } },
	{ heading: 'dcterms:isPartOf', value: { kind: 'part', field: 'fonds' } },
	{ heading: 'dcterms:source', value: { kind: 'part', field: 'url' } },
	{ heading: 'dcterms:bibliographicCitation', value: { kind: 'part', field: 'citation' } },
	{ heading: 'dcterms:rights', value: { kind: 'part', field: 'rights' } },
	{ heading: 'dcterms:subject', value: { kind: 'tags' } }
];

let profiles: ExportProfile[] = [
	{
		id: 'profile-00000000-0000-0000-0000-0000000000p1',
		name: 'Omeka S import',
		columns: [
			...DUBLIN_CORE,
			{ heading: 'o:item_set', value: { kind: 'fixed', text: 'Fonds AOF' } }
		],
		list_separator: '|',
		missing: '',
		join_lines: true,
		updated_at: 1_762_000_000_000
	}
];

function check(profile: ExportProfile): ProfileError | null {
	if (!profile.name.trim()) return { kind: 'no_name' };
	if (profile.columns.length === 0) return { kind: 'no_columns' };
	const seen: string[] = [];
	for (const [i, column] of profile.columns.entries()) {
		const heading = column.heading.trim();
		if (!heading) return { kind: 'unnamed', column: i + 1 };
		if (/[\u0000-\u001f\u007f]/.test(heading)) return { kind: 'bad_heading', column: i + 1 };
		const first = seen.indexOf(heading.toLowerCase());
		if (first >= 0) return { kind: 'duplicate', column: i + 1, first: first + 1 };
		seen.push(heading.toLowerCase());
	}
	if (!profile.list_separator || /[\n\r]/.test(profile.list_separator)) {
		return { kind: 'bad_separator' };
	}
	return null;
}

function saved(profile: ExportProfile): ExportProfile {
	const id = profile.id || `profile-${crypto.randomUUID()}`;
	const out = { ...profile, id, name: profile.name.trim(), updated_at: Date.now() };
	profiles = [...profiles.filter((p) => p.id !== id), out].sort((a, b) =>
		a.name.localeCompare(b.name)
	);
	return out;
}

export function profileCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'list_export_profiles':
			return profiles;
		case 'save_export_profile': {
			const profile = a.profile as ExportProfile;
			const problem = check(profile);
			if (problem) throw { id: `profile.${problem.kind}` };
			return saved(profile);
		}
		case 'duplicate_export_profile': {
			const from = profiles.find((p) => p.id === a.id);
			if (!from) throw { id: 'prompt.not_found' };
			return saved({ ...structuredClone(from), id: '', name: String(a.name) });
		}
		case 'delete_export_profile':
			profiles = profiles.filter((p) => p.id !== a.id);
			return null;
		case 'check_export_profile':
			return check(a.profile as ExportProfile);
		case 'profile_starter':
			return structuredClone(a.starter === 'dublin_core' ? DUBLIN_CORE : EVERY);
		case 'share_export_profile':
			return null;
		case 'import_export_profile':
			return saved({ ...structuredClone(profiles[0]), id: '', name: 'Imported profile' });
		default:
			return undefined;
	}
}
