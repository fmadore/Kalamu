<script lang="ts">
	// The provider, model and prompt a review's request goes to, as the passage and region dialogs
	// offer them. Another provider brings its usual model; `defaultPrompt` names a first choice
	// that leaves the prompt to the engine; `focus` takes the focus to the first picker, for
	// pickers opened from the keyboard.
	import { providerName as providerNameOf } from '#lib/catalogText.js';
	import { messages } from '#lib/i18n/index.js';
	import type { ModelChoice } from '#lib/modelAsk.svelte.js';

	let {
		choice,
		defaultPrompt,
		focus = false
	}: { choice: ModelChoice; defaultPrompt?: string; focus?: boolean } = $props();
	const m = messages();

	let first = $state<HTMLSelectElement | null>(null);
	$effect(() => {
		if (focus && first) first.focus();
	});
</script>

<div class="pickers">
	<label class="field">
		<span>{m.newJob.provider}</span>
		<select
			bind:this={first}
			bind:value={choice.provider}
			onchange={() => choice.providerChanged()}
		>
			{#each choice.providers as pr (pr.id)}
				<option value={pr.id}>{providerNameOf(pr)}</option>
			{/each}
		</select>
	</label>
	<label class="field">
		<span>{m.newJob.model}</span>
		<select bind:value={choice.model}>
			{#each choice.models as md (md.id)}
				<option value={md.id}>{md.name}</option>
			{/each}
		</select>
	</label>
	<label class="field wide">
		<span>{m.newJob.prompt}</span>
		<select bind:value={choice.preset}>
			{#if defaultPrompt !== undefined}<option value="">{defaultPrompt}</option>{/if}
			{#each choice.presets as pr (pr.id)}
				<option value={pr.id}>{pr.name}</option>
			{/each}
		</select>
	</label>
</div>

<style>
	.pickers {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-3);
	}
	.pickers .wide {
		grid-column: 1 / -1;
	}
	.pickers select {
		width: 100%;
	}
</style>
