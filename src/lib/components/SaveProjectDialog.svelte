<script lang="ts">
	// Saves a project to one file (#213): what the file holds and what it never holds, its size,
	// one named choice to add the original files, then where it goes, its progress, and where it
	// is once written. Rust decides what goes in; this only shows it.
	import { onMount } from 'svelte';
	import type { ProjectFilePlan } from '#lib/bindings/ProjectFilePlan.js';
	import type { ProjectFileSaved } from '#lib/bindings/ProjectFileSaved.js';
	import { Action } from '#lib/action.svelte.js';
	import { askForFolders } from '#lib/dialogs.js';
	import { messages } from '#lib/i18n/index.js';
	import { forSystem } from '#lib/platform.js';
	import { pickProjectFileSave } from '#lib/projectFile.js';
	import { api, listenWhileMounted, on, revealInFolder } from '#lib/tauri.js';
	import Dialog from './Dialog.svelte';
	import Icon from './Icon.svelte';
	import ProgressBar from './ProgressBar.svelte';

	let { projectId, onclose }: { projectId: string; onclose: () => void } = $props();
	const m = messages();
	const t = m.projectFile;

	let plan = $state<ProjectFilePlan | null>(null);
	let withOriginals = $state(false);
	let progress = $state<{ done: number; total: number } | null>(null);
	let saved = $state<ProjectFileSaved | null>(null);
	const action = new Action();

	const size = $derived(plan ? plan.results_bytes + (withOriginals ? plan.originals_bytes : 0) : 0);

	onMount(() => {
		void action.run(
			() => api.planProjectFile(projectId),
			(found) => (plan = found)
		);
		return listenWhileMounted(() =>
			on.projectFileProgress((p) => {
				if (action.busy) progress = p;
			})
		);
	});

	async function save() {
		if (!plan) return;
		const path = await pickProjectFileSave(plan.file_name);
		if (!path) return;
		progress = { done: 0, total: 0 };
		await action.run(
			async () => {
				// The file is written beside itself first: in the Mac App Store's build, its folder
				// is asked for.
				if (!(await askForFolders([path]))) return null;
				return api.saveProjectFile(projectId, path, withOriginals);
			},
			(done) => {
				if (done) saved = done;
			}
		);
		progress = null;
	}
</script>

<Dialog labelledby="project-file-title" size="medium" onescape={action.busy ? undefined : onclose}>
	<h2 id="project-file-title">{t.title}</h2>
	<p class="muted">{t.lede}</p>

	{#if action.error}<p class="chip danger" role="alert">{action.error}</p>{/if}

	{#if saved}
		<p class="done" role="status">{t.saved(saved.path)}</p>
		{#if saved.originals_left > 0}<p class="muted">{t.leftOut(saved.originals_left)}</p>{/if}
		<div class="actions">
			<button class="btn ghost" onclick={() => saved && void revealInFolder(saved.path)}>
				<Icon name="folder" size={14} />{forSystem(t.show)}
			</button>
			<button class="btn primary" onclick={onclose}>{t.close}</button>
		</div>
	{:else if !plan}
		{#if !action.error}<p class="muted" role="status">{t.reading}</p>{/if}
		<div class="actions">
			<button class="btn ghost" onclick={onclose}>{t.cancel}</button>
		</div>
	{:else}
		<section aria-labelledby="project-file-in">
			<h3 id="project-file-in">{t.inFile}</h3>
			<ul>
				<li>{t.documents(plan.documents, plan.jobs)}</li>
				<li>{t.yourWork}</li>
			</ul>
			<p class="muted">{t.never}</p>
		</section>

		{#if plan.originals > 0}
			<label class="choice">
				<input type="checkbox" bind:checked={withOriginals} disabled={action.busy} />
				<span>
					<span class="label">{t.originals}</span>
					<span class="muted hint">
						{t.originalsHint(plan.originals, t.bytes(plan.originals_bytes))}
					</span>
				</span>
			</label>
		{:else}
			<p class="muted">{t.noOriginals}</p>
		{/if}
		{#if plan.originals_missing > 0}
			<p class="muted">{t.originalsMissing(plan.originals_missing)}</p>
		{/if}
		<p class="size num">{t.size(t.bytes(size))}</p>

		{#if progress}
			<div class="progress" role="status">
				<span>{t.saving}</span>
				<ProgressBar done={progress.done} total={progress.total || 1} label={t.saving} />
			</div>
		{/if}

		<div class="actions">
			<button class="btn ghost" onclick={onclose} disabled={action.busy}>{t.cancel}</button>
			<button class="btn primary" onclick={save} disabled={action.busy}>
				<Icon name="download" size={14} />{t.saveButton}
			</button>
		</div>
	{/if}
</Dialog>

<style>
	h3 {
		font-size: var(--type-sm);
		margin: var(--space-3) 0 var(--space-1);
	}
	ul {
		margin: 0 0 var(--space-2);
		padding-left: var(--space-4);
	}
	.choice {
		display: flex;
		gap: var(--space-2);
		align-items: flex-start;
		margin-top: var(--space-3);
	}
	.choice input {
		margin-top: 0.2rem;
	}
	.choice .label {
		display: block;
		font-weight: 600;
	}
	.hint {
		display: block;
		font-size: var(--type-sm);
	}
	.size {
		margin-top: var(--space-3);
	}
	.progress {
		display: grid;
		gap: var(--space-1);
		margin-top: var(--space-3);
	}
	.done {
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
		margin-top: var(--space-4);
	}
</style>
