<script lang="ts">
	// Sends a result's text, as the researcher corrected it, into a new round: an analysis, a
	// correction, or a saved workflow. The text is frozen first (snapshots, #47), so nothing is
	// read again and a later correction does not change what the new round read.
	import Icon from './Icon.svelte';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { go, type FromProject } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		setupId,
		sourceId = null,
		project,
		small = false,
		onerror
	}: {
		jobId: string;
		setupId: string;
		/** One document's text only; every source of the job when absent. */
		sourceId?: string | null;
		/** The project it is sent from, whose documents the new jobs are tied to. */
		project?: FromProject;
		small?: boolean;
		onerror: (message: string) => void;
	} = $props();

	const m = messages();
	const u = m.useText;
	let menu = $state<HTMLDetailsElement | null>(null);
	let busy = $state(false);

	type Choice = 'analyse' | 'correct' | 'workflow';
	const choices: Choice[] = ['analyse', 'correct', 'workflow'];

	async function choose(choice: Choice) {
		if (menu) menu.open = false;
		busy = true;
		try {
			const sources = await api.snapshotJob(jobId, setupId, sourceId);
			if (choice === 'workflow') go({ name: 'workflows', sources, project });
			else
				go({
					name: 'new',
					task: choice === 'analyse' ? 'analysis' : 'text_correction',
					sources,
					project
				});
		} catch (e) {
			onerror(describeError(e, m));
		} finally {
			busy = false;
		}
	}
</script>

<details class="use-text" bind:this={menu}>
	<summary class="btn" class:sm={small} aria-disabled={busy} title={u.hint}>
		{small ? u.short : u.button}<Icon name="chevron" size={12} />
	</summary>
	<ul class="menu" role="list">
		{#each choices as choice (choice)}
			<li>
				<button class="item" onclick={() => choose(choice)} disabled={busy}>
					<Icon
						name={choice === 'analyse'
							? 'analysis'
							: choice === 'correct'
								? 'text_correction'
								: 'workflow'}
						size={16}
					/>
					<span>
						<span class="label">{u[choice]}</span>
						<span class="sub">{u[`${choice}Hint`]}</span>
					</span>
				</button>
			</li>
		{/each}
	</ul>
</details>

<style>
	.use-text {
		position: relative;
		display: inline-block;
	}
	summary {
		list-style: none;
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		cursor: pointer;
	}
	summary::-webkit-details-marker {
		display: none;
	}
	.use-text[open] summary :global(svg) {
		transform: rotate(90deg);
	}
	.menu {
		position: absolute;
		right: 0;
		top: calc(100% + 4px);
		z-index: 30;
		width: 20rem;
		margin: 0;
		padding: var(--space-1);
		list-style: none;
		background: var(--surface-2);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		box-shadow: var(--shadow-flyout);
	}
	.item {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		width: 100%;
		padding: var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: none;
		color: var(--text-body);
		text-align: left;
		cursor: pointer;
	}
	.item:hover:not(:disabled),
	.item:focus-visible {
		background: var(--wash);
	}
	.item :global(svg) {
		flex: none;
		margin-top: 2px;
	}
	.label {
		display: block;
		color: var(--text-bright);
		font-size: var(--type-sm);
		font-weight: 500;
	}
	.sub {
		display: block;
		color: var(--text-muted);
		font-size: var(--type-xs);
		line-height: 1.4;
	}
</style>
