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
	const queue = (selection: unknown) =>
		queueCommand('review_queue', { selection, excluded: [] }) as QueueView;

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
});
