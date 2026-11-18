import { describe, expect, it, vi } from 'vitest';

import type { DrawnRegion } from './bindings/DrawnRegion.js';
import type { PageHalf } from './bindings/PageHalf.js';
import type { RegionDraft } from './bindings/RegionDraft.js';

// The app's answers, as the browser preview gives them (`dev/mockDrawn.ts`), and the messages
// shown, with their Undo.
const shown = vi.hoisted(() => ({ toasts: [] as { text: string; undo?: () => void }[] }));
vi.mock('./tauri.js', async () => {
	const { regionCommand } = await import('./dev/mockDrawn.js');
	const { mockRegions } = await import('./dev/mockRegions.js');
	return {
		api: {
			pageRegions: async (jobId: string, _s: string, setupId: string, index: number) =>
				mockRegions(jobId, setupId, index),
			drawnRegions: async (jobId: string, sourceId: string, page: number, half: PageHalf | null) =>
				regionCommand('drawn_regions', { jobId, sourceId, page, half }),
			saveRegion: vi.fn(async (draft: RegionDraft) => regionCommand('save_region', { draft })),
			removeRegion: async (id: string) => regionCommand('remove_region', { id }),
			restoreRegion: async (region: DrawnRegion) => regionCommand('restore_region', { region })
		}
	};
});
vi.mock('./state.svelte.js', () => ({
	toast: (text: string, o: { action?: { run: () => void } } = {}) =>
		shown.toasts.push({ text, undo: o.action?.run })
}));

const { Regions, placeOf } = await import('./regions.svelte.js');
const { api } = await import('./tauri.js');
const { job, setupA, source, texts, pieces } = await import('./dev/mockJobs.js');
const { contentsJob, contentsSource } = await import('./dev/mockContents.js');

/** The text review's regions of the sample newspaper's page `index`, read. */
async function textPage(index: number) {
	const regions = new Regions(job.id, source.id);
	const piece = pieces.find((p) => p.setup_id === setupA.id && p.index === index)?.piece;
	regions.load(setupA.id, index, texts[index] ?? '', true, piece);
	await vi.waitFor(() => expect(regions.page).not.toBeNull());
	return regions;
}

