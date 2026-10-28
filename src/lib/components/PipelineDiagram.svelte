<script lang="ts">
	// How a task's work is split and put back together, drawn with the user's own files once
	// they are added: a PDF's pages, a recording's pieces, the prompt's fields. It plays once when
	// a task is chosen and on request; at rest, and under reduced motion, it shows its last frame.
	// Each step is a list item with its words, so the drawing itself is only decoration.
	import Icon from './Icon.svelte';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import { messages } from '#lib/i18n/index.js';
	import { duration, type Mode } from '#lib/newJob.js';

	let {
		mode,
		files,
		fields,
		models,
		play
	}: {
		mode: Mode;
		files: InspectedFile[];
		/** The chosen prompt's fields, for an analysis. */
		fields: string[];
		/** Names of the chosen models. */
		models: string[];
		/** Counts the times a task was chosen; each one plays the diagram again. */
		play: number;
	} = $props();

	const m = messages();

	let replays = $state(0);
	const run = $derived(play + replays);

	type Art =
		| 'stack'
		| 'pages'
		| 'read'
		| 'joined'
		| 'whole'
		| 'fields'
		| 'table'
		| 'wave'
		| 'cut'
		| 'transcribe'
		| 'transcript'
		| 'link'
		| 'send'
		| 'watch';
	/** `detail` names the user's file, in mono; `note` adds what the step does with it. */
	type Stage = { art: Art; caption: string; detail?: string[]; note?: string };

	const sources = $derived(
		files.flatMap((f) => (f.source ? [{ ...f.source, pieces: f.pieces }] : []))
	);
	const first = $derived(sources[0]);
	const more = $derived(sources.length > 1 ? m.pipeline.moreFiles(sources.length - 1) : undefined);
	const model = $derived(models.length === 1 ? models[0] : m.pipeline.theModel);
	const fieldNames = $derived(fields.length > 0 ? fields : m.pipeline.exampleFields);

	/** A file's name, then its size as the file list shows it. */
	const fileDetail = $derived(
		first
			? [
					first.file_name,
					first.page_count ? m.newJob.pages(first.page_count) : '',
					first.duration_ms ? duration(Number(first.duration_ms), m) : ''
				].filter(Boolean)
			: undefined
	);

	/** Pages of the first file as pieces: a spread split in two counts twice; an image is one. */
	const pages = $derived(first ? (first.page_count ?? 1) * (first.split_spreads ? 2 : 1) : 3);
	const pieces = $derived(first?.pieces ?? 3);
	const shownPages = $derived(Math.min(pages, 3));
	const shownPieces = $derived(Math.min(pieces, 5));

	const stages = $derived.by((): Stage[] => {
		switch (mode) {
			case 'text_extraction':
				return [
					{ art: 'stack', caption: m.pipeline.extractSource, detail: fileDetail },
					{
						art: 'pages',
						caption: m.pipeline.extractSplit,
						note: first?.split_spreads ? m.pipeline.extractSpreads : undefined
					},
					{ art: 'read', caption: m.pipeline.extractRead(model) },
					{ art: 'joined', caption: m.pipeline.extractJoin, note: more }
				];
			case 'analysis':
				return [
					{ art: 'whole', caption: m.pipeline.analysisWhole, detail: fileDetail },
					{ art: 'fields', caption: m.pipeline.analysisFill(model) },
					{ art: 'table', caption: m.pipeline.analysisTable }
				];
			case 'audio_transcription':
				return [
					{ art: 'wave', caption: m.pipeline.audioSource, detail: fileDetail },
					{
						art: 'cut',
						caption: first?.pieces ? m.pipeline.audioCut(pieces) : m.pipeline.audioCutExample
					},
					{ art: 'transcribe', caption: m.pipeline.audioRead(model) },
					{
						art: 'transcript',
						caption: m.pipeline.audioJoin,
						note: pieces > 1 ? m.pipeline.audioSpeakers : more
					}
				];
			case 'youtube':
				return [
					{
						art: 'link',
						caption: m.pipeline.youtubeSource,
						detail: first?.title ? [first.title] : undefined
					},
					{ art: 'send', caption: m.pipeline.youtubeSend },
					{ art: 'watch', caption: m.pipeline.youtubeRead(model) },
					{ art: 'transcript', caption: m.pipeline.youtubeJoin, note: more }
				];
		}
	});

	// Drawing helpers, on a 140 × 72 grid.
	const W = 140;
	/** Left edges of `n` boxes `w` wide, `gap` apart, centred. */
	const row = (n: number, w: number, gap: number) =>
		Array.from({ length: n }, (_, i) => (W - (n * w + (n - 1) * gap)) / 2 + i * (w + gap));

	/** A waveform of 32 bars, quiet where the recording is cut. */
	const BARS = 32;
	const cuts = $derived(
		Array.from({ length: shownPieces - 1 }, (_, i) => Math.round(((i + 1) * BARS) / shownPieces))
	);
	const bars = $derived(
		Array.from({ length: BARS }, (_, k) =>
			cuts.some((c) => k === c || k === c - 1)
				? 2
				: 4 + Math.round(24 * Math.abs(Math.sin(k * 1.7) * Math.cos(k * 0.45)))
		)
	);
	/** Which piece each bar falls in. */
	const pieceOf = (k: number) => cuts.filter((c) => k >= c).length;
	const BAR_STEP = 3.5;
	const CUT_GAP = 6;
	const waveLeft = $derived((W - (BARS * BAR_STEP + (shownPieces - 1) * CUT_GAP)) / 2);

	const video = $derived(
		first?.path.replace(/^https?:\/\/(www\.)?/, '') ?? 'youtube.com/watch?v=…'
	);
	/** The table's rows: the user's documents, or stand-ins until there are some. */
	const rows = $derived(
		sources.length > 0
			? sources.slice(0, sources.length > 3 ? 2 : 3).map((s) => s.file_name)
			: [1, 2, 3].map((n) => m.pipeline.document(n))
	);
