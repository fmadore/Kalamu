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
	import Dialog from './Dialog.svelte';
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';
	import type { Job } from '#lib/bindings/Job.js';
	import type { PageHalf } from '#lib/bindings/PageHalf.js';
	import type { PagePreview } from '#lib/bindings/PagePreview.js';
	import type { PassageEstimate } from '#lib/bindings/PassageEstimate.js';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { ReadingEffect } from '#lib/bindings/ReadingEffect.js';
	import type { ReadingTarget } from '#lib/bindings/ReadingTarget.js';
	import type { RegionAsk } from '#lib/bindings/RegionAsk.js';
	import type { RegionReading } from '#lib/bindings/RegionReading.js';
	import type { ShownRegion } from '#lib/bindings/ShownRegion.js';
	import { providerName as providerNameOf } from '#lib/catalogText.js';
	import { askConsent } from '#lib/dialogs.js';
	import { describeError, isAppError } from '#lib/errors.js';
	import { formatUsd, messages } from '#lib/i18n/index.js';
	import { rulesFor } from '#lib/newJob.js';
	import { IMAGES, findModel, modelsFor, providerDefault } from '#lib/newJobModels.js';
	import { SourcePage } from '#lib/pdf.js';
	import { pageWidthFor, pixelsOf } from '#lib/regionReading.js';
	import { app, hasKey, isInstalled } from '#lib/state.svelte.js';
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

	const catalog = $derived(app.catalog);
	const rule = $derived(rulesFor(app.rules, 'text_extraction'));
	const providers = $derived(
		(catalog?.providers ?? []).filter(
			(pr) => catalog && modelsFor(catalog, rule, pr.id, PICTURES).length > 0
		)
	);
	let presets = $state<Preset[]>([]);

	type Chosen = { provider: string; model: string; preset: string | null };
	function readChosen(v: unknown): Chosen | null {
		const s = v as Partial<Chosen> | null;
		return typeof s?.provider === 'string' && typeof s.model === 'string'
			? { provider: s.provider, model: s.model, preset: s.preset ?? null }
			: null;
	}
	const usable = (provider: string) =>
		catalog?.providers.find((pr) => pr.id === provider)?.local === true || hasKey(provider);

	/** The model to start from: the job's own when it reads pictures, the last one used here,
	 * else the first provider that can be used and its usual model. */
	function startingChoice(): Chosen {
		const offered = (s: Chosen | null): s is Chosen =>
			s !== null &&
			catalog !== null &&
			modelsFor(catalog, rule, s.provider, PICTURES).some((x) => x.id === s.model);
		const own = job.setups[0];
		const fromJob: Chosen | null = own
			? {
					provider: own.provider,
					model: own.model,
					preset: job.task === 'text_extraction' ? own.prompt.preset_id : null
				}
			: null;
		if (offered(fromJob)) return fromJob;
		const last = recall(LAST_KEY, readChosen, null);
		if (offered(last)) return last;
		const first = providers.find((pr) => usable(pr.id)) ?? providers[0];
		const id = first?.id ?? '';
		return {
			provider: id,
			model: catalog ? providerDefault(catalog, rule, id, PICTURES) : '',
			preset: null
		};
	}
	const start = startingChoice();
	let provider = $state(start.provider);
	let model = $state(start.model);
	let preset = $state<string>(start.preset ?? '');
	let choosing = $state(false);
	const models = $derived(catalog ? modelsFor(catalog, rule, provider, PICTURES) : []);

	const providerName = $derived.by(() => {
		const found = catalog?.providers.find((pr) => pr.id === provider);
		return found ? providerNameOf(found) : provider;
	});
	const modelName = $derived(
		(catalog ? findModel(catalog, provider, model)?.name : undefined) ?? model
	);
	const local = $derived(catalog?.providers.find((pr) => pr.id === provider)?.local === true);
	const keyed = $derived(usable(provider));
	const installed = $derived(isInstalled(provider, model));

	type Stage = 'ask' | 'waiting' | 'answer' | 'changed';
	let stage = $state<Stage>('ask');
	let estimate = $state<PassageEstimate | null>(null);
	/** Why the request would be refused, from the engine: a box off the page, say. */
	let estimateError = $state<string | null>(null);
	let error = $state<string | null>(null);
	let stopped = $state(false);
	let reading = $state<RegionReading | null>(null);
	let spans = $state<DiffSpan[]>([]);
	let editing = $state(false);
	let draft = $state('');
	/** Read out as it changes. */
	let said = $state('');
	let asking = $state<string | null>(null);
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
			provider,
			model,
			preset_id: preset || null
		};
	}

	// The pickers opened from the keyboard: focus goes to the first, not to the page behind.
	let firstPicker = $state<HTMLSelectElement | null>(null);
	$effect(() => {
		if (choosing && firstPicker) firstPicker.focus();
	});

	$effect(() => {
		void api
			.listPresets('text_extraction')
			.then((list) => (presets = list))
			.catch(() => (presets = []));
	});

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
		estimateError = null;
		if (stage !== 'ask' || !ask.model) {
			estimate = null;
			return;
		}
		let gone = false;
		api
			.regionReadingEstimate(ask)
			.then((e) => {
				if (!gone) estimate = e;
			})
			.catch((e) => {
				if (gone) return;
				estimate = null;
				estimateError = describeError(e, m);
			});
		return () => (gone = true);
	});

	$effect(() => {
		if (editing && editor) editor.focus();
	});

	async function read() {
		error = null;
		stopped = false;
		if (!(await askConsent([provider], app.catalog))) return;
		keep(LAST_KEY, { provider, model, preset: preset || null });
		const id = crypto.randomUUID();
		asking = id;
		stage = 'waiting';
		said = r.waiting(modelName);
		try {
			const answer = await api.askRegionReading(askOf(id));
			if (answer.status === 'answered' && answer.answer !== null) {
				reading = answer;
				draft = answer.answer;
				spans =
					target.kind === 'text'
						? await api.diffTexts(target.was, answer.answer).catch(() => [])
						: [];
				stage = 'answer';
				said = r.ready;
			} else if (answer.status === 'cancelled') {
				stage = 'ask';
				stopped = true;
				said = r.stopped;
			} else {
				stage = 'ask';
				error = r.failed(answer.error ?? '');
				said = '';
			}
		} catch (e) {
			stage = 'ask';
			error = describeError(e, m);
			said = '';
		} finally {
			asking = null;
		}
	}

	function stop() {
		if (asking) void api.cancelRegionReading(asking);
	}

	const shownText = $derived(editing ? draft : (reading?.answer ?? ''));

	async function use() {
		if (!reading) return;
		error = null;
		try {
			const before = await api.acceptRegionReading(reading.ask.id, shownText);
			onaccepted(reading.ask.id, before);
		} catch (e) {
			if (isAppError(e) && e.id === 'region_reading.changed') {
				stage = 'changed';
				said = r.changed;
			} else {
				error = describeError(e, m);
			}
		}
	}

	async function copy() {
		try {
			await navigator.clipboard.writeText(shownText);
			said = r.copied;
		} catch (e) {
			error = describeError(e, m);
		}
	}

	/** Leaves the reading unused, and closes. */
	function discard() {
		if (reading) void api.discardRegionReading(reading.ask.id).catch(() => undefined);
		onclose();
	}

	function escape() {
		if (stage === 'waiting') stop();
		else if (stage === 'answer' || stage === 'changed') discard();
		else onclose();
	}
