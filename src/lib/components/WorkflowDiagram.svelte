<script lang="ts">
	// How a workflow's steps follow each other, drawn from its own steps: the files, each step,
	// the researcher's check where the next step waits for it, and the files written. It plays
	// when shown and on request; under reduced motion it shows its last frame. Each stage is a
	// list item with its words, so the drawing is only decoration.
	import Icon from './Icon.svelte';
	import type { Task } from '#lib/bindings/Task.js';
	import { messages } from '#lib/i18n/index.js';

	type Shown = { task: Task; name: string; review: boolean };
	let {
		steps,
		compact = false
	}: {
		steps: Shown[];
		/** Smaller, for New job and the examples: no title or replay. */
		compact?: boolean;
	} = $props();

	const m = messages();
	const d = m.workflows.diagram;
	let replays = $state(0);

	type Stage =
		| { kind: 'files' }
		| { kind: 'step'; task: Task; caption: string; note: string }
		| { kind: 'check' }
		| { kind: 'written' };

	const stages = $derived<Stage[]>([
		{ kind: 'files' },
		...steps.flatMap((s, i): Stage[] => [
			{ kind: 'step', task: s.task, caption: s.name, note: m.task[s.task] },
			...(s.review && i < steps.length - 1 ? [{ kind: 'check' } as Stage] : [])
		]),
		{ kind: 'written' }
	]);
</script>

<figure class="workflow-diagram" class:compact>
	{#if !compact}
		<figcaption class="head">
			<span class="title">{d.title}</span>
			<button class="btn sm ghost" onclick={() => replays++}
				><Icon name="replay" size={14} />{d.replay}</button
			>
		</figcaption>
	{/if}
	{#key `${replays}/${stages.length}`}
		<ol class="stages">
			{#each stages as stage, i (i)}
				<li class="stage is-{stage.kind}" style="--i: {i}">
					{#if i > 0}<span class="connector" aria-hidden="true"><i class="dot"></i></span>{/if}
					<span class="tile" aria-hidden="true">
						{#if stage.kind === 'files'}
							<Icon name="file" size={compact ? 18 : 22} />
						{:else if stage.kind === 'step'}
							<Icon name={stage.task} size={compact ? 18 : 22} />
						{:else if stage.kind === 'check'}
							<Icon name="edit" size={compact ? 18 : 22} />
						{:else}
							<Icon name="download" size={compact ? 18 : 22} />
						{/if}
					</span>
					<span class="caption">
						{#if stage.kind === 'files'}{d.files}
						{:else if stage.kind === 'step'}{stage.caption}
						{:else if stage.kind === 'check'}{d.check}
						{:else}{d.written}{/if}
					</span>
					{#if stage.kind === 'step' && stage.caption !== stage.note}
						<span class="note">{stage.note}</span>
					{:else if stage.kind === 'check'}
						<span class="note">{d.checkNote}</span>
					{/if}
				</li>
			{/each}
		</ol>
	{/key}
</figure>

<style>
	/* A quiet panel, as New job's diagram: each stage a column, read left to right. */
	.workflow-diagram {
		/* Its steps fit the width the diagram is given, wherever it sits. */
		container-type: inline-size;
		margin: 0;
		padding: var(--space-2) var(--space-4) var(--space-4);
		border-radius: var(--radius-card);
		background: var(--surface-1);
	}
	.compact {
		padding: var(--space-3);
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
		display: flex;
		gap: var(--space-5);
		margin: var(--space-2) 0 0;
		padding: 0;
		list-style: none;
		/* Scrolls sideways only when the steps cannot fit; never up and down, which the stages'
		   entrance would otherwise set off with a scroll bar while they appear. */
		overflow-x: auto;
		overflow-y: hidden;
	}
	.compact .stages {
		margin: 0;
		gap: var(--space-4);
	}
	.stage {
		position: relative;
		flex: 1 1 0;
		min-width: 5.5rem;
		display: grid;
		justify-items: center;
		align-content: start;
		gap: var(--space-1);
		text-align: center;
	}
	.tile {
		display: grid;
		place-items: center;
		width: 2.75rem;
		height: 2.75rem;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		color: var(--text-secondary);
	}
	.compact .tile {
		width: 2.25rem;
		height: 2.25rem;
	}
	.is-step .tile {
		color: var(--text-bright);
	}
	/* The researcher's own stop: the highlighter, with ink on it. */
	.is-check .tile {
		background: var(--marker);
		border-color: transparent;
		color: var(--on-marker);
	}
	.caption {
		font-size: var(--type-xs);
		line-height: 1.35;
		color: var(--text-secondary);
	}
	.is-step .caption,
	.is-check .caption {
		color: var(--text-bright);
		font-weight: 500;
	}
	.note {
		font-size: 0.75rem;
		line-height: 1.35;
		color: var(--text-muted);
	}
	/* The line from the stage before, a mark travelling along it as the work moves on. */
	.connector {
		position: absolute;
		top: calc(1.375rem - 1px);
		right: calc(50% + 1.375rem + 4px);
		width: calc(100% - 2.75rem + var(--space-5) - 8px);
		height: 2px;
		background: var(--line);
	}
	.compact .connector {
		top: calc(1.125rem - 1px);
		right: calc(50% + 1.125rem + 4px);
		width: calc(100% - 2.25rem + var(--space-4) - 8px);
	}
	.dot {
		position: absolute;
		top: -2px;
		right: 0;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--ink);
		opacity: 0;
	}

	/* Each stage appears in turn after the mark has travelled to it; the check holds a moment
	   longer, as the work waits for the researcher. */
	.stage {
		--at: calc(var(--i) * 700ms);
		animation: appear 360ms ease-out var(--at) both;
	}
	.dot {
		animation: travel 600ms ease-in-out calc(var(--at) - 600ms) both;
	}
	.is-check .tile {
		animation: wait 1100ms ease-in-out calc(var(--at) + 200ms) 2 both;
	}
	@keyframes appear {
		from {
			opacity: 0;
		}
	}
	@keyframes travel {
		0% {
			right: calc(100% - 6px);
			opacity: 1;
		}
		90% {
			opacity: 1;
		}
		100% {
			right: 0;
			opacity: 0;
		}
	}
	@keyframes wait {
		50% {
			transform: scale(1.08);
		}
	}
	/* In a narrow diagram (a phone-sized window, or a card beside the sidebar) the steps share
	   the width there is, their captions wrapping, rather than scroll sideways. */
	@container (max-width: 34rem) {
		.stage {
			min-width: 0;
		}
		.caption {
			overflow-wrap: anywhere;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.stage,
		.dot,
		.is-check .tile {
			animation: none;
		}
		.head .btn {
			display: none;
		}
	}
</style>
