<script lang="ts">
	// The recording next to its transcript: the piece under review plays here, the line being
	// spoken is highlighted, and clicking a line jumps to it. Times are shown on the recording's
	// clock; the player itself plays only this piece.
	import Icon from './Icon.svelte';
	import TranscriptLines from './TranscriptLines.svelte';
	import type { Segment } from '#lib/bindings/Segment.js';
	import { messages } from '#lib/i18n/index.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		sourceId,
		index,
		offsetMs,
		segments
	}: {
		jobId: string;
		sourceId: string;
		index: number;
		offsetMs: number;
		segments: Segment[];
	} = $props();

	const m = messages();
	let url = $state<string | null>(null);
	let failed = $state(false);
	let player = $state<HTMLAudioElement | null>(null);
	let now = $state(0);
	let duration = $state(0);
	let paused = $state(true);

	// Load the piece whenever it changes, and release the previous one's memory.
	$effect(() => {
		const i = index;
		let created: string | null = null;
		url = null;
		failed = false;
		api
			.pieceAudio(jobId, sourceId, i)
			.then((bytes) => {
				if (index !== i) return;
				const head = new Uint8Array(bytes.slice(0, 4));
				const ogg = String.fromCharCode(...head) === 'OggS';
				created = URL.createObjectURL(new Blob([bytes], { type: ogg ? 'audio/ogg' : 'audio/wav' }));
				url = created;
			})
			.catch(() => (failed = true));
		return () => {
			if (created) URL.revokeObjectURL(created);
		};
	});

	function clock(ms: number): string {
		const s = Math.floor(ms / 1000);
		return [s / 3600, (s / 60) % 60, s % 60]
			.map((n) => String(Math.floor(n)).padStart(2, '0'))
			.join(':');
	}

	function jump(segment: Segment) {
		if (!player) return;
		player.currentTime = Number(segment.start_ms) / 1000;
		void player.play();
	}

	function toggle() {
		if (!player) return;
		if (player.paused) void player.play();
		else player.pause();
	}

	const playing = (s: Segment) => now * 1000 >= Number(s.start_ms) && now * 1000 < Number(s.end_ms);
	/** The piece's length in seconds; 0 until the player has read it. */
	const length = $derived(Number.isFinite(duration) ? duration : 0);
	const done = $derived(length > 0 ? (100 * now) / length : 0);
</script>

<div class="audio">
	{#if url}
		<audio
			src={url}
			preload="metadata"
			bind:this={player}
			bind:currentTime={now}
			bind:duration
			bind:paused
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
					if (player) player.currentTime = Number(e.currentTarget.value);
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
