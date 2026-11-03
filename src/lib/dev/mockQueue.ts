// The sample project's list of what to check, for the browser preview (`?mock&view=queue`): the
// bulletin read by two models that disagree, one page cut off, the interview with a failed
// segment, and the analysis read before its text was corrected. Development builds only.

import type { QueueItem } from '../bindings/QueueItem.js';
import type { QueueView } from '../bindings/QueueView.js';
import type { ResultKey } from '../bindings/ResultKey.js';
import { analysisJob, audioJob, job } from './mockJobs.js';
import { DOCUMENT_ID } from './mockProjects.js';

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
		...over
	};
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
				{ kind: 'truncated' },
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

export function queueCommand(cmd: string, a: Record<string, unknown>): QueueView | undefined {
	switch (cmd) {
		case 'review_queue':
			return queueView((a.excluded as ResultKey[] | undefined) ?? []);
		default:
			return undefined;
	}
}

function queueView(excluded: ResultKey[]): QueueView {
	const list = items(excluded);
	const analyses = analysisJob.setups.filter(
		(s) => !excluded.some((k) => k.setup_id === s.id)
	).length;
	return {
		queue: {
			counts: {
				documents: 4,
				documents_read: 2,
				results: 2 + analyses,
				pieces: 6 + analyses,
				processed: 5 + analyses,
				failed: 1,
				not_read: 0,
				reviewed: 1,
				flagged: list.length,
				unresolved: list.filter((i) => i.open).length,
				truncated: 1,
				disagrees: 3 + analyses,
				stale: analyses,
				not_reviewed: 2 + analyses
			},
			items: list
		},
		choices: analysisJob.setups.map((s) => ({
			key: { job_id: analysisJob.id, setup_id: s.id },
			job_name: analysisJob.name,
			task: analysisJob.task,
			setup_label: s.label,
			model: s.model,
			documents: 1,
			included: !excluded.some((k) => k.setup_id === s.id)
		})),
		preferred: 2
	};
}
