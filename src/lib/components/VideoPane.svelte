<script lang="ts">
	// A YouTube video next to its transcript, on the dark stage like a scan. Nothing is asked of
	// YouTube until the researcher plays: a poster stands in, and the first play (or a click on a
	// line) loads YouTube's privacy-enhanced player there. The player is driven by messages, so
	// no YouTube script runs in the app. A video its owner keeps off other sites, or an offline
	// PC, gets a card that opens it on YouTube instead, and lines then open it there.
	import Icon from './Icon.svelte';
	import TranscriptLines from './TranscriptLines.svelte';
	import type { Segment } from '#lib/bindings/Segment.js';
	import { messages } from '#lib/i18n/index.js';
	import { openLink } from '#lib/state.svelte.js';
	import { EMBED_ORIGIN, embedUrl, playerCommand, playerNews, videoAt } from '#lib/youtube.js';

	let { url, title, segments }: { url: string; title: string | null; segments: Segment[] } =
		$props();

	const m = messages();
	/** Set on the first play: where to start, which loads the player. */
	let startMs = $state<number | null>(null);
	let frame = $state<HTMLIFrameElement | null>(null);
	/** Seconds into the video, as the player last reported. */
	let now = $state(0);
	let playing = $state(false);
	let blocked = $state(false);
	/** Whether the player has answered since it loaded. */
	let heard = false;

	const src = $derived(
		startMs === null
			? null
			: embedUrl(url, location.origin, {
					language: m.locale.tag,
					startMs: startMs,
					autoplay: true
				})
	);

	function send(func: string, args: unknown[] = []) {
		frame?.contentWindow?.postMessage(playerCommand(func, args), EMBED_ORIGIN);
	}

	// Once loaded, ask the player to report its time; one that never answers cannot play here.
	function loaded() {
		heard = false;
		frame?.contentWindow?.postMessage(
			JSON.stringify({ event: 'listening', id: 1, channel: 'widget' }),
			EMBED_ORIGIN
		);
		setTimeout(() => {
			if (!heard) blocked = true;
		}, 10_000);
	}

	$effect(() => {
		const onMessage = (e: MessageEvent) => {
			if (e.origin !== EMBED_ORIGIN || e.source !== frame?.contentWindow) return;
			const news = playerNews(e.data);
			if (!news) return;
			heard = true;
			// 101 and 150: the owner does not allow the video on other sites.
			if (news.error !== undefined && news.error !== 5) blocked = true;
			if (news.time !== undefined) now = news.time;
			if (news.playing !== undefined) playing = news.playing;
		};
		window.addEventListener('message', onMessage);
		return () => window.removeEventListener('message', onMessage);
	});

	function jump(segment: Segment) {
		const ms = Number(segment.start_ms);
		if (blocked) {
			void openLink(videoAt(url, ms));
			return;
		}
		now = ms / 1000;
		if (startMs === null) {
			startMs = ms;
			return;
		}
		send('seekTo', [ms / 1000, true]);
		send('playVideo');
	}

	const current = (s: Segment) => now * 1000 >= Number(s.start_ms) && now * 1000 < Number(s.end_ms);
</script>

