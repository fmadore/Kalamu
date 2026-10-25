<script lang="ts">
	// The recording next to its transcript: the piece under review plays here, the line being
	// spoken is highlighted, and clicking a line jumps to it. Times are shown on the recording's
	// clock; the player itself plays only this piece.
	import Icon from './Icon.svelte';
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
		<ol class="lines" aria-label={m.review.lines}>
			{#each segments as s, k (k)}
				<li>
					<button class="line" class:now={playing(s)} onclick={() => jump(s)}>
						<span class="num stamp">{clock(offsetMs + Number(s.start_ms))}</span>
						{#if s.speaker}<strong class="speaker">{s.speaker}</strong>{/if}
						<span>{s.text}</span>
					</button>
				</li>
			{/each}
		</ol>
	{:else}
		<p class="muted pad small">{m.review.noTimings}</p>
	{/if}
</div>

<style>
	.audio {
		display: grid;
		grid-template-rows: auto 1fr;
		gap: var(--space-3);
		min-height: 0;
	}
	/* The player rests on the desk like the scan does: a white strip, not the browser's own. */
	.player {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-4) var(--space-2) var(--space-2);
		background: var(--paper);
		box-shadow: var(--shadow-flyout);
	}
	.play {
		display: grid;
		place-items: center;
		width: var(--control-md);
		height: var(--control-md);
		flex: none;
		border: 0;
		border-radius: 50%;
		background: var(--accent);
		color: var(--on-accent);
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.play:hover {
		background: var(--accent-soft);
	}
	.time {
		flex: none;
		font-size: var(--type-sm);
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
		background: linear-gradient(
			to right,
			var(--accent) var(--done, 0%),
			var(--line-strong) var(--done, 0%)
		);
	}
	.scrub::-webkit-slider-thumb {
		width: 14px;
		height: 14px;
		margin-top: -5px;
		border: 2px solid var(--paper);
		border-radius: 50%;
		background: var(--accent);
		box-shadow: 0 0 0 1px var(--accent);
		appearance: none;
	}
	.scrub::-moz-range-track {
		height: 4px;
		border-radius: 2px;
		background: var(--line-strong);
	}
	.scrub::-moz-range-progress {
		height: 4px;
		border-radius: 2px;
		background: var(--accent);
	}
	.scrub::-moz-range-thumb {
		width: 10px;
		height: 10px;
		border: 2px solid var(--paper);
		border-radius: 50%;
		background: var(--accent);
	}
	.lines {
		list-style: none;
		margin: 0;
		padding: 0;
		overflow: auto;
		min-height: 0;
	}
	.line {
		display: flex;
		gap: var(--space-3);
		align-items: baseline;
		width: 100%;
		padding: var(--space-1) var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: none;
		color: inherit;
		font: inherit;
		text-align: start;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.line:hover {
		background: var(--accent-wash);
	}
	.line.now {
		background: var(--tint);
	}
	.stamp {
		color: var(--text-muted);
		font-size: var(--type-sm);
		flex: none;
	}
	.now .stamp {
		color: var(--text-secondary);
	}
	.speaker {
		flex: none;
	}
	.pad {
		padding: var(--space-3);
	}
	.small {
		font-size: var(--type-sm);
	}
</style>
