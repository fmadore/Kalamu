<script lang="ts">
	// Asking a model to read one region of a page again, and what it read. First the request: a
	// picture of the part that will be read, the model that will read it (the job's own when it
	// reads pictures, else the last one used here, else the usual one), what it costs; Read.
	// Then the wait, which can be stopped. Then the reading: used where the region's target says
	// (the region's own text, or the page's text in place of the region's words, the change
	// marked), edited first, copied or discarded. A region whose words have no certain place in
	// the text can only be copied. Nothing changes before the researcher says so; the page's own
	// reading never changes. Asked through the engine, which records it and keeps its answer, so
	// the same request again costs nothing.
	import AskStatus from './AskStatus.svelte';
	import Dialog from './Dialog.svelte';
	import DiffSpans from './DiffSpans.svelte';
	import ModelPickers from './ModelPickers.svelte';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PageHalf } from '#lib/bindings/PageHalf.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { ReadingEffect } from '#lib/bindings/ReadingEffect.js';
	import type { ReadingTarget } from '#lib/bindings/ReadingTarget.js';
	import type { RegionAsk } from '#lib/bindings/RegionAsk.js';
	import type { RegionReading } from '#lib/bindings/RegionReading.js';
	import type { ShownRegion } from '#lib/bindings/ShownRegion.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { ModelChoice, ModelRequest, readSetup } from '#lib/modelAsk.svelte.js';
	import { IMAGES } from '#lib/newJobModels.js';
	import { SourcePage } from '#lib/pdf.js';
	import { pageWidthFor, pixelsOf } from '#lib/regionReading.js';
	import { app, hasKey } from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';
	import { api } from '#lib/tauri.js';

	let {
		job,
		sourceId,
		page,
		half,
		region,
		target,
		preview = null,
		onaccepted,
		onclose
	}: {
		job: Job;
		sourceId: string;
		/** The page's number in the source, from 1. */
		page: number;
		half: PageHalf | null;
		/** The region to read, as the review draws it. */
		region: ShownRegion;
		/** Where an accepted reading goes (`targetFor`). */
		target: ReadingTarget;
		/** The page as the review shows it, to draw the region's picture from; without it the
		 * dialog says in words what will be read. */
		preview?: PagePreview | null;
		/** The reading written: its id and what it replaced, for Undo. */
		onaccepted: (readingId: string, before: ReadingEffect) => void;
		onclose: () => void;
	} = $props();
	const m = messages();
	const r = m.regionReading;
	/** A region is sent as a picture of the page. */
	const PICTURES = [IMAGES];
	const LAST_KEY = 'kalamu.regionReading.setup';

	/** A provider can be used when it runs on this computer or has its key. */
	const usable = (provider: string) =>
		app.catalog?.providers.find((pr) => pr.id === provider)?.local === true || hasKey(provider);
	// The model to start from: the job's own when it reads pictures, the last one used here, else
	// the first provider that can be used and its usual model.
	// svelte-ignore state_referenced_locally
	const own = job.setups[0];
	// svelte-ignore state_referenced_locally
	const ownPreset = own && job.task === 'text_extraction' ? own.prompt.preset_id : null;
	const choice = new ModelChoice(
		'text_extraction',
		PICTURES,
		usable,
		[
			own ? { provider: own.provider, model: own.model, preset: ownPreset } : null,
			recall(LAST_KEY, readSetup, null)
		],
		''
	);
	const request = new ModelRequest<RegionReading, ReadingEffect>(r, {
		cancel: api.cancelRegionReading,
		accept: api.acceptRegionReading,
		discard: api.discardRegionReading,
		changed: 'region_reading.changed'
	});
	let choosing = $state(false);
	let editor = $state<HTMLTextAreaElement | null>(null);
	let picture = $state<HTMLCanvasElement | null>(null);
	let drawn = $state(false);

	function askOf(id: string): RegionAsk {
		return {
			id,
			job_id: job.id,
			source_id: sourceId,
			page,
			half,
			x: region.x,
			y: region.y,
			width: region.width,
			height: region.height,
			region_id: region.id,
			target,
			provider: choice.provider,
			model: choice.model,
			preset_id: choice.preset || null
		};
	}

	$effect(() => choice.loadPresets());

	// The region's picture, cut from the page the review already shows: the page drawn wide
	// enough for the region to be legible, then only the region copied.
	$effect(() => {
		const canvas = picture;
		const shown = preview;
		if (!canvas || !shown) return;
		let gone = false;
		let opened: SourcePage | null = null;
		const whole = document.createElement('canvas');
		SourcePage.open(shown.mime, shown.data)
			.then(async (p) => {
				opened = p;
				if (gone || !(await p.draw(whole, pageWidthFor(region)))) return;
				const [x, y, w, h] = pixelsOf(region, whole.width, whole.height);
				canvas.width = w;
				canvas.height = h;
				canvas.getContext('2d')?.drawImage(whole, x, y, w, h, 0, 0, w, h);
				drawn = !gone;
			})
			.catch(() => (drawn = false))
			.finally(() => void opened?.destroy());
		return () => (gone = true);
	});

	// What the request would cost, and whether it was read before, as the choice changes.
	$effect(() => {
		const ask = askOf('estimate');
		return request.estimateWith(ask.model ? () => api.regionReadingEstimate(ask) : null);
	});

	$effect(() => {
		if (request.editing && editor) editor.focus();
	});

	function read() {
		void request.ask({
			provider: choice.provider,
			modelName: choice.modelName,
			remember: () =>
				keep(LAST_KEY, {
					provider: choice.provider,
					model: choice.model,
					preset: choice.preset || null
				}),
			send: (id) => api.askRegionReading(askOf(id)),
			base: target.kind === 'text' ? target.was : null
		});
	}

	async function copy() {
		try {
			await navigator.clipboard.writeText(request.text);
			request.said = r.copied;
		} catch (e) {
			request.error = describeError(e, m);
		}
	}
