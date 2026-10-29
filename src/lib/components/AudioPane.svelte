<script lang="ts">
	// The recording next to its transcript: the piece under review plays here, the line being
	// spoken is highlighted, and clicking a line jumps to it. Times are shown on the recording's
	// clock; the player plays only this piece, from the recording itself when it can. A search
	// that goes to a line plays it too, once the piece is loaded.
	import { untrack } from 'svelte';
	import Icon from './Icon.svelte';
	import TranscriptLines from './TranscriptLines.svelte';
	import type { PieceAudio } from '#lib/bindings/PieceAudio.js';
	import type { Segment } from '#lib/bindings/Segment.js';
	import { messages } from '#lib/i18n/index.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		sourceId,
		index,
		offsetMs,
		segments,
		seek = null
	}: {
		jobId: string;
		sourceId: string;
		index: number;
		offsetMs: number;
		segments: Segment[];
		/** Play from this time into the piece; a new `id` plays again. */
		seek?: { ms: number; id: number } | null;
	} = $props();

	const m = messages();
	let audio = $state<PieceAudio | null>(null);
	let url = $state<string | null>(null);
	let failed = $state(false);
	let player = $state<HTMLAudioElement | null>(null);
	/** Where the player is in the file, in seconds. */
	let position = $state(0);
	let duration = $state(0);
	let paused = $state(true);
	let request = 0;

	// Where the piece starts and ends in the file, in seconds: a span of the recording itself,
	// or the whole of a converted copy.
	const start = $derived(audio ? Number(audio.start_ms) / 1000 : 0);
	const end = $derived(
		audio && Number(audio.end_ms) > 0
			? Number(audio.end_ms) / 1000
			: Number.isFinite(duration)
				? duration
				: 0
	);
	/** Seconds into the piece. */
	const now = $derived(Math.max(0, position - start));

	function load(i: number, convert: boolean) {
		const mine = ++request;
		audio = null;
		url = null;
		failed = false;
		position = 0;
		api
			.pieceAudio(jobId, sourceId, i, convert)
			.then(async (found) => {
				const src = await api.fileUrl(found.path);
				if (mine !== request) return;
				audio = found;
				url = src;
			})
			.catch(() => {
				if (mine === request) failed = true;
			});
	}

	// Load the piece whenever it changes.
	$effect(() => load(index, false));

	// The player could not read the recording after all: ask for a converted copy instead.
	function unplayable() {
		if (audio?.original) load(index, true);
		else failed = true;
	}

	/** Where to play from once the piece has loaded, in milliseconds into it. */
	let cued: number | null = null;

	function ready() {
		if (player && start > 0) player.currentTime = start;
		if (cued !== null) playFrom(cued);
	}

	$effect(() => {
		const s = seek;
		if (!s) return;
		untrack(() => {
			if (player && audio && player.readyState > 0) playFrom(s.ms);
			else cued = s.ms;
		});
	});

	// The recording plays on past the piece; stop where the piece ends.
	function tick() {
		if (player && end > 0 && player.currentTime >= end) {
			player.pause();
			player.currentTime = end;
		}
	}

	function clock(ms: number): string {
		const s = Math.floor(ms / 1000);
		return [s / 3600, (s / 60) % 60, s % 60]
			.map((n) => String(Math.floor(n)).padStart(2, '0'))
			.join(':');
	}

	function playFrom(ms: number) {
		cued = null;
		if (!player) return;
		player.currentTime = start + ms / 1000;
		void player.play();
	}

	const jump = (segment: Segment) => playFrom(Number(segment.start_ms));

	function toggle() {
		if (!player) return;
		if (!player.paused) return player.pause();
		if (end > 0 && player.currentTime >= end) player.currentTime = start;
		void player.play();
	}

	const playing = (s: Segment) => now * 1000 >= Number(s.start_ms) && now * 1000 < Number(s.end_ms);
	/** The piece's length in seconds; 0 until the player has read it. */
	const length = $derived(Math.max(0, end - start));
	const done = $derived(length > 0 ? (100 * now) / length : 0);
</script>

<div class="audio">
	{#if url}
		<audio
			src={url}
			preload="metadata"
			bind:this={player}
			bind:currentTime={position}
			bind:duration
			bind:paused
			onloadedmetadata={ready}
			ontimeupdate={tick}
			onerror={unplayable}
		></audio>
		<div class="player">
			<button
				class="play"
				aria-label={paused ? m.review.play : m.review.pausePlayback}
				title={paused ? m.review.play : m.review.pausePlayback}
				onclick={toggle}><Icon name={paused ? 'play' : 'pause'} size={14} /></button
			>
			<span class="time num">{clock(offsetMs + now * 1000)}</span>
			<input
				class="scrub"
				type="range"
				min="0"
				max={length}
				step="0.1"
				value={now}
				style:--done="{done}%"
				aria-label={m.review.position}
				aria-valuetext={clock(offsetMs + now * 1000)}
				oninput={(e) => {
					if (player) player.currentTime = start + Number(e.currentTarget.value);
				}}
			/>
			{#if length > 0}<span class="time num muted">{clock(offsetMs + length * 1000)}</span>{/if}
		</div>
	{:else if failed}
		<p class="muted pad">{m.review.noAudio}</p>
	{:else}
		<p class="muted pad">{m.common.loading}</p>
	{/if}
	{#if segments.length > 0}
		<TranscriptLines {segments} {offsetMs} current={playing} follow={!paused} onpick={jump} />
	{:else}
		<p class="muted pad small">{m.review.noTimings}</p>
	{/if}
</div>

<style>
	.audio {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		gap: var(--space-3);
		min-height: 0;
		height: 100%;
	}
	/* The player: a round ink button, mono times and a thin track filled in ink. */
	.player {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-4) var(--space-2) var(--space-2);
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-raised);
	}
	.play {
		display: grid;
		place-items: center;
		width: 2.5rem;
		height: 2.5rem;
		flex: none;
		border: 0;
		border-radius: 50%;
		background: var(--ink);
		color: var(--on-ink);
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.play:hover {
		background: var(--ink-hover);
	}
	.time {
		flex: none;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
	}
	.scrub {
		flex: 1;
		min-width: 4rem;
		height: var(--control-sm);
		margin: 0;
		background: transparent;
		cursor: pointer;
		appearance: none;
	}
	.scrub::-webkit-slider-runnable-track {
		height: 4px;
		border-radius: 2px;
		background: linear-gradient(to right, var(--ink) var(--done, 0%), var(--line) var(--done, 0%));
	}
	.scrub::-webkit-slider-thumb {
		width: 14px;
		height: 14px;
		margin-top: -5px;
		border: 2px solid var(--surface-2);
		border-radius: 50%;
		background: var(--ink);
		box-shadow: 0 0 0 1px var(--ink);
		appearance: none;
	}
	.scrub::-moz-range-track {
		height: 4px;
		border-radius: 2px;
		background: var(--line);
	}
	.scrub::-moz-range-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--ink);
	}
	.scrub::-moz-range-thumb {
		width: 10px;
		height: 10px;
		border: 2px solid var(--surface-2);
		border-radius: 50%;
		background: var(--ink);
	}
	.pad {
		padding: var(--space-3);
	}
	.small {
		font-size: var(--type-sm);
	}
</style>
