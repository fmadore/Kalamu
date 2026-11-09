<script lang="ts">
	// The parts of a source's description, each a field of its own: the archive, the fonds, the
	// shelfmark, how to cite it. A part the project gives every document shows its value as a
	// hint, so the researcher sees what an empty field stands for.
	import { FIELDS, LONG, type Draft } from '#lib/described.js';
	import { messages } from '#lib/i18n/index.js';

	let { value = $bindable(), inherited = {} }: { value: Draft; inherited?: Draft } = $props();
	const m = messages();
</script>

<div class="fields">
	{#each FIELDS as field (field)}
		{@const hint = inherited[field]}
		<label class="field" class:long={LONG.includes(field)}>
			<span>{m.projects.describe.fields[field]}</span>
			{#if LONG.includes(field)}
				<textarea rows="2" bind:value={value[field]} placeholder={hint ?? ''}></textarea>
			{:else}
				<input
					type="text"
					inputmode={field === 'url' ? 'url' : undefined}
					bind:value={value[field]}
					placeholder={hint ?? ''}
				/>
			{/if}
			{#if hint && !value[field]?.trim()}
				<small class="muted">{m.projects.describe.fromProject}</small>
			{/if}
		</label>
	{/each}
</div>

<style>
	.fields {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
		gap: var(--space-3);
		/* A field with the project's hint under it is taller: its neighbours keep their height. */
		align-items: start;
	}
	.long {
		grid-column: 1 / -1;
	}
	small {
		font-size: var(--type-xs);
	}
</style>
