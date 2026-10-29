<script lang="ts">
	// One answer being corrected, in the control its kind needs: a menu for one choice, boxes to
	// tick for several, a list one item per line, a text in full (or on one line, for a title or
	// pages). The label is the caller's; the control carries it for screen readers.
	import type { Field } from '#lib/bindings/Field.js';
	import { messages } from '#lib/i18n/index.js';

	let {
		field,
		value = $bindable(),
		line = false
	}: {
		field: Field;
		value: string | string[];
		/** A short text, on one line. */
		line?: boolean;
	} = $props();
	const m = messages();

	const ticked = $derived(Array.isArray(value) ? value : []);
</script>

{#if field.kind.type === 'choice'}
	<select bind:value aria-label={field.label}>
		{#each field.kind.values as v (v)}<option value={v}>{v}</option>{/each}
	</select>
{:else if field.kind.type === 'choices'}
	<div class="checks" role="group" aria-label={field.label}>
		{#each field.kind.values as v (v)}
			<label>
				<input
					type="checkbox"
					checked={ticked.includes(v)}
					onchange={(e) =>
						(value = e.currentTarget.checked ? [...ticked, v] : ticked.filter((x) => x !== v))}
				/>
				{v}
			</label>
		{/each}
	</div>
{:else if line}
	<input type="text" bind:value aria-label={field.label} />
{:else}
	<textarea class:short={field.kind.type === 'list'} bind:value aria-label={field.label}></textarea>
	{#if field.kind.type === 'list'}<p class="hint">{m.review.onePerLine}</p>{/if}
{/if}

<style>
	textarea {
		width: 100%;
		min-height: 7rem;
		font-family: var(--font-text);
		font-size: var(--type-md);
		line-height: 1.5;
	}
	textarea.short {
		min-height: 4.5rem;
	}
	.checks {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-4);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
	}
	.checks label {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		white-space: nowrap;
	}
	.hint {
		margin-top: var(--space-1);
		font-family: var(--font-ui);
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
</style>