<div class="box">
	<div class="video">
		<div class="stage">
			<div class="screen">
				{#if blocked}
					<div class="card">
						<p>{m.review.videoBlocked}</p>
						<button class="btn on-stage" onclick={() => openLink(videoAt(url, now * 1000))}
							>{m.review.openVideo}</button
						>
					</div>
				{:else if src}
					<iframe
						bind:this={frame}
						{src}
						title={m.review.videoPlayer}
						allow="autoplay; encrypted-media; picture-in-picture; fullscreen"
						referrerpolicy="strict-origin-when-cross-origin"
						onload={loaded}
					></iframe>
				{:else}
					<button class="poster" onclick={() => (startMs = 0)}>
						<span class="play" aria-hidden="true"><Icon name="play" size={18} /></span>
						<span class="poster-text">
							<span class="poster-title">{title ?? m.review.playVideo}</span>
							<span class="poster-note">{m.review.playVideoNote}</span>
						</span>
						<span class="sr-only">{m.review.playVideo}</span>
					</button>
				{/if}
			</div>
		</div>
		<div class="under">
			<span class="muted small">{blocked ? m.review.videoHintOpen : m.review.videoHint}</span>
			{#if !blocked}
				<button class="btn sm ghost" onclick={() => openLink(videoAt(url, now * 1000))}
					>{m.review.openVideo}</button
				>
			{/if}
		</div>
		<div class="transcript">
			{#if segments.length > 0}
				<TranscriptLines {segments} offsetMs={0} {current} follow={playing} onpick={jump} />
			{:else}
				<p class="muted small pad">{m.review.noTimings}</p>
			{/if}
		</div>
	</div>
</div>

<style>
	/* Sized from the pane, not the window: the stage takes at most half the pane's height, and
	   in a wide, short pane (the review stacked in a half-screen window) it sits beside the
	   lines instead of above them. */
	.box {
		container-type: size;
		height: 100%;
		min-height: 0;
	}
	.video {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		grid-template-rows: auto auto minmax(0, 1fr);
		gap: var(--space-3);
		height: 100%;
		min-height: 0;
	}
	.stage {
		display: grid;
		place-items: center;
		height: min(26rem, 50cqh);
		min-height: 0;
		padding: var(--space-3);
		border-radius: var(--radius-card);
		background: var(--stage);
	}
	.screen {
		position: relative;
		height: 100%;
		max-width: 100%;
		aspect-ratio: 16 / 9;
		border-radius: var(--radius-control);
		overflow: hidden;
		background: var(--stage-raised);
	}
	.screen iframe {
		display: block;
		width: 100%;
		height: 100%;
		border: 0;
	}
	/* The poster: the app's own play button and the video's title, until the first play. */
	.poster {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		width: 100%;
		height: 100%;
		padding: var(--space-4) var(--space-5);
		border: 0;
		background: none;
		color: var(--on-stage);
		font: inherit;
		text-align: start;
		cursor: pointer;
	}
	.play {
		display: grid;
		place-items: center;
		width: 3rem;
		height: 3rem;
		flex: none;
		border-radius: 50%;
		background: var(--on-stage);
		color: var(--stage);
		transition: transform 120ms ease-out;
	}
	.poster:hover .play {
		transform: scale(1.06);
	}
	.poster-text {
		display: grid;
		gap: var(--space-1);
		min-width: 0;
	}
	.poster-title {
		font-size: var(--type-md);
		font-weight: 500;
		line-height: 1.3;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.poster-note {
		font-size: var(--type-xs);
		color: var(--on-stage-muted);
	}
	.card {
		display: grid;
		align-content: center;
		justify-items: start;
		gap: var(--space-3);
		height: 100%;
		padding: var(--space-4) var(--space-5);
		color: var(--on-stage);
		font-size: var(--type-sm);
	}
	.card p {
		margin: 0;
		max-width: 40ch;
	}
	.btn.on-stage {
		background: var(--on-stage);
		border-color: var(--on-stage);
		color: var(--stage);
	}
	.under {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		min-width: 0;
		padding-left: var(--space-3);
	}
	.transcript {
		display: grid;
		min-height: 0;
	}
	.small {
		font-size: var(--type-sm);
	}
	.pad {
		padding: 0 var(--space-3);
	}
	@container (min-aspect-ratio: 2 / 1) {
		.video {
			grid-template-columns: minmax(0, 45%) minmax(0, 1fr);
			grid-template-rows: auto minmax(0, 1fr);
		}
		.stage {
			grid-row: 1 / -1;
			height: 100%;
		}
		/* Here the width is what limits the picture. */
		.screen {
			width: 100%;
			height: auto;
			max-height: 100%;
		}
	}
</style>