</script>

<figure class="diagram" class:playing={run > 0}>
	<figcaption class="head">
		<span class="title">{m.pipeline.title}</span>
		<button class="btn sm ghost replay" onclick={() => replays++}
			><Icon name="replay" size={14} />{m.pipeline.replay}</button
		>
	</figcaption>
	{#key run}
		<ol class="stages" class:analysis={mode === 'analysis'} style="--n: {stages.length}">
			{#each stages as stage, i (stage.art)}
				<li class="stage" style="--i: {i}">
					{#if i > 0}<span class="connector"><Icon name="chevron" size={14} /></span>{/if}
					<div class="art" aria-hidden="true">
						{@render art(stage.art)}
					</div>
					<p class="caption">{stage.caption}</p>
					{#each stage.detail ?? [] as line (line)}<p class="detail mono" title={line}>
							{line}
						</p>{/each}
					{#if stage.note}<p class="note">{stage.note}</p>{/if}
				</li>
			{/each}
		</ol>
	{/key}
</figure>

{#snippet sheet(x: number, y: number, w: number, h: number, lines: number, ink = 'stub')}
	<rect class="paper" {x} {y} width={w} height={h} rx="2" />
	{#each { length: lines } as _, k (k)}
		<rect
			class={ink}
			x={x + 5}
			y={y + 7 + k * 6}
			width={(w - 10) * (k % 3 === 2 ? 0.6 : 1)}
			height="2"
			rx="1"
		/>
	{/each}
{/snippet}

{#snippet transcript(split: boolean)}
	<svg width={W} height="72" viewBox="0 0 {W} 72">
		{#each [0, 1, 2, 3] as k (k)}
			{@const y = 8 + k * 15 + (split && k > 1 ? 4 : 0)}
			{@const second = split && k > 1 ? k % 2 === 0 : k % 2 === 1}
			<g class="grow" style="--j: {k}">
				<rect class="time" x="14" {y} width="16" height="3" rx="1" />
				<rect
					class={second ? 'speaker-2' : 'speaker-1'}
					x="36"
					y={y - 2}
					width="12"
					height="7"
					rx="2"
				/>
				<rect class="stub" x="54" {y} width={[66, 50, 58, 40][k]} height="3" rx="1.5" />
			</g>
		{/each}
		{#if split}
			<line class="rule" x1="14" x2="126" y1="40" y2="40" />
		{/if}
	</svg>
{/snippet}

{#snippet art(kind: Art)}
	{#if kind === 'stack' || kind === 'whole'}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			{@render sheet(56, 4, 40, 54, 0)}
			{@render sheet(50, 8, 40, 54, 0)}
			<!-- A scan's print is fainter than the text a model writes. -->
			{@render sheet(44, 12, 40, 54, 7, kind === 'stack' ? 'print' : 'stub')}
			{#if kind === 'whole'}
				<!-- One piece: the whole document in one outline; what the fields ask for, found. -->
				<rect class="outline" x="38" y="1" width="64" height="70" rx="6" />
				{#each [[49, 24, 14], [62, 42, 12], [49, 54, 10]] as [x, y, w], j (j)}
					<g class="marker" style="--j: {j}">
						<rect class="mark" {x} y={y - 0.5} width={w} height="5" rx="1" />
						<rect class="on-mark" x={x + 2} y={y + 1} width={w - 4} height="2" rx="1" />
					</g>
				{/each}
			{/if}
		</svg>
	{:else if kind === 'pages' || kind === 'read'}
		{@const xs = row(shownPages + (pages > shownPages ? 1 : 0), 22, 8)}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			{#each xs.slice(0, shownPages) as x, j (j)}
				<g class={kind === 'pages' ? 'fan' : ''} style="--j: {j}; --dx: {W / 2 - 11 - x}px">
					<rect class="paper" {x} y="10" width="22" height="30" rx="2" />
					{#if kind === 'pages'}
						{#each [0, 1, 2, 3] as k (k)}
							<rect
								class="print"
								x={x + 4}
								y={16 + k * 5}
								width={[14, 12, 14, 8][k]}
								height="2"
								rx="1"
							/>
						{/each}
					{:else}
						{#each [0, 1, 2, 3] as k (k)}
							<rect
								class="stub grow"
								style="--j: {j * 2 + k * 0.4}"
								x={x + 4}
								y={16 + k * 5}
								width={[14, 12, 14, 8][k]}
								height="2"
								rx="1"
							/>
						{/each}
					{/if}
					<text class="num" x={x + 11} y="56">{j + 1}</text>
				</g>
			{/each}
			{#if pages > shownPages}
				<text class="num" x={xs[shownPages] + 11} y="30">+{pages - shownPages}</text>
			{/if}
		</svg>
	{:else if kind === 'joined'}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			<rect class="paper" x="40" y="2" width="60" height="68" rx="2" />
			{#each { length: Math.min(pages, 3) } as _, g (g)}
				<g class="grow" style="--j: {g}">
					<text class="num" x="49" y={16 + g * 21}>{g + 1}</text>
					<rect class="stub" x="58" y={8 + g * 21} width="34" height="2" rx="1" />
					<rect class="stub" x="58" y={13 + g * 21} width="30" height="2" rx="1" />
					<rect class="stub" x="58" y={18 + g * 21} width="20" height="2" rx="1" />
				</g>
			{/each}
		</svg>
	{:else if kind === 'fields'}
		<!-- One document's answers, under its name: one row of the table to come. -->
		<div class="card">
			<p class="card-head" class:mono={sources.length > 0}>{rows[0]}</p>
			<dl>
				{#each fieldNames.slice(0, 3) as name, j (j)}
					<div class="field">
						<dt>{name}</dt>
						<dd>
							{#if j === 0}
								<span class="value grow" style="--j: {j}; width: 90%"></span>
							{:else}
								{#each { length: j % 2 === 1 ? 2 : 1 } as _, k (k)}
									<span class="chip-stub grow" style="--j: {j + k * 0.4}"></span>
								{/each}
							{/if}
						</dd>
					</div>
				{/each}
			</dl>
		</div>
	{:else if kind === 'table'}
		{@const columns = fieldNames.slice(0, 3)}
		<table class="table" style="--cols: {columns.length}">
			<thead>
				<tr>
					<th></th>
					{#each columns as name, j (j)}<th>{name}</th>{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as name, r (r)}
					<tr class="grow-row" style="--j: {r}">
						<td class:mono={sources.length > 0}>{name}</td>
						{#each columns as _, j (j)}
							<td>
								{#if j === 0}<span class="value"></span>
								{:else}<span class="chips"
										>{#each { length: j % 2 === 1 ? 2 : 1 } as _, k (k)}<span class="chip-stub"
											></span>{/each}</span
									>{/if}
							</td>
						{/each}
					</tr>
				{/each}
				{#if sources.length > 3}
					<tr>
						<td class="mono" colspan={columns.length + 1}>+{sources.length - 2}</td>
					</tr>
				{/if}
			</tbody>
		</table>
	{:else if kind === 'wave' || kind === 'cut'}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			{#each { length: shownPieces } as _, p (p)}
				<!-- Cut apart, each piece moves from where it lay in the whole recording. -->
				{@const whole = ((shownPieces - 1) * CUT_GAP) / 2}
				{@const shift = kind === 'cut' ? p * CUT_GAP : whole}
				<g class={kind === 'cut' ? 'part' : ''} style="--dx: {whole - shift}px">
					{#each bars as h, k (k)}
						{#if pieceOf(k) === p}
							<rect
								class="bar"
								x={waveLeft + k * BAR_STEP + shift}
								y={30 - h / 2}
								width="2"
								height={h}
								rx="1"
							/>
						{/if}
					{/each}
					{#if kind === 'cut'}
						{@const from = cuts[p - 1] ?? 0}
						{@const to = cuts[p] ?? BARS}
						<text class="num" x={waveLeft + ((from + to) / 2) * BAR_STEP + p * CUT_GAP} y="62"
							>{p === shownPieces - 1 ? pieces : p + 1}</text
						>
					{/if}
				</g>
			{/each}
		</svg>
	{:else if kind === 'transcribe'}
		{@const xs = row(shownPieces, 20, 6)}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			{#each xs as x, p (p)}
				<rect class="paper" {x} y="12" width="20" height="34" rx="2" />
				{#each [0, 1, 2, 3] as k (k)}
					<rect
						class="stub grow"
						style="--j: {p * 1.5 + k * 0.4}"
						x={x + 4}
						y={18 + k * 6}
						width={[12, 10, 12, 7][k]}
						height="2"
						rx="1"
					/>
				{/each}
				<text class="num" x={x + 10} y="62">{p === shownPieces - 1 ? pieces : p + 1}</text>
			{/each}
		</svg>
	{:else if kind === 'transcript'}
		{@render transcript(mode === 'audio_transcription' && pieces > 1)}
	{:else if kind === 'link'}
		<div class="link"><Icon name="link" size={14} /><span class="mono">{video}</span></div>
	{:else if kind === 'send'}
		<div class="send">
			<span class="end"><Icon name="laptop" size={20} />{m.pipeline.thisPc}</span>
			<span class="wire"><span class="dot"></span></span>
			<span class="end"><Icon name="cloud" size={20} />{m.pipeline.google}</span>
		</div>
	{:else if kind === 'watch'}
		<svg width={W} height="72" viewBox="0 0 {W} 72">
			<rect class="paper" x="34" y="4" width="72" height="44" rx="4" />
			<path class="play" d="M64 18 L64 34 L78 26 Z" />
			<rect class="track" x="34" y="56" width="72" height="3" rx="1.5" />
			<rect class="sweep" x="34" y="56" width="72" height="3" rx="1.5" />
		</svg>
	{/if}
{/snippet}

<style>
	/* A quiet panel under the tasks; each step is a column, read left to right. */
	.diagram {
		container-type: inline-size;
		margin: var(--space-3) 0 0;
		padding: var(--space-2) var(--space-4) var(--space-4);
		border-radius: var(--radius-card);
		background: var(--surface-1);
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	.title {
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-secondary);
	}
	.stages {
		display: grid;
		grid-template-columns: repeat(var(--n), minmax(0, 1fr));
		gap: var(--space-5);
		margin: var(--space-2) 0 0;
		padding: 0;
		list-style: none;
	}
	/* An analysis has three steps, and its table needs the room. */
	.stages.analysis {
		grid-template-columns: minmax(0, 0.8fr) minmax(0, 1fr) minmax(0, 1.3fr);
	}
	.stage {
		position: relative;
		display: grid;
		align-content: start;
		gap: var(--space-1);
		min-width: 0;
	}
	.connector {
		position: absolute;
		top: calc(44px - 7px);
		left: calc(-1 * var(--space-5) / 2 - 7px);
		color: var(--line-hover);
	}
	.art {
		display: grid;
		place-items: center;
		height: 88px;
	}
	.art > :global(*) {
		min-width: 0;
		max-width: 100%;
	}
	.art svg {
		max-width: 100%;
		height: auto;
		overflow: visible;
	}
	.caption {
		margin: 0;
		font-size: var(--type-xs);
		line-height: 1.4;
		color: var(--text-secondary);
		text-align: center;
	}
	.note {
		margin: 0;
		font-size: 0.6875rem;
		line-height: 1.4;
		color: var(--text-muted);
		text-align: center;
	}
	.detail {
		margin: 0;
		overflow: hidden;
		font-size: 0.6875rem;
		color: var(--text-muted);
		text-align: center;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* The drawing's marks, in the interface's tokens so they follow the theme. */
	.paper {
		fill: var(--surface-2);
		stroke: var(--line-strong);
		stroke-width: 1;
	}
	.stub {
		fill: var(--line-hover);
	}
	.print {
		fill: var(--line-strong);
	}
	.time {
		fill: var(--text-muted);
	}
	.speaker-1 {
		fill: var(--speaker-1);
	}
	.speaker-2 {
		fill: var(--speaker-2);
	}
	.rule {
		stroke: var(--line-strong);
		stroke-width: 1;
	}
	.outline {
		fill: none;
		stroke: var(--line-hover);
		stroke-width: 1.5;
	}
	/* A name the fields ask for, found: the highlighter, with ink on it as always. */
	.mark {
		fill: var(--marker);
	}
	.on-mark {
		fill: var(--on-marker);
	}
	.bar {
		fill: var(--text-secondary);
		shape-rendering: crispEdges;
	}
	.num {
		font-family: var(--font-mono);
		font-size: 11px;
		fill: var(--text-muted);
		text-anchor: middle;
	}
	.play {
		fill: var(--text-muted);
	}
	.track {
		fill: var(--line);
	}
	.sweep {
		fill: var(--ink);
	}

	.card {
		display: grid;
		gap: 4px;
		width: 100%;
		max-width: 10rem;
		line-height: 1.3;
		margin: 0;
		padding: 6px 8px;
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	.card dl {
		display: grid;
		gap: 4px;
		margin: 0;
	}
	.card-head {
		margin: 0;
		overflow: hidden;
		padding-bottom: 4px;
		border-bottom: 1px solid var(--line);
		font-size: 0.6875rem;
		color: var(--text-body);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.field {
		display: grid;
		grid-template-columns: 3.75rem minmax(0, 1fr);
		align-items: center;
		gap: var(--space-2);
	}
	.field dt {
		overflow: hidden;
		font-size: 0.6875rem;
		font-weight: 500;
		color: var(--text-secondary);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.field dd {
		display: flex;
		gap: 3px;
		margin: 0;
	}
	.value,
	.chip-stub {
		display: block;
		height: 5px;
		border-radius: 3px;
		background: var(--line-hover);
	}
	.chip-stub {
		width: 1.25rem;
		height: 7px;
		border-radius: 2px;
		background: var(--chrome);
		box-shadow: inset 0 0 0 1px var(--line-strong);
	}

	.table {
		width: 100%;
		max-width: 18rem;
		border-collapse: collapse;
		table-layout: fixed;
		border: 1px solid var(--line-strong);
		background: var(--surface-2);
		font-size: 0.6875rem;
		line-height: 1.3;
	}
	.table th,
	.table td {
		overflow: hidden;
		padding: 2px 5px;
		border: 1px solid var(--line);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.table th {
		font-weight: 500;
		color: var(--text-secondary);
		text-align: left;
		background: var(--surface-1);
	}
	.table th:first-child,
	.table td:first-child {
		width: 40%;
	}
	.chips {
		display: flex;
		gap: 3px;
	}
	.table td.mono {
		color: var(--text-body);
	}
	.table .value {
		width: 80%;
	}

	.link {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		max-width: 100%;
		padding: 3px 8px;
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-tag);
		background: var(--surface-2);
		color: var(--text-secondary);
		font-size: 0.6875rem;
	}
	.link span {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.send {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		max-width: 8.5rem;
	}
	.end {
		display: grid;
		justify-items: center;
		gap: 2px;
		font-size: 0.6875rem;
		color: var(--text-secondary);
	}
	.wire {
		position: relative;
		flex: 1;
		height: 1px;
		margin-bottom: 1rem;
		background: var(--line-hover);
	}
	.dot {
		position: absolute;
		top: -3px;
		right: 0;
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--ink);
	}

	/* Playing: each step in turn, then its marks. The resting styles are the last frame, so
	   reduced motion (which removes every animation) shows it whole. */
	.playing .stage {
		--at: calc(var(--i) * 650ms);
		animation: appear 360ms ease-out var(--at) both;
	}
	.playing .connector {
		animation: appear 240ms ease-out calc(var(--at) - 120ms) both;
	}
	.playing .fan {
		animation: fan 520ms cubic-bezier(0.2, 0.7, 0.2, 1) calc(var(--at) + 120ms + var(--j) * 70ms)
			both;
	}
	.playing .part {
		animation: part 480ms ease-out calc(var(--at) + 160ms) both;
	}
	.playing .grow,
	.playing .marker {
		transform-box: fill-box;
		transform-origin: left center;
		animation: grow 260ms ease-out calc(var(--at) + 180ms + var(--j) * 120ms) both;
	}
	/* The names found light up while the next step fills in the fields. */
	.playing .marker {
		animation-delay: calc(650ms + 200ms + var(--j) * 180ms);
	}
	.playing .grow-row {
		animation: appear 260ms ease-out calc(var(--at) + 180ms + var(--j) * 160ms) both;
	}
	.playing .sweep {
		transform-box: fill-box;
		transform-origin: left center;
		animation: grow 900ms ease-in-out calc(var(--at) + 150ms) both;
	}
	.playing .dot {
		animation: travel 700ms ease-in-out calc(var(--at) + 150ms) both;
	}
	@keyframes appear {
		from {
			opacity: 0;
			transform: translateY(3px);
		}
	}
	@keyframes fan {
		from {
			opacity: 0.4;
			transform: translateX(var(--dx));
		}
	}
	@keyframes part {
		from {
			transform: translateX(var(--dx));
		}
	}
	@keyframes grow {
		from {
			transform: scaleX(0);
		}
	}
	@keyframes travel {
		from {
			right: calc(100% - 7px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.replay {
			display: none;
		}
	}

	/* A narrow form: two steps a row, read in order, without the arrows. */
	@container (max-width: 34rem) {
		.stages,
		.stages.analysis {
			grid-template-columns: repeat(2, minmax(0, 1fr));
			row-gap: var(--space-4);
		}
		.connector {
			display: none;
		}
		/* The table, the point of an analysis, keeps a whole row. */
		.analysis .stage:last-child {
			grid-column: 1 / -1;
		}
	}
</style>
