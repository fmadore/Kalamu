<script lang="ts">
	// A search box over a list that opens under it as the box takes the focus or is typed in.
	// ↓ in the box goes to the list's first item, ↓ and ↑ move through the items, ↑ from the
	// first goes back to the box; Escape closes the list (from the list, the focus goes back to
	// the box) and so does the focus leaving both. Each picker (a job's languages, the language
	// data in Settings) gives its own list, labels and what Enter in the box takes.
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		query = $bindable(),
		open = $bindable(),
		label,
		placeholder,
		items,
		width,
		clipX = false,
		reveal = false,
		onenter,
		children
	}: {
		query: string;
		open: boolean;
		/** The box's accessible name. */
		label: string;
		placeholder: string;
		/** Which buttons in the list the arrow keys move between. */
		items: string;
		/** The widest the box and its list grow, as CSS. */
		width: string;
		/** Whether the list cuts off what is wider than it, rather than scrolling across. */
		clipX?: boolean;
		/** Whether the list, once open, is scrolled into view. */
		reveal?: boolean;
		/** Enter in the box: takes an item and says so, or returns false to leave the key alone. */
		onenter: () => boolean;
		children: Snippet;
	} = $props();

	const listId = $props.id();
	let box = $state<HTMLInputElement>();
	let list = $state<HTMLElement>();

	/** Puts the focus back in the box. */
	export function focus() {
		box?.focus();
	}

	// A box low on its screen has the list opened below it scrolled into view.
	$effect(() => {
		if (reveal && open && list) list.scrollIntoView({ block: 'nearest' });
	});

	function onListKey(e: KeyboardEvent) {
		const buttons = [...(list?.querySelectorAll<HTMLButtonElement>(items) ?? [])];
		const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			const next = at + (e.key === 'ArrowDown' ? 1 : -1);
			if (next < 0) box?.focus();
			else buttons[Math.min(next, buttons.length - 1)]?.focus();
		} else if (e.key === 'Escape') {
			open = false;
			box?.focus();
		}
	}

	function onBoxKey(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			open = true;
			queueMicrotask(() => list?.querySelector<HTMLButtonElement>(items)?.focus());
		} else if (e.key === 'Enter') {
			if (onenter()) e.preventDefault();
		} else if (e.key === 'Escape') {
			open = false;
		}
	}

	/** Closes the list when focus leaves the picker, not when it moves inside it. */
	function onFocusOut(e: FocusEvent) {
		const to = e.relatedTarget as Node | null;
		if (!to || !(e.currentTarget as HTMLElement).contains(to)) open = false;
	}
</script>

<div class="find" style:max-width={width} onfocusout={onFocusOut}>
	<div class="box">
		<Icon name="search" size={16} />
		<input
			bind:this={box}
			bind:value={query}
			type="search"
			{placeholder}
			aria-label={label}
			aria-controls={listId}
			autocomplete="off"
			spellcheck="false"
			onfocus={() => (open = true)}
			oninput={() => (open = true)}
			onkeydown={onBoxKey}
		/>
	</div>
	{#if open}
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<div
			class="list"
			class:clip={clipX}
			id={listId}
			bind:this={list}
			role="group"
			onkeydown={onListKey}
		>
			{@render children()}
		</div>
	{/if}
</div>

<style>
	.find {
		position: relative;
	}
	.box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding-left: var(--space-3);
		border: 1px solid var(--control-line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
		color: var(--text-muted);
	}
	.box:hover {
		border-color: var(--control-line-hover);
	}
	.box:focus-within {
		border-color: var(--text-bright);
		box-shadow: 0 0 0 3px var(--wash);
	}
	.box input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: none;
		outline: none;
		box-shadow: none;
	}
	.list {
		position: absolute;
		z-index: 2;
		top: calc(100% + var(--space-1));
		left: 0;
		right: 0;
		max-height: 18rem;
		overflow-y: auto;
		padding: var(--space-1) 0;
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
		box-shadow: var(--shadow-pop);
	}
	.list.clip {
		overflow: hidden auto;
	}
</style>
