<script lang="ts" module>
	import type Icon from './Icon.svelte';
	import type { ComponentProps } from 'svelte';

	/** One thing the menu offers. */
	export interface Action {
		label: string;
		icon?: ComponentProps<typeof Icon>['name'];
		run: () => void;
	}
</script>

<script lang="ts">
	// A menu of actions opened from a right click or the menu key, over a part of the review that
	// has things to offer (the text compared, a region of the scan): each action closes it, then
	// runs. `ContextMenu` places it, moves the keyboard round it and gives the focus back.
	import ContextMenu from './ContextMenu.svelte';
	import IconGlyph from './Icon.svelte';

	let {
		actions,
		x,
		y,
		label,
		width = '17rem',
		onclose
	}: {
		actions: Action[];
		/** Where it opens, in the window. */
		x: number;
		y: number;
		/** Its accessible name. */
		label: string;
		width?: string;
		onclose: () => void;
	} = $props();

	let menu = $state<ContextMenu | null>(null);
</script>

<ContextMenu bind:this={menu} {x} {y} {label} {width} {onclose}>
	{#each actions as action (action.label)}
		<button
			type="button"
			role="menuitem"
			class="item"
			onclick={() => {
				menu?.close();
				action.run();
			}}
			>{#if action.icon}<IconGlyph name={action.icon} size={14} />{/if}{action.label}</button
		>
	{/each}
</ContextMenu>

<style>
	.item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		min-height: 2rem;
		padding: 0 var(--space-2);
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-bright);
		font: inherit;
		text-align: left;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
	}
	.item:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
	}
</style>