describe('the regions of a page', () => {
	it('say which page a piece is', () => {
		expect(placeOf({ kind: 'page', number: 3, half: 'left' })).toEqual({ page: 3, half: 'left' });
		expect(placeOf({ kind: 'whole' })).toBeNull();
	});

	it('keep a box drawn with the pointer at once, among the researcher’s own, chosen', async () => {
		const regions = await textPage(2);
		expect(regions.any).toBe(false);
		expect(regions.drawable).toBe(true);
		regions.edit.startDrawing();
		expect(regions.edit.mode).toBe('drawing');
		await regions.edit.drawn({ x: 0.1, y: 0.1, width: 0.3, height: 0.2 });
		expect(regions.edit.mode).toBe('idle');
		expect(regions.page?.drawn).toHaveLength(1);
		expect(regions.chosen).toMatchObject({ level: 'drawn', index: 0 });
		expect(regions.anyAtAll).toBe(true);
		expect(regions.focusing).toBe(true);

		// What is written there is kept with it; the same text again is not sent.
		const region = regions.region;
		if (!region) throw new Error('no region chosen');
		await regions.edit.saveText(region, 'Note en marge');
		expect(regions.region?.text).toBe('Note en marge');

		// Removed, then brought back from the message that follows.
		shown.toasts = [];
		await regions.edit.remove(regions.region ?? region);
		expect(regions.page?.drawn).toHaveLength(0);
		expect(regions.chosen).toBeNull();
		expect(shown.toasts[0]?.text).toBe('Region removed.');
		shown.toasts[0]?.undo?.();
		await vi.waitFor(() => expect(regions.page?.drawn).toHaveLength(1));
		expect(regions.region?.text).toBe('Note en marge');
		await regions.edit.remove(regions.region ?? region);
	});

	it('give an engine’s region the researcher’s box beside its own, and put the engine’s back', async () => {
		const regions = await textPage(0);
		const before = regions.page?.blocks[2];
		if (!before) throw new Error('no block');
		regions.choose('blocks', 2);
		regions.edit.adjustChosen();
		expect(regions.edit.target).toMatchObject({ kind: 'engine', id: before.id });
		expect(regions.edit.nudge('ArrowRight', false, 1, 0)).toBe(true);
		expect(regions.edit.said).toBe('Moved right');
		regions.edit.nudge('ArrowDown', true, 1, 0);
		expect(regions.edit.said).toBe('Taller');
		await regions.edit.done();
		expect(regions.edit.mode).toBe('idle');
		const after = regions.page?.blocks[2];
		expect(after?.origin).toBe('manual');
		expect(before.adjusted_by).toBeNull();
		expect(after?.adjusted_by).toEqual(expect.any(String));
		expect(after?.id).toBe(before.id);
		expect(after?.text).toBe(before.text);
		expect(after?.x).toBeCloseTo(before.x + 0.01);
		expect(after?.height).toBeCloseTo(before.height + 0.01);
		// Not among the researcher's own regions: it stays the engine's region.
		expect(regions.page?.drawn).toHaveLength(0);
		expect(regions.chosen).toMatchObject({ level: 'blocks', index: 2 });

		// Put back by removing the box the page names, with nothing saved first.
		shown.toasts = [];
		const saves = vi.mocked(api.saveRegion).mock.calls.length;
		await regions.edit.putBack(after ?? before);
		expect(vi.mocked(api.saveRegion).mock.calls.length).toBe(saves);
		expect(regions.page?.blocks[2]).toMatchObject({
			origin: 'provider',
			x: before.x,
			adjusted_by: null
		});
		expect(shown.toasts[0]?.text).toBe('The model’s box is back.');
		shown.toasts[0]?.undo?.();
		await vi.waitFor(() => expect(regions.page?.blocks[2]?.origin).toBe('manual'));
		await regions.edit.putBack(regions.page?.blocks[2] ?? before);
	});

	it('put a box adjusted back as it was when cancelled, and let one left as it was go', async () => {
		const regions = await textPage(0);
		regions.choose('blocks', 0);
		regions.edit.adjustChosen();
		regions.edit.nudge('ArrowLeft', false, 2, 90);
		regions.edit.cancel();
		expect(regions.edit.mode).toBe('idle');
		expect(regions.page?.blocks[0].origin).toBe('provider');
		regions.edit.adjustChosen();
		await regions.edit.done();
		expect(regions.page?.blocks[0].origin).toBe('provider');
	});

	it('place a box from the keyboard in the middle of the view, kept only with Done', async () => {
		const regions = await textPage(1);
		regions.edit.seen = () => ({ view: { x: 0, y: 0, width: 1, height: 0.5 }, rotation: 0 });
		regions.edit.startDrawing();
		regions.edit.startDrawing();
		expect(regions.edit.mode).toBe('adjusting');
		expect(regions.edit.box?.y).toBeCloseTo(0.225);
		regions.edit.cancel();
		expect(regions.page?.drawn).toHaveLength(0);
		regions.edit.startDrawing();
		regions.edit.startDrawing();
		await regions.edit.done();
		expect(regions.page?.drawn).toHaveLength(1);
		const kept = regions.region;
		if (kept) await regions.edit.remove(kept);
	});

	it('read only the researcher’s own on a page of a table of contents', async () => {
		const regions = new Regions(contentsJob.id, contentsSource.id);
		regions.loadDrawn({ page: 2, half: null });
		await vi.waitFor(() => expect(regions.page).not.toBeNull());
		expect(regions.engine).toBe(false);
		expect(regions.page?.blocks).toHaveLength(0);
		await regions.edit.drawn({ x: 0.5, y: 0.5, width: 0.2, height: 0.2 });
		expect(regions.page?.drawn).toHaveLength(1);
		regions.loadDrawn({ page: 1, half: null });
		await vi.waitFor(() => expect(regions.page?.drawn).toHaveLength(0));
		expect(regions.chosen).toBeNull();
		regions.loadDrawn(null);
		expect(regions.place).toBeNull();
	});
});
