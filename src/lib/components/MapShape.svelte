<svelte:options namespace="svg" />

<script lang="ts">
	// One mark of the map or of its legend: a shape and a tone per group, so that a group is told
	// by its shape as well as its colour; documents in no group are hollow.
	import type { Shape } from '#lib/meaning.js';

	let { shape, tone, x, y, r }: { shape: Shape; tone: number; x: number; y: number; r: number } =
		$props();
	const poly = (pts: [number, number][]) =>
		pts.map(([a, b]) => `${x + a * r},${y + b * r}`).join(' ');
</script>

{#if shape === 'square'}
	<rect
		class="mark tone-{tone}"
		x={x - r * 0.85}
		y={y - r * 0.85}
		width={r * 1.7}
		height={r * 1.7}
	/>
{:else if shape === 'triangle'}
	<polygon
		class="mark tone-{tone}"
		points={poly([
			[0, -1.1],
			[1, 0.75],
			[-1, 0.75]
		])}
	/>
{:else if shape === 'diamond'}
	<polygon
		class="mark tone-{tone}"
		points={poly([
			[0, -1.15],
			[1.15, 0],
			[0, 1.15],
			[-1.15, 0]
		])}
	/>
{:else if shape === 'cross'}
	<polygon
		class="mark tone-{tone}"
		points={poly([
			[-0.35, -1],
			[0.35, -1],
			[0.35, -0.35],
			[1, -0.35],
			[1, 0.35],
			[0.35, 0.35],
			[0.35, 1],
			[-0.35, 1],
			[-0.35, 0.35],
			[-1, 0.35],
			[-1, -0.35],
			[-0.35, -0.35]
		])}
	/>
{:else if shape === 'ring'}
	<circle class="ring tone-{tone}" cx={x} cy={y} r={r * 0.75} stroke-width={r * 0.5} />
{:else if shape === 'hollow'}
	<circle class="hollow" cx={x} cy={y} r={r * 0.8} stroke-width={r * 0.3} />
{:else}
	<circle class="mark tone-{tone}" cx={x} cy={y} {r} />
{/if}

<style>
	.mark {
		fill: var(--tone);
	}
	.ring {
		fill: none;
		stroke: var(--tone);
	}
	.hollow {
		fill: var(--surface-2);
		stroke: var(--text-muted);
	}
	/* Ink, the speakers' teal and amber, grey, and their darker text tones: none is the
	 * highlighter's yellow or the hand's blue, whose meanings are their own. */
	.tone-0 {
		--tone: var(--text-bright);
	}
	.tone-1 {
		--tone: var(--speaker-1);
	}
	.tone-2 {
		--tone: var(--speaker-2);
	}
	.tone-3 {
		--tone: var(--text-muted);
	}
	.tone-4 {
		--tone: var(--speaker-1-text);
	}
	.tone-5 {
		--tone: var(--speaker-2-text);
	}
</style>
