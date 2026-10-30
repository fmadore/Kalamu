<script lang="ts" generics="T extends string">
	// A joined choice: its options side by side in one bordered strip, the chosen one lifted with
	// an ink ring, each named and explained in a line, a price beside the name where it has one.
	// New job chooses its line breaks and its pace with it.
	let {
		options,
		value = $bindable()
	}: {
		/** The options in order: their value, name, one-line explanation and, if any, price. */
		options: { value: T; label: string; hint: string; price?: string | null }[];
		value: T;
	} = $props();
</script>

<div class="choices">
	{#each options as option (option.value)}
		<label class="choice" class:selected={value === option.value}>
			<input type="radio" bind:group={value} value={option.value} />
			<span>
				<strong
					>{option.label}{#if option.price}<span class="mono price">{option.price}</span
						>{/if}</strong
				>
				<span class="hint">{option.hint}</span>
			</span>
		</label>
	{/each}
</div>

<style>
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	.choices {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
	}
	.choice {
		display: flex;
		gap: var(--space-3);
		align-items: start;
		padding: var(--space-3) var(--space-4);
		border-left: 1px solid var(--line);
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.choice:first-child {
		border-left: 0;
		border-radius: var(--radius-card) 0 0 var(--radius-card);
	}
	.choice:last-child {
		border-radius: 0 var(--radius-card) var(--radius-card) 0;
	}
	.choice:hover {
		background: var(--surface-1);
	}
	.choice.selected {
		background: var(--surface-2);
		box-shadow: inset 0 0 0 1.5px var(--ink);
	}
	.choice input {
		margin-top: 0.25em;
	}
	.choice .price {
		margin-left: var(--space-2);
		font-weight: 400;
		color: var(--text-muted);
	}
	.choice > span {
		display: grid;
		gap: var(--space-1);
	}
	.choice strong {
		font-size: var(--type-sm);
		font-weight: 500;
		color: var(--text-bright);
	}
	@media (max-width: 760px) {
		.choices {
			grid-template-columns: 1fr;
		}
		.choice {
			border-left: 0;
			border-top: 1px solid var(--line);
			border-radius: 0;
		}
	}
</style>
