// The sample project's list of what to check, for the browser preview (`?mock&view=queue`): the
// bulletin read by two models that disagree, one page cut off, the interview with a failed
// segment, and the analysis read before its text was corrected. Only the documents the queue
// covers are listed and counted: the project's, a collection's or those chosen, as
// `Projects::queue_plan` chooses them. A sample drawn at random takes the interview's segments
// nothing flags, in order (Rust ranks them by the seed); items kept aside are matched as Rust
// matches them, by their answer; results set aside are kept per scope, as the project keeps
// them. Development builds only.

import type { KeptAside } from '../bindings/KeptAside.js';
import type { QueueCounts } from '../bindings/QueueCounts.js';
import type { QueueItem } from '../bindings/QueueItem.js';
import type { QueueItemKey } from '../bindings/QueueItemKey.js';
import type { QueueSelection } from '../bindings/QueueSelection.js';
import type { QueueView } from '../bindings/QueueView.js';
import type { ReasonCounts } from '../bindings/ReasonCounts.js';
import type { ResultKey } from '../bindings/ResultKey.js';
import type { SampleSpec } from '../bindings/SampleSpec.js';
import { analysisJob, audioJob, job } from './mockJobs.js';
import { DOCUMENT_ID, documentsIn } from './mockProjects.js';

const INTERVIEW = '00000000-0000-0000-0000-0000000000d2';
const now = Date.now();

function item(over: Partial<QueueItem> & Pick<QueueItem, 'signals'>): QueueItem {
	return {
		document_id: DOCUMENT_ID,
		document: 'Bulletin, May 1957',
		job_id: job.id,
		job_name: job.name,
		task: job.task,
		source_id: job.sources[0].id,
		setup_id: job.setups[0].id,
		setup_label: job.setups[0].label,
		model: job.setups[0].model,
		preferred: false,
		piece: null,
		index: null,
		revision: '3f9a1c07be52',
		answered_at: now - 3_600_000,
		reviewed: false,
		corrected: false,
		processing: false,
		open: true,
		decided_at: null,
		sample: false,
		kept: null,
		...over
	};
}

/** The sample asked for in each scope, by `QueueSelection::key`'s rule, the results set aside
 * from each, and the items kept aside, as the project keeps them. */
const samples = new Map<string, SampleSpec>();
const setAside = new Map<string, ResultKey[]>();
let kept: KeptAside[] = [];

function scopeKey(selection: QueueSelection): string {
	if (selection.kind === 'collection') return `collection:${selection.id}`;
	if (selection.kind === 'documents') return `documents:${[...selection.ids].sort().join(',')}`;
	return 'project';
}

const sameItem = (k: QueueItemKey, i: QueueItem) =>
	k.job_id === i.job_id &&
	k.setup_id === i.setup_id &&
	k.source_id === i.source_id &&
	k.index === i.index;

/** The interview's segments read and flagged by nothing: what a sample is drawn from. */
function unflagged(): QueueItem[] {
	return [0, 2].map((index) =>
		item({
			document_id: INTERVIEW,
			document: 'Interview with the imam',
			job_id: audioJob.id,
			job_name: audioJob.name,
			task: audioJob.task,
			source_id: audioJob.sources[0].id,
			setup_id: audioJob.setups[0].id,
			setup_label: audioJob.setups[0].label,
			piece: {
				kind: 'segment',
				index,
				start_ms: index * 1_200_000,
				end_ms: (index + 1) * 1_200_000
			},
			index,
			signals: [{ kind: 'not_reviewed' }]
		})
	);
}

const page = (n: number) => ({ piece: { kind: 'page' as const, number: n }, index: n - 1 });

function items(excluded: ResultKey[]): QueueItem[] {
	const out: QueueItem[] = [
		item({
			document_id: INTERVIEW,
			document: 'Interview with the imam',
			job_id: audioJob.id,
			job_name: audioJob.name,
			task: audioJob.task,
			source_id: audioJob.sources[0].id,
			setup_id: audioJob.setups[0].id,
			setup_label: audioJob.setups[0].label,
			piece: { kind: 'segment', index: 1, start_ms: 1_200_000, end_ms: 2_400_000 },
			index: 1,
			revision: null,
			answered_at: null,
			processing: true,
			signals: [{ kind: 'failed', error: 'The provider was busy (503) after 3 attempts.' }]
		}),
		item({
			...page(1),
			preferred: true,
			setup_id: job.setups[1].id,
			setup_label: job.setups[1].label,
			signals: [
				{ kind: 'disagrees', with: [job.setups[0].label], words: 4, fields: [] },
				{ kind: 'not_reviewed' }
			]
		}),
		item({
			...page(2),
			preferred: true,
			setup_id: job.setups[1].id,
			setup_label: job.setups[1].label,
			reviewed: true,
			open: false,
			signals: [{ kind: 'disagrees', with: [job.setups[0].label], words: 1, fields: [] }]
		}),
		item({
			...page(3),
			preferred: true,
			setup_id: job.setups[1].id,
			setup_label: job.setups[1].label,
			signals: [
				{ kind: 'truncated', reason: 'limit' },
				{ kind: 'disagrees', with: [job.setups[0].label], words: 12, fields: [] },
				{ kind: 'not_reviewed' }
			]
		})
	];
	for (const setup of analysisJob.setups) {
		if (excluded.some((k) => k.job_id === analysisJob.id && k.setup_id === setup.id)) continue;
		const other = analysisJob.setups.find((s) => s.id !== setup.id)?.label ?? '';
		out.push(
			item({
				job_id: analysisJob.id,
				job_name: analysisJob.name,
				task: analysisJob.task,
				source_id: analysisJob.sources[0].id,
				setup_id: setup.id,
				setup_label: setup.label,
				model: setup.model,
				piece: { kind: 'whole' },
				index: 0,
				signals: [
					{ kind: 'disagrees', with: [other], words: 0, fields: ['People', 'Tone'] },
					{ kind: 'stale', gone: false, unit: false },
					{ kind: 'not_reviewed' }
				]
			})
		);
	}
	return out;
}

