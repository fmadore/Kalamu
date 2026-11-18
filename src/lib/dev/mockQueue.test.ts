import { beforeAll, describe, expect, it } from 'vitest';
import type { QueueView } from '../bindings/QueueView.js';
import { analysisJob, audioJob, job, source } from './mockJobs.js';
import { DOCUMENT_ID, seedProjects } from './mockProjects.js';
import { queueCommand } from './mockQueue.js';

const INTERVIEW = '00000000-0000-0000-0000-0000000000d2';
const LETTERS = '00000000-0000-0000-0000-0000000000c1';

beforeAll(() =>
	seedProjects({
		job,
		audioJob,
		analysisJob,
		extras: [1, 2].map((n) => ({
			...source,
			id: `00000000-0000-0000-0000-0000000000f${n}`,
			file_name: `letter-${n}.jpg`
		}))
	})
);

describe('the review queue mock honours its selection', () => {
	const queue = (selection: unknown) => queueCommand('review_queue', { selection }) as QueueView;

	it('lists and counts only the documents chosen', () => {
		const whole = queue({ kind: 'project' });
		expect(whole.queue.counts.documents).toBe(4);
		expect(new Set(whole.queue.items.map((i) => i.document_id))).toEqual(
			new Set([DOCUMENT_ID, INTERVIEW])
		);

		const chosen = queue({ kind: 'documents', ids: [INTERVIEW] });
		expect(chosen.queue.items.every((i) => i.document_id === INTERVIEW)).toBe(true);
		expect(chosen.queue.counts).toMatchObject({ documents: 1, documents_read: 1, failed: 1 });
		expect(chosen.choices).toEqual([]);
		expect(chosen.preferred).toBe(0);
	});

	it('counts a collection whose documents were never read', () => {
		const letters = queue({ kind: 'collection', id: LETTERS });
		expect(letters.queue.items).toEqual([]);
		expect(letters.queue.counts).toMatchObject({ documents: 2, documents_read: 0, results: 0 });
	});

	it('draws a sample from what nothing flags, kept per scope until removed', () => {
		const project = { kind: 'project' };
		const spec = queueCommand('set_review_sample', { selection: project, size: 1, seed: 7 });
		expect(spec).toMatchObject({ seed: 7, size: 1 });
		const view = queue(project);
		expect(view.queue.sample).toMatchObject({ seed: 7, drawn: 1, candidates: 2, reviewed: 0 });
		const drawn = view.queue.items.filter((i) => i.sample);
		expect(drawn).toHaveLength(1);
		expect(drawn.every((i) => i.signals.every((s) => s.kind === 'not_reviewed'))).toBe(true);
		// Another scope has no sample of its own.
		expect(queue({ kind: 'collection', id: LETTERS }).queue.sample).toBeNull();
		queueCommand('set_review_sample', { selection: project, size: null });
		expect(queue(project).queue.sample).toBeNull();
	});

	it('keeps an item aside for the answer it holds, and puts it back', () => {
		const project = { kind: 'project' };
		const first = queue(project).queue.items.find((i) => i.document_id === DOCUMENT_ID);
		if (!first) throw new Error('an item');
		const key = {
			job_id: first.job_id,
			setup_id: first.setup_id,
			source_id: first.source_id,
			index: first.index
		};
		queueCommand('keep_aside', { item: key, revision: first.revision, note: ' Faded ink ' });
		const kept = queue(project).queue.items.find(
			(i) => i.job_id === first.job_id && i.setup_id === first.setup_id && i.index === first.index
		);
		expect(kept?.kept).toMatchObject({ note: 'Faded ink', current: true });
		queueCommand('keep_aside', { item: key, revision: 'older', note: 'Faded ink' });
		expect(
			queue(project).queue.items.find((i) => i.index === first.index && i.kept)?.kept?.current
		).toBe(false);
		queueCommand('keep_aside', { item: key, revision: null, note: null });
		expect(queue(project).queue.items.some((i) => i.kept)).toBe(false);
	});

	it('keeps the results set aside for their scope until brought back', () => {
		const project = { kind: 'project' };
		const result = { job_id: analysisJob.id, setup_id: analysisJob.setups[0].id };
		const reads = (view: QueueView) =>
			view.queue.items.filter((i) => i.setup_id === result.setup_id).length;
		expect(reads(queue(project))).toBe(1);
		queueCommand('set_result_aside', { selection: project, result, aside: true });
		const aside = queue(project);
		expect(reads(aside)).toBe(0);
		expect(aside.choices.find((c) => c.key.setup_id === result.setup_id)?.included).toBe(false);
		// Another scope reads it still.
		const chosen = { kind: 'documents', ids: [DOCUMENT_ID] };
		expect(reads(queue(chosen))).toBe(1);
		queueCommand('set_result_aside', { selection: project, result, aside: false });
		expect(reads(queue(project))).toBe(1);
	});
});
