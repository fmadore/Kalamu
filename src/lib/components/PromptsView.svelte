<script lang="ts">
	import { onMount } from 'svelte';
	import type { Preset } from '#lib/bindings/Preset.js';
	import type { Task } from '#lib/bindings/Task.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, pickPresetFile, pickSaveFile } from '#lib/tauri.js';
	import { toast } from '#lib/state.svelte.js';

	const m = messages();

	let presets = $state<Preset[]>([]);
	let selectedId = $state<string | null>(null);
	let draft = $state<Preset | null>(null);
	let error = $state<string | null>(null);

	const selected = $derived(presets.find((p) => p.id === selectedId) ?? null);
	const dirty = $derived(
		!!draft &&
			!!selected &&
			(draft.name !== selected.name ||
				draft.description !== selected.description ||
				draft.instructions !== selected.instructions ||
				draft.user_prompt !== selected.user_prompt)
	);

	async function load(keep?: string) {
		presets = await api.listPresets();
		select(keep ?? selectedId ?? presets[0]?.id ?? null);
	}

	function select(id: string | null) {
		selectedId = id;
		const p = presets.find((x) => x.id === id);
		draft = p ? { ...p } : null;
	}

	async function run(action: () => Promise<string | undefined | void>) {
		error = null;
		try {
			const keep = await action();
			await load(keep ?? undefined);
		} catch (e) {
			error = describeError(e, m);
		}
	}

	const tasks: Task[] = ['text_extraction', 'audio_transcription'];

	onMount(() => {
		void load();
	});
</script>

<div class="layout">
	<section class="list" aria-labelledby="prompts-title">
		<h1 id="prompts-title">{m.prompts.title}</h1>
		<p class="muted small">{m.prompts.hint}</p>
		<div class="row">
			<button
				class="btn sm"
				onclick={() =>
					run(async () => {
						const path = await pickPresetFile();
						if (!path) return;
						const imported = await api.importPresets(path);
						toast(m.prompts.imported(imported.length));
						return imported[0]?.id;
					})}>{m.prompts.import}</button
			>
			<button
				class="btn sm"
				disabled={!selected}
				onclick={() =>
					run(async () => {
						if (!selected) return;
						const path = await pickSaveFile(`${selected.name}.json`);
						if (path) await api.exportPresets([selected.id], path);
					})}>{m.prompts.export}</button
			>
		</div>
		{#each tasks as task (task)}
			<h2 class="label group">{m.task[task]}</h2>
			<ul>
				{#each presets.filter((p) => p.task === task) as p (p.id)}
					<li>
						<button class="item" class:active={p.id === selectedId} onclick={() => select(p.id)}>
							<span>{p.name}</span>
							<span class="chip"
								>{p.modified
									? m.prompts.modified
									: p.builtin
										? m.prompts.builtin
										: m.prompts.own}</span
							>
						</button>
					</li>
				{/each}
			</ul>
		{/each}
	</section>

	{#if draft && selected}
		<section class="editor card" aria-label={draft.name}>
			<label class="field">
				<span>{m.prompts.name}</span>
				<input type="text" bind:value={draft.name} />
			</label>
			<label class="field">
				<span>{m.prompts.description}</span>
				<input type="text" bind:value={draft.description} />
			</label>
			<label class="field grow">
				<span>{m.prompts.instructions}</span>
				<textarea class="mono" bind:value={draft.instructions}></textarea>
			</label>
			<label class="field">
				<span>{m.prompts.userPrompt}</span>
				<textarea class="short mono" bind:value={draft.user_prompt}></textarea>
			</label>
			{#if error}<p class="chip danger">{error}</p>{/if}
			<div class="actions">
				<button
					class="btn primary"
					disabled={!dirty}
					onclick={() => run(async () => (await api.savePreset($state.snapshot(draft)!)).id)}
					>{m.prompts.save}</button
				>
				<button
					class="btn"
					onclick={() =>
						run(
							async () =>
								(await api.duplicatePreset(selected.id, m.prompts.copyName(selected.name))).id
						)}>{m.prompts.duplicate}</button
				>
				{#if selected.modified}
					<button
						class="btn"
						onclick={() =>
							run(async () => {
								await api.deletePreset(selected.id);
								return selected.id;
							})}
					>
						{m.prompts.reset}
					</button>
				{:else if !selected.builtin}
					<button
						class="btn danger"
						onclick={() =>
							run(async () => {
								await api.deletePreset(selected.id);
								selectedId = null;
							})}
					>
						{m.prompts.delete}
					</button>
				{/if}
				{#if dirty}<span class="muted small">{m.prompts.unsaved}</span>{/if}
			</div>
		</section>
	{/if}
</div>

<style>
	.layout {
		display: grid;
		grid-template-columns: minmax(16rem, 22rem) 1fr;
		gap: var(--space-5);
		align-items: start;
		height: calc(100vh - 2 * var(--space-5));
	}
	.list {
		display: grid;
		gap: var(--space-2);
		align-content: start;
		overflow: auto;
		max-height: 100%;
	}
	.small {
		font-size: var(--type-sm);
	}
	.row {
		display: flex;
		gap: var(--space-2);
	}
	.group {
		margin-top: var(--space-3);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 2px;
	}
	.item {
		width: 100%;
		display: flex;
		justify-content: space-between;
		gap: var(--space-2);
		align-items: center;
		text-align: left;
		padding: var(--space-2) var(--space-3);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		cursor: pointer;
	}
	.item:hover {
		background: var(--accent-wash);
	}
	.item.active {
		background: var(--accent-chip);
	}
	.editor {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-5);
		height: 100%;
		min-height: 0;
	}
	.grow {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.grow textarea {
		flex: 1;
	}
	textarea {
		font-size: var(--type-sm);
	}
	.short {
		min-height: 4rem;
	}
	.actions {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
</style>
