<script lang="ts">
	// How the model fields list models: by provider or by tier. Two buttons joined in one strip,
	// the one in use pressed in, as a view that is on reads elsewhere.
	import { messages } from '#lib/i18n/index.js';
	import type { ModelsBy } from '#lib/tiers.js';

	let { by = $bindable() }: { by: ModelsBy } = $props();
	const m = messages();
	const options: ModelsBy[] = ['provider', 'tier'];
</script>

<div class="by" role="group" aria-label={m.newJob.tiers.listBy}>
	<span class="label small" aria-hidden="true">{m.newJob.tiers.listBy}</span>
	<div class="strip">
		{#each options as option (option)}
			<button
				type="button"
				class="btn sm"
				aria-pressed={by === option}
				onclick={() => (by = option)}>{m.newJob.tiers.by[option]}</button
			>
		{/each}
	</div>
</div>

<style>
	.by {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}
	.small {
		font-size: var(--type-xs);
	}
	.strip {
		display: inline-flex;
	}
	.strip .btn {
		border-radius: 0;
	}
	.strip .btn:first-child {
		border-radius: var(--radius-control) 0 0 var(--radius-control);
	}
	.strip .btn:last-child {
		border-radius: 0 var(--radius-control) var(--radius-control) 0;
	}
	.strip .btn + .btn {
		margin-left: -1px;
	}
	/* A view that is on reads as pressed in, not as an action. */
	.strip .btn[aria-pressed='true'] {
		position: relative;
		background: var(--wash);
		border-color: var(--control-line-hover);
	}
</style>
