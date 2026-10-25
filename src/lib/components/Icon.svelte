<script lang="ts">
	// The few icons the interface draws, on a 16-unit grid in the current text colour. Each is
	// decorative: the control that holds it carries the accessible name.
	type Name = 'back' | 'chevron' | 'close' | 'pause' | 'play' | 'plus' | 'zoom-in' | 'zoom-out';
	let { name, size = 16 }: { name: Name; size?: number } = $props();
</script>

<svg
	class="icon"
	width={size}
	height={size}
	viewBox="0 0 16 16"
	fill="none"
	stroke="currentColor"
	stroke-width="1.6"
	stroke-linecap="round"
	stroke-linejoin="round"
	aria-hidden="true"
>
	{#if name === 'zoom-in' || name === 'zoom-out'}
		<circle cx="7" cy="7" r="4.75" />
		<path d="M10.5 10.5 14 14M4.75 7h4.5" />
		{#if name === 'zoom-in'}<path d="M7 4.75v4.5" />{/if}
	{:else if name === 'plus'}
		<path d="M8 3v10M3 8h10" />
	{:else if name === 'close'}
		<path d="M4.5 4.5l7 7M11.5 4.5l-7 7" />
	{:else if name === 'back'}
		<path d="M13 8H3.5M7.5 4 3.5 8l4 4" />
	{:else if name === 'chevron'}
		<path d="M6 3.5 10.5 8 6 12.5" />
	{:else if name === 'play'}
		<path d="M5 3v10l8-5z" fill="currentColor" />
	{:else if name === 'pause'}
		<path d="M4.5 3h2.25v10H4.5zM9.25 3h2.25v10H9.25z" fill="currentColor" stroke="none" />
	{/if}
</svg>

<style>
	.icon {
		display: block;
		flex: none;
	}
</style>