</script>

<Dialog labelledby="region-reading-title" size="medium" onescape={() => request.escape(onclose)}>
	<h2 id="region-reading-title">{r.title}</h2>

	{#if preview}
		<figure class="picture">
			<!-- Named by its caption; the picture itself says nothing a screen reader can read. -->
			<canvas
				bind:this={picture}
				class:hidden={!drawn}
				aria-hidden="true"
				data-testid="region-picture"
			></canvas>
			<figcaption class="muted small">{r.picture}</figcaption>
		</figure>
	{:else}
		<p class="muted">{r.noPicture}</p>
	{/if}

	{#if request.stage === 'ask'}
		{#if choice.providers.length === 0}
			<p class="notice" role="status">{r.noModel}</p>
		{:else}
			<p class="model">
				<span>{r.readBy(choice.modelName)}</span>
				{#if !choosing}
					<button class="btn ghost" onclick={() => (choosing = true)}>{r.change}</button>
				{/if}
			</p>
			{#if choosing}
				<!-- Opened from the keyboard: the focus goes to the first picker, not the page behind. -->
				<ModelPickers {choice} defaultPrompt={r.defaultPrompt} focus />
			{/if}
			<p class="muted">
				{choice.local
					? r.sentOffline
					: choice.server
						? r.sentServer(choice.providerName, choice.server.host)
						: r.sentCloud(choice.providerName)}
			</p>
			<AskStatus {choice} estimate={request.estimate} texts={r} testid="region-cost" />
		{/if}
	{:else if request.stage === 'waiting'}
		<p class="waiting">{r.waiting(choice.modelName)}</p>
	{:else if request.stage === 'answer' && request.answer}
		{@const reading = request.answer}
		{#if request.editing}
			<label class="field">
				<span>{r.editLabel}</span>
				<textarea
					bind:this={editor}
					bind:value={request.draft}
					rows="5"
					spellcheck="false"
					class="own-direction"></textarea>
			</label>
		{:else if target.kind === 'text'}
			<figure>
				<figcaption class="muted">{r.replaced}</figcaption>
				<!-- prettier-ignore -->
				<p class="text own-direction"><DiffSpans spans={request.spans} answer={reading.answer ?? ''} removed={r.removed} added={r.added} /></p>
			</figure>
		{:else}
			<figure>
				<figcaption class="muted">{r.answer}</figcaption>
				<p class="text own-direction">{reading.answer}</p>
			</figure>
		{/if}
		{#if target.kind === 'nowhere'}
			<p class="notice" role="note">{r.nowhere}</p>
		{/if}
	{:else if request.stage === 'changed'}
		<p class="notice" role="alert">{r.changed}</p>
	{/if}

	{#if request.error}<p class="chip danger" role="alert">{request.error}</p>{/if}
	{#if request.estimateError && request.stage === 'ask'}<p class="chip danger" role="alert">
			{request.estimateError}
		</p>{/if}
	<p class="sr-only" aria-live="polite">{request.said}</p>

	<div class="actions">
		{#if request.stage === 'ask'}
			<button class="btn" onclick={onclose}>{m.common.cancel}</button>
			<button
				class="btn primary"
				disabled={request.estimateError !== null ||
					!choice.model ||
					!choice.keyed ||
					!choice.installed}
				onclick={read}
			>
				{r.read}
			</button>
		{:else if request.stage === 'waiting'}
			<button class="btn" onclick={() => request.stop()}>{r.stop}</button>
		{:else if request.stage === 'answer' && target.kind === 'nowhere'}
			<button class="btn" onclick={copy}>{r.copy}</button>
			<button class="btn primary" onclick={() => request.discard(onclose)}>{r.close}</button>
		{:else if request.stage === 'answer'}
			<button class="btn" onclick={() => request.discard(onclose)}>{r.discard}</button>
			<button class="btn" onclick={copy}>{r.copy}</button>
			{#if !request.editing}
				<button class="btn" onclick={() => (request.editing = true)}>{r.edit}</button>
			{/if}
			<button
				class="btn primary"
				disabled={request.editing && !request.draft.trim()}
				onclick={() => request.accept(onaccepted)}
			>
				{target.kind === 'region' ? r.useRegion : r.useText}
			</button>
		{:else}
			<button class="btn primary" onclick={() => request.discard(onclose)}>{r.lookAgain}</button>
		{/if}
	</div>
	{#if request.stopped && request.stage === 'ask'}<p class="muted small" role="status">
			{r.stopped}
		</p>{/if}
</Dialog>

<style>
	h2 {
		margin: 0;
		font-size: var(--type-lg);
	}
	figure {
		margin: 0;
		display: grid;
		gap: var(--space-2);
	}
	.picture canvas {
		max-width: 100%;
		max-height: 10rem;
		justify-self: start;
		border-radius: var(--radius-control);
		border: 1px solid var(--border-subtle, var(--surface-1));
		background: var(--surface-1);
	}
	.picture canvas.hidden {
		visibility: hidden;
		height: 0;
	}
	.text {
		margin: 0;
		max-height: 12rem;
		overflow: auto;
		padding: var(--space-3);
		border-radius: var(--radius-control);
		background: var(--surface-1);
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.6;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		color: var(--text-body);
	}
	.model {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-3);
	}
	textarea {
		width: 100%;
		font-family: var(--font-text);
		font-size: var(--type-reading);
		line-height: 1.6;
	}
	.waiting {
		color: var(--text-secondary);
	}
	.small {
		margin: 0;
		font-size: var(--type-xs);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	p {
		margin: 0;
	}
</style>
