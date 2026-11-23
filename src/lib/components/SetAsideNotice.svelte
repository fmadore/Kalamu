<script module lang="ts">
	import type { Refused } from '#lib/bindings/Refused.js';
	import type { Mode } from '#lib/newJob.js';

	/**
	 * What the last files added gave that the task does not read, the tasks that read it, and
	 * the files to move there; on YouTube, the recordings and videos dropped.
	 */
	export interface SetAside {
		refused: Refused[];
		readers: Mode[];
		move: string[];
		youtube: boolean;
	}
</script>

<script lang="ts">
	// New job's notice beside the files: what the files just added gave that the task does not
	// read, and the tasks that read it, to move them there.
	import Icon from './Icon.svelte';
	import { listNames, messages } from '#lib/i18n/index.js';
	import { taskOf } from '#lib/newJob.js';

	let {
		aside,
		mode,
		onswitch,
		ondismiss
	}: {
		aside: SetAside;
		/** The task the files were added to. */
		mode: Mode;
		/** Moves the files set aside to a task that reads them. */
		onswitch: (reader: Mode) => void;
		ondismiss: () => void;
	} = $props();
	const m = messages();

	const taskName = $derived(m.common.quoted(m.task[mode]));
	const set = m.newJob.setAside;
	const refusedFiles = $derived(aside.refused.filter((r) => !r.folder));
	const folders = $derived(aside.refused.filter((r) => r.folder));
</script>

<div class="notice set-aside" role="status">
	<div class="set-aside-text">
		{#if aside.youtube}
			<p>{set.youtube}</p>
		{/if}
		{#if refusedFiles.length > 0}
			<p>
				{set.files(listNames(refusedFiles.map((r) => r.file_name)), taskName, refusedFiles.length)}
			</p>
		{/if}
		{#if folders.length > 0}
			<p>{set.folders(listNames(folders.map((r) => r.file_name)), taskName)}</p>
		{/if}
		{#if refusedFiles.some((r) => r.kind === null)}
			<p class="set-aside-kinds">{set.kinds}</p>
		{/if}
	</div>
	<div class="set-aside-actions">
		{#each aside.readers as reader (reader)}
			<button class="btn sm" title={set.insteadHint} onclick={() => onswitch(reader)}>
				<Icon name={taskOf(reader)} size={14} />{set.instead(m.task[reader])}
			</button>
		{/each}
		<button
			class="btn ghost sm icon"
			aria-label={set.dismiss}
			title={set.dismiss}
			onclick={ondismiss}
		>
			<Icon name="close" size={14} />
		</button>
	</div>
</div>

<style>
	/* Files added that the task does not read, beside the files: what and where else. */
	.set-aside {
		justify-content: space-between;
		align-items: flex-start;
	}
	.set-aside-text {
		display: grid;
		gap: var(--space-1);
		flex: 1 1 18rem;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.set-aside-kinds {
		color: var(--text-secondary);
	}
	.set-aside-actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
</style>