export function queueCommand(
	cmd: string,
	a: Record<string, unknown>
): QueueView | SampleSpec | null | undefined {
	switch (cmd) {
		case 'review_queue': {
			const selection = (a.selection as QueueSelection | undefined) ?? { kind: 'project' };
			return queueView(selection, setAside.get(scopeKey(selection)) ?? []);
		}
		case 'set_result_aside': {
			const key = scopeKey((a.selection as QueueSelection | undefined) ?? { kind: 'project' });
			const result = a.result as ResultKey;
			const others = (setAside.get(key) ?? []).filter(
				(k) => k.job_id !== result.job_id || k.setup_id !== result.setup_id
			);
			setAside.set(key, a.aside ? [...others, result] : others);
			return null;
		}
		case 'set_review_sample': {
			const key = scopeKey((a.selection as QueueSelection | undefined) ?? { kind: 'project' });
			if (a.size == null) {
				samples.delete(key);
				return null;
			}
			if (typeof a.seed === 'number' && a.seed > 999_999) throw { id: 'project.seed' };
			const spec: SampleSpec = {
				seed: (a.seed as number | null) ?? 1957 + samples.size,
				size: Math.min(1000, Math.max(1, a.size as number)),
				drawn_at: Date.now()
			};
			samples.set(key, spec);
			return spec;
		}
		case 'keep_aside': {
			const key = a.item as QueueItemKey;
			kept = kept.filter((k) => !sameItem(k, key as unknown as QueueItem));
			if (typeof a.note === 'string')
				kept.push({
					...key,
					revision: (a.revision as string | null) ?? null,
					note: a.note.trim(),
					kept_at: Date.now()
				});
			return null;
		}
		case 'export_review_manifest':
			return null;
		default:
			return undefined;
	}
}

/** What a document adds to the counts: its results kept, their pieces, and the reasons its
 * items carry. */
type Tally = Omit<
	QueueCounts,
	'documents' | 'documents_read' | 'flagged' | 'unresolved' | 'reasons'
>;

function tally(documentId: string, analyses: number): Tally | null {
	if (documentId === DOCUMENT_ID) {
		return {
			results: 1 + analyses,
			pieces: 3 + analyses,
			processed: 3 + analyses,
			failed: 0,
			not_read: 0,
			reviewed: 1
		};
	}
	if (documentId === INTERVIEW) {
		return {
			results: 1,
			pieces: 3,
			processed: 2,
			failed: 1,
			not_read: 0,
			reviewed: 0
		};
	}
	return null;
}

function reasonsOf(list: QueueItem[]): ReasonCounts {
	const counts: ReasonCounts = {
		failed: 0,
		not_read: 0,
		truncated: 0,
		disagrees: 0,
		stale: 0,
		not_reviewed: 0
	};
	for (const item of list) for (const s of item.signals) counts[s.kind]++;
	return counts;
}

function queueView(selection: QueueSelection, excluded: ResultKey[]): QueueView {
	const documents = documentsIn(
		selection.kind === 'collection'
			? { collection: selection.id }
			: selection.kind === 'documents'
				? { ids: selection.ids }
				: {}
	).map((d) => d.id);
	const list = items(excluded).filter((i) => documents.includes(i.document_id));
	const spec = samples.get(scopeKey(selection)) ?? null;
	const candidates = documents.includes(INTERVIEW) ? unflagged() : [];
	const picked = spec ? candidates.slice(0, spec.size) : [];
	const all = [...list, ...picked.map((i) => ({ ...i, sample: true }))].map((i) => {
		const k = kept.find((x) => sameItem(x, i));
		return k
			? { ...i, kept: { note: k.note, kept_at: k.kept_at, current: k.revision === i.revision } }
			: i;
	});
	const analyses = analysisJob.setups.filter(
		(s) => !excluded.some((k) => k.setup_id === s.id)
	).length;
	const tallies = documents.flatMap((d) => tally(d, analyses) ?? []);
	const sum = (key: keyof Tally) => tallies.reduce((n, t) => n + t[key], 0);
	const bulletin = documents.includes(DOCUMENT_ID);
	return {
		queue: {
			counts: {
				documents: documents.length,
				documents_read: tallies.length,
				results: sum('results'),
				pieces: sum('pieces'),
				processed: sum('processed'),
				failed: sum('failed'),
				not_read: sum('not_read'),
				reviewed: sum('reviewed'),
				flagged: list.length,
				unresolved: list.filter((i) => i.open).length,
				// As Rust counts them: every item's reasons.
				reasons: reasonsOf(list)
			},
			items: all,
			sample: spec && {
				...spec,
				candidates: candidates.length,
				drawn: picked.length,
				reviewed: 0
			}
		},
		// The analysis's readings stand for the bulletin, which has no preferred one; its text is
		// read through the preferred reading.
		choices: (bulletin ? analysisJob.setups : []).map((s) => ({
			key: { job_id: analysisJob.id, setup_id: s.id },
			job_name: analysisJob.name,
			task: analysisJob.task,
			setup_label: s.label,
			model: s.model,
			documents: 1,
			included: !excluded.some((k) => k.setup_id === s.id)
		})),
		preferred: bulletin ? 1 : 0
	};
}
