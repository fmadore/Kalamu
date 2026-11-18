<script lang="ts">
	// *Read this region again*, from a region's card to the dialog and back: where the regions are
	// offered, the card's button opens the dialog on the region chosen, on the page as the review
	// shows it; a reading used is followed by the message that undoes it. In the text review a
	// reading can go into the page's text (`page`); elsewhere only into a region the researcher
	// drew.
	import RegionReadDialog from './RegionReadDialog.svelte';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { ReadingEffect } from '#lib/bindings/ReadingEffect.js';
	import type { ShownRegion } from '#lib/bindings/ShownRegion.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { targetFor } from '#lib/regionReading.js';
	import type { Regions } from '#lib/regions.svelte.js';
	import { toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		regions,
		preview = null,
		page = null
	}: {
		regions: Regions;
		/** The page as the review shows it, for the picture of the part to read. */
		preview?: PagePreview | null;
		/** The text review's page: the text shown beside the scan, and what reads it again once a
		 * reading has replaced words in it. */
		page?: { text: string; refresh: () => Promise<unknown> } | null;
	} = $props();
	const m = messages();

	let job = $state<Job | null>(null);
	let asked = $state<ShownRegion | null>(null);

	$effect(() => {
		regions.onread = (region) => void open(region);
		return () => (regions.onread = undefined);
	});

	async function open(region: ShownRegion) {
		try {
			job ??= await api.getJob(regions.jobId);
			asked = region;
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	}

	const target = $derived.by(() => {
		if (!asked || !regions.page) return null;
		const piece = page && regions.reading ? { ...regions.reading, text: page.text } : null;
		return targetFor(asked, regions.page, piece);
	});

	/** Reads again what a reading changed: the page's text when its words were replaced, and the
	 * regions, whose places in the text follow it. */
	async function reload(effect: ReadingEffect) {
		if (effect.kind === 'correction') await page?.refresh();
		await regions.reload();
	}

	async function accepted(id: string, before: ReadingEffect) {
		asked = null;
		await reload(before);
		const r = m.regionReading;
		toast(before.kind === 'region' ? r.acceptedRegion : r.acceptedText, {
			action: { label: m.common.undo, run: () => void undo(id, before) }
		});
	}

	async function undo(id: string, before: ReadingEffect) {
		try {
			await api.undoRegionReading(id, before);
			await reload(before);
			toast(m.regionReading.undone);
		} catch (e) {
			toast(describeError(e, m), { kind: 'error' });
		}
	}
</script>

{#if asked && job && target && regions.place}
	<RegionReadDialog
		{job}
		sourceId={regions.sourceId}
		page={regions.place.page}
		half={regions.place.half}
		region={asked}
		{target}
		{preview}
		onaccepted={accepted}
		onclose={() => (asked = null)}
	/>
{/if}
