<script lang="ts">
	// The recording next to its transcript: the piece under review plays here, the line being
	// spoken is highlighted, and clicking a line jumps to it. Times are shown on the recording's
	// clock; the player itself plays only this piece.
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

	const playing = (s: Segment) => now * 1000 >= Number(s.start_ms) && now * 1000 < Number(s.end_ms);
</script>

<div class="audio">
	{#if url}
		<audio controls src={url} bind:this={player} bind:currentTime={now}></audio>
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
						<span class="mono time">{clock(offsetMs + Number(s.start_ms))}</span>
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
		gap: var(--space-2);
		min-height: 0;
	}
	audio {
		width: 100%;
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
		gap: var(--space-2);
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
	}
	.line:hover {
		background: var(--accent-wash);
	}
	.line.now {
		background: var(--accent-chip);
		box-shadow: inset 0 0 0 1px var(--accent-border);
	}
	.time {
		color: var(--text-muted);
		font-size: var(--type-sm);
		flex: none;
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
