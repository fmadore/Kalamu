<script lang="ts">
	// A menu opened at the pointer or under the button that opened it, kept inside the window, in
	// the top layer: the job list's (`JobMenu`) and an analysis name's (`ItemMenu`). The arrow
	// keys move round its items, passing over a disabled one, the focus starting on the first.
	// Escape, a click elsewhere or a choice closes it, and the focus goes back to what had it
	// unless the choice took it elsewhere. Each menu gives its own items, labels and width.
	import type { Snippet } from 'svelte';
	import { giveFocusBack } from '#lib/keys.js';
	import { menuTarget } from '#lib/menu.js';

	let {
		x,
		y,
		label,
		role = 'menu',
		width,
		moves = true,
		homeEnd = true,
		onclose,
		children
	}: {
		/** Where it opens, in the window. */
		x: number;
		y: number;
		/** Its accessible name. */
		label: string;
		/** `menu`, or `dialog` while it holds a form instead of its items. */
		role?: 'menu' | 'dialog';
		/** Its width, as CSS. */
		width: string;
		/** Whether the arrow keys move between the items (not while it holds a form). */
		moves?: boolean;
		/** Whether Home and End go to the first and last item. */
		homeEnd?: boolean;
		onclose: () => void;
		children: Snippet;
	} = $props();

	let box = $state<HTMLElement | null>(null);
	/** What had the focus when the menu opened, given it back when the menu closes. */
	const opener = document.activeElement;

	/** Places it at the point it opened from, kept inside the window, as large as it now is. */
	export function place() {
		if (!box) return;
		const { offsetWidth: w, offsetHeight: h } = box;
		box.style.left = `${Math.max(8, Math.min(x, innerWidth - w - 8))}px`;
		box.style.top = `${Math.max(8, Math.min(y, innerHeight - h - 8))}px`;
	}

	export function close() {
		box?.hidePopover();
	}

	$effect(() => {
		if (!box) return;
		box.showPopover();
		place();
		items()[0]?.focus();
	});

	const items = () =>
		box ? [...box.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')] : [];

	function keys(e: KeyboardEvent) {
		if (!moves) return;
		const all = items();
		const at = all.indexOf(document.activeElement as HTMLButtonElement);
		const to = menuTarget(all.length, at, e.key, homeEnd);
		if (to < 0) return;
		e.preventDefault();
		all[to]?.focus();
	}
</script>

<div
	bind:this={box}
	class="menu"
	style:width
	popover="auto"
	{role}
	tabindex="-1"
	aria-label={label}
	onkeydown={keys}
	ontoggle={(e) => {
		if (e.newState !== 'closed') return;
		giveFocusBack(opener, box);
		onclose();
	}}
>
	{@render children()}
</div>

<style>
	/* In the top layer, placed at the pointer by `place`. */
	.menu {
		position: fixed;
		inset: auto;
		margin: 0;
		padding: var(--space-1);
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	/* One column no wider than the menu: a long label is cut short, never scrolled. */
	.menu:popover-open {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 1px;
	}
</style>