</script>

<Dialog labelledby="region-reading-title" size="medium" onescape={escape}>
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

	{#if stage === 'ask'}
		{#if providers.length === 0}
			<p class="notice" role="status">{r.noModel}</p>
		{:else}
			<p class="model">
				<span>{r.readBy(modelName)}</span>
				{#if !choosing}
					<button class="btn ghost" onclick={() => (choosing = true)}>{r.change}</button>
				{/if}
			</p>
			{#if choosing}
				<div class="pickers">
					<label class="field">
						<span>{m.newJob.provider}</span>
						<select
							bind:this={firstPicker}
							bind:value={provider}
							onchange={() =>
								catalog && (model = providerDefault(catalog, rule, provider, PICTURES))}
						>
							{#each providers as pr (pr.id)}
								<option value={pr.id}>{providerNameOf(pr)}</option>
							{/each}
						</select>
					</label>
					<label class="field">
						<span>{m.newJob.model}</span>
						<select bind:value={model}>
							{#each models as md (md.id)}
								<option value={md.id}>{md.name}</option>
							{/each}
						</select>
					</label>
					<label class="field wide">
						<span>{m.newJob.prompt}</span>
						<select bind:value={preset}>
							<option value="">{r.defaultPrompt}</option>
							{#each presets as pr (pr.id)}
								<option value={pr.id}>{pr.name}</option>
							{/each}
						</select>
					</label>
				</div>
			{/if}
			<p class="muted">{local ? r.sentOffline : r.sentCloud(providerName)}</p>
			{#if !keyed}
				<p class="notice" role="status">{r.noKey(providerName)}</p>
			{:else if !installed}
				<p class="notice" role="status">{r.notInstalled(modelName)}</p>
			{/if}
			{#if estimate}
				<p class="num" data-testid="region-cost">
					{#if estimate.cached}{r.costStored}{:else if estimate.offline}{r.costOffline}{:else if estimate.usd !== null}{r.costAbout(
							formatUsd(estimate.usd)
						)}{:else}{r.costUnknown}{/if}
				</p>
			{/if}
		{/if}
	{:else if stage === 'waiting'}
		<p class="waiting">{r.waiting(modelName)}</p>
	{:else if stage === 'answer' && reading}
		{#if editing}
			<label class="field">
				<span>{r.editLabel}</span>
				<textarea bind:this={editor} bind:value={draft} rows="5" spellcheck="false" dir="auto"
				></textarea>
			</label>
		{:else if target.kind === 'text'}
			<figure>
				<figcaption class="muted">{r.replaced}</figcaption>
				<!-- prettier-ignore -->
				<p class="text" dir="auto">{#each spans as s, i (i)}{#if s.kind === 'removed'}<del><span class="sr-only">{r.removed}: </span>{s.text}</del>{:else if s.kind === 'added'}<ins><span class="sr-only">{r.added}: </span>{s.text}</ins>{:else}{s.text}{/if}{:else}{reading.answer}{/each}</p>
			</figure>
		{:else}
			<figure>
				<figcaption class="muted">{r.answer}</figcaption>
				<p class="text" dir="auto">{reading.answer}</p>
			</figure>
		{/if}
		{#if target.kind === 'nowhere'}
			<p class="notice" role="note">{r.nowhere}</p>
		{/if}
	{:else if stage === 'changed'}
		<p class="notice" role="alert">{r.changed}</p>
	{/if}

	{#if error}<p class="chip danger" role="alert">{error}</p>{/if}
	{#if estimateError && stage === 'ask'}<p class="chip danger" role="alert">{estimateError}</p>{/if}
	<p class="sr-only" aria-live="polite">{said}</p>

	<div class="actions">
		{#if stage === 'ask'}
			<button class="btn" onclick={onclose}>{m.common.cancel}</button>
			<button
				class="btn primary"
				disabled={estimateError !== null || !model || !keyed || !installed}
				onclick={read}>{r.read}</button
			>
		{:else if stage === 'waiting'}
			<button class="btn" onclick={stop}>{r.stop}</button>
		{:else if stage === 'answer' && target.kind === 'nowhere'}
			<button class="btn" onclick={copy}>{r.copy}</button>
			<button class="btn primary" onclick={discard}>{r.close}</button>
		{:else if stage === 'answer'}
			<button class="btn" onclick={discard}>{r.discard}</button>
			<button class="btn" onclick={copy}>{r.copy}</button>
			{#if !editing}
				<button class="btn" onclick={() => (editing = true)}>{r.edit}</button>
			{/if}
			<button class="btn primary" disabled={editing && !draft.trim()} onclick={use}
				>{target.kind === 'region' ? r.useRegion : r.useText}</button
			>
		{:else}
			<button class="btn primary" onclick={discard}>{r.lookAgain}</button>
		{/if}
	</div>
	{#if stopped && stage === 'ask'}<p class="muted small" role="status">{r.stopped}</p>{/if}
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
	del {
		color: var(--danger-text);
		text-decoration-thickness: 2px;
	}
	ins {
		text-decoration: none;
		background: var(--hand-wash);
		color: var(--text-bright);
		border-bottom: 2px solid var(--hand);
	}
	.model {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: var(--space-3);
	}
	.pickers {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-3);
	}
	.pickers .wide {
		grid-column: 1 / -1;
	}
	.pickers select {
		width: 100%;
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
