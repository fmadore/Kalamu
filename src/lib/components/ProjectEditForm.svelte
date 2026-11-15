<script lang="ts">
	// A project's name, description and the description its documents start with, edited under
	// its masthead. The project's own screen fills the drafts when editing starts and saves them.
	import DescriptionFields from './DescriptionFields.svelte';
	import type { Draft } from '#lib/described.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		name = $bindable(),
		description = $bindable(),
		defaults = $bindable(),
		onsubmit,
		oncancel
	}: {
		name: string;
		description: string;
		/** What each document added to the project is described with to begin with. */
		defaults: Draft;
		onsubmit: (e: SubmitEvent) => void;
		oncancel: () => void;
	} = $props();
	const m = messages();
</script>

<form class="edit" {onsubmit}>
	<label class="field">
		<span>{m.projects.name}</span>
		<input type="text" bind:value={name} required />
	</label>
	<label class="field">
		<span>{m.projects.description}</span>
		<textarea rows="2" bind:value={description} placeholder={m.projects.descriptionHint}></textarea>
	</label>
	<fieldset class="defaults">
		<legend>{m.projects.describe.projectTitle}</legend>
		<p class="muted hint">{m.projects.describe.projectLede}</p>
		<DescriptionFields bind:value={defaults} />
	</fieldset>
	<div class="row">
		<button type="button" class="btn ghost" onclick={oncancel}>{m.projects.cancel}</button>
		<button class="btn primary" disabled={!name.trim()}>{m.projects.save}</button>
	</div>
</form>

<style>
	.edit {
		display: grid;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--line);
	}
	.defaults {
		display: grid;
		gap: var(--space-2);
		margin: 0;
		padding: var(--space-3) 0 0;
		border: 0;
		border-top: 1px solid var(--line-soft);
	}
	.defaults legend {
		padding: 0;
		font-size: var(--type-sm);
		font-weight: 500;
		color: var(--text-bright);
	}
	.defaults .hint {
		font-size: var(--type-xs);
	}
	.row {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
