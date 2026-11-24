<script lang="ts">
	// Checks a project file without opening it (#246): its progress, then what the check found,
	// a short verdict first. A file Kalamu cannot read at all is said as opening would say it.
	import { onMount } from 'svelte';
	import type { ProjectFileCheck } from '#lib/bindings/ProjectFileCheck.js';
	import { Action } from '#lib/action.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, listenWhileMounted, on } from '#lib/tauri.js';
	import Dialog from './Dialog.svelte';
	import ProgressBar from './ProgressBar.svelte';
	import ProjectFileReport from './ProjectFileReport.svelte';

	let { path, onclose }: { path: string; onclose: () => void } = $props();
	const t = messages().projectFile;

	let progress = $state<{ done: number; total: number }>({ done: 0, total: 0 });
	let check = $state<ProjectFileCheck | null>(null);
	const action = new Action();

	onMount(() => {
		void action.run(
			() => api.checkProjectFile(path),
			(found) => (check = found)
		);
		return listenWhileMounted(() =>
			on.projectFileProgress((p) => {
				if (action.busy) progress = p;
			})
		);
	});
</script>

<Dialog labelledby="check-file-title" size="medium" onescape={action.busy ? undefined : onclose}>
	<h2 id="check-file-title">{t.checkTitle}</h2>
	<p class="muted">{t.checkLede}</p>
	<p class="path">{path}</p>

	{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}

	{#if check}
		<div role="status"><ProjectFileReport {check} /></div>
	{:else if action.busy}
		<div class="progress" role="status">
			<span>{t.checking}</span>
			<ProgressBar done={progress.done} total={progress.total || 1} label={t.checking} />
		</div>
	{/if}

	<div class="actions">
		<button class="btn primary" onclick={onclose} disabled={action.busy}>{t.close}</button>
	</div>
</Dialog>

<style>
	.path {
		font-size: var(--type-sm);
		overflow-wrap: anywhere;
	}
	.progress {
		display: grid;
		gap: var(--space-1);
		margin-top: var(--space-3);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
		margin-top: var(--space-4);
	}
</style>
