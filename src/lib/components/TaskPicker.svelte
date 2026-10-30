<script lang="ts">
	// The tasks of a new job as one joined choice, two by two: reading pages, working on texts,
	// spoken sources. Each is named and explained in a line.
	import { messages } from '#lib/i18n/index.js';
	import { type Mode, MODES } from '#lib/newJob.js';

	let {
		mode,
		onpick
	}: {
		/** The task chosen. */
		mode: Mode;
		/** A task was picked, the one already chosen included. */
		onpick: (mode: Mode) => void;
	} = $props();
	const m = messages();
</script>

<div class="tasks">
	{#each MODES as t (t)}
		<button
			class="task"
			class:selected={mode === t}
			aria-pressed={mode === t}
			onclick={() => onpick(t)}
		>
			<span class="task-name"><span class="radio" aria-hidden="true"></span>{m.task[t]}</span>
			<span class="task-hint">{m.task[`${t}Hint`]}</span>
		</button>
	{/each}
</div>

<style>
	.tasks {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		margin-top: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	.task {
		display: grid;
		align-content: start;
		gap: var(--space-1);
		padding: var(--space-3) var(--space-4) var(--space-4);
		border: 0;
		background: transparent;
		color: var(--text-body);
		text-align: left;
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.task:nth-child(even) {
		border-left: 1px solid var(--line);
	}
	.task:nth-child(n + 3) {
		border-top: 1px solid var(--line);
	}
	.task:nth-child(1) {
		border-radius: var(--radius-card) 0 0 0;
	}
	.task:nth-child(2) {
		border-radius: 0 var(--radius-card) 0 0;
	}
	.task:nth-child(5) {
		border-radius: 0 0 0 var(--radius-card);
	}
	.task:nth-child(6) {
		border-radius: 0 0 var(--radius-card) 0;
	}
	.task:hover {
		background: var(--surface-1);
	}
	.task.selected {
		background: var(--surface-2);
		box-shadow: inset 0 0 0 1.5px var(--ink);
	}
	.task-name {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--type-md);
		font-weight: 500;
		color: var(--text-bright);
	}
	.selected .task-name {
		font-weight: 600;
	}
	.radio {
		width: 0.875rem;
		height: 0.875rem;
		border-radius: 50%;
		border: 1.5px solid var(--line-hover);
		flex: none;
	}
	.selected .radio {
		border: 4px solid var(--ink);
	}
	.task-hint {
		padding-left: 1.375rem;
		font-size: var(--type-xs);
		line-height: 1.45;
		color: var(--text-muted);
	}
	@media (max-width: 760px) {
		.tasks {
			grid-template-columns: 1fr;
		}
		.task:nth-child(n) {
			border-left: 0;
			border-top: 1px solid var(--line);
			border-radius: 0;
		}
		.task:first-child {
			border-top: 0;
			border-radius: var(--radius-card) var(--radius-card) 0 0;
		}
		.task:last-child {
			border-radius: 0 0 var(--radius-card) var(--radius-card);
		}
	}
</style>
