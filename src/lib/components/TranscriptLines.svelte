<script lang="ts">
	// A transcript's timed lines: when each begins, who speaks, what was said. Clicking a line
	// plays the recording or video beside it from that moment.
	import type { Segment } from '#lib/bindings/Segment.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		segments,
		offsetMs,
		current = () => false,
		follow = false,
		onpick
	}: {
		segments: Segment[];
		/** Where the piece starts in the recording, so times read on its clock. */
		offsetMs: number;
		/** The line being heard now. */
		current?: (segment: Segment) => boolean;
		/** Keep the line being heard in view, as while the recording or video plays. */
		follow?: boolean;
		onpick: (segment: Segment) => void;
	} = $props();

	const m = messages();

	function clock(ms: number): string {
		const s = Math.floor(ms / 1000);
		return [s / 3600, (s / 60) % 60, s % 60]
			.map((n) => String(Math.floor(n)).padStart(2, '0'))
			.join(':');
	}

	/** Each speaker keeps one of two colours, in the order they first speak. */
	const voices = $derived([...new Set(segments.flatMap((s) => (s.speaker ? [s.speaker] : [])))]);
	const voice = (speaker: string) => (voices.indexOf(speaker) % 2) + 1;

	let list = $state<HTMLOListElement | null>(null);
	const heard = $derived(segments.findIndex((s) => current(s)));
	/** When the researcher last scrolled the lines; following waits a moment after it. */
	let scrolledAt = 0;
	$effect(() => {
		const el = list;
		if (!el) return;
		const scrolled = () => (scrolledAt = Date.now());
		const events = ['wheel', 'touchmove', 'keydown'] as const;
		for (const name of events) el.addEventListener(name, scrolled, { passive: true });
		return () => events.forEach((name) => el.removeEventListener(name, scrolled));
	});

	$effect(() => {
		const i = heard;
		if (!follow || i < 0 || !list || Date.now() - scrolledAt < 4_000) return;
		const reduce = matchMedia('(prefers-reduced-motion: reduce)').matches;
		list.children[i]?.scrollIntoView({ block: 'nearest', behavior: reduce ? 'auto' : 'smooth' });
	});
</script>

<ol class="lines" aria-label={m.review.lines} bind:this={list}>
	{#each segments as s, k (k)}
		{@const time = clock(offsetMs + Number(s.start_ms))}
		<li>
			<button
				class="line"
				class:now={current(s)}
				aria-current={current(s) ? 'true' : undefined}
				onclick={() => onpick(s)}
			>
				<span class="num stamp">{time}</span>
				{#if s.speaker}<span class="speaker v{voice(s.speaker)}">{s.speaker}</span>{/if}
				<span class="said">{s.text}</span>
			</button>
		</li>
	{/each}
</ol>

<style>
	.lines {
		list-style: none;
		margin: 0;
		padding: 0;
		overflow: auto;
		min-height: 0;
		display: grid;
		align-content: start;
		gap: 2px;
	}
	/* One turn: its time, who speaks, what was said. */
	.line {
		display: grid;
		grid-template-columns: 4.75rem auto minmax(0, 1fr);
		gap: var(--space-3);
		align-items: baseline;
		width: 100%;
		padding: var(--space-2) var(--space-3);
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
		background: var(--wash);
	}
	.line.now {
		background: var(--surface-2);
		box-shadow: var(--shadow-raised);
	}
	.stamp {
		font-family: var(--font-mono);
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.now .stamp {
		color: var(--text-bright);
		font-weight: 500;
	}
	.speaker {
		justify-self: start;
		padding: 1px var(--space-2);
		border-radius: var(--radius-tag);
		font-size: var(--type-xs);
		font-weight: 500;
		white-space: nowrap;
	}
	.speaker.v1 {
		background: var(--speaker-1-wash);
		color: var(--speaker-1-text);
	}
	.speaker.v2 {
		background: var(--speaker-2-wash);
		color: var(--speaker-2-text);
	}
	.said {
		font-family: var(--font-text);
		font-size: var(--type-lg);
		line-height: 1.55;
		color: var(--text-body);
	}
	.line:not(:has(.speaker)) {
		grid-template-columns: 4.75rem minmax(0, 1fr);
	}
</style>
