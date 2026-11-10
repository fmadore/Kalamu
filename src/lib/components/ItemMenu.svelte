<script lang="ts">
	// What can be done to one name an analysis found, opened from the name (a click, Enter, a
	// right click or the menu key): rename it or remove it, straight away, without opening the
	// editor. At the pointer or under the name, kept inside the window; closed by Escape, a click
	// elsewhere or a choice, the focus going back to the name. Renaming turns the menu into a
	// field holding the name.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { giveFocusBack } from '#lib/keys.js';

	let {
		value,
		x,
		y,
		onrename,
		onremove,
		onclose
	}: {
		/** The name, as shown. */
		value: string;
		/** Where it opens, in the window. */
		x: number;
		y: number;
		onrename: (to: string) => void;
		onremove: () => void;
		onclose: () => void;
	} = $props();
	const m = messages();

	let box = $state<HTMLElement | null>(null);
	/** What had the focus when the menu opened, given it back when the menu closes. */
	const opener = document.activeElement;
	let renaming = $state(false);
	// svelte-ignore state_referenced_locally
	let draft = $state(value);
	let field = $state<HTMLInputElement | null>(null);

	function place() {
		if (!box) return;
		const { offsetWidth: w, offsetHeight: h } = box;
		box.style.left = `${Math.max(8, Math.min(x, innerWidth - w - 8))}px`;
		box.style.top = `${Math.max(8, Math.min(y, innerHeight - h - 8))}px`;
	}

	$effect(() => {
		if (!box) return;
		box.showPopover();
		place();
		items()[0]?.focus();
	});

	$effect(() => {
		if (!renaming || !field) return;
		place();
		field.focus();
		field.select();
	});

	const items = () =>
		box ? [...box.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')] : [];

	function keys(e: KeyboardEvent) {
		if (renaming) return;
		const all = items();
		const at = all.indexOf(document.activeElement as HTMLButtonElement);
		const to =
			e.key === 'ArrowDown'
				? (at + 1) % all.length
				: e.key === 'ArrowUp'
					? (at - 1 + all.length) % all.length
					: -1;
		if (to < 0) return;
		e.preventDefault();
		all[to]?.focus();
	}

	function close() {
		box?.hidePopover();
	}

	function submit(e: SubmitEvent) {
		e.preventDefault();
		const to = draft.trim();
		close();
		if (to && to !== value) onrename(to);
	}
</script>

<div
	bind:this={box}
	class="menu"
	popover="auto"
	role={renaming ? 'dialog' : 'menu'}
	tabindex="-1"
	aria-label={m.review.itemMenu(value)}
	onkeydown={keys}
	ontoggle={(e) => {
		if (e.newState !== 'closed') return;
		giveFocusBack(opener, box);
		onclose();
	}}
>
	{#if renaming}
		<form onsubmit={submit}>
			<label class="field">
				<span>{m.review.renameItem}</span>
				<input bind:this={field} bind:value={draft} type="text" spellcheck="false" />
			</label>
			<p class="hint">{m.review.itemEverywhere}</p>
			<div class="actions">
				<button type="button" class="btn ghost sm" onclick={close}>{m.common.cancel}</button>
				<button type="submit" class="btn primary sm" disabled={!draft.trim()}
					>{m.common.save}</button
				>
			</div>
		</form>
	{:else}
		<button type="button" role="menuitem" class="item" onclick={() => (renaming = true)}
			><Icon name="edit" size={14} />{m.review.renameItem}</button
		>
		<button
			type="button"
			role="menuitem"
			class="item danger"
			onclick={() => {
				close();
				onremove();
			}}><Icon name="close" size={14} />{m.review.removeItem}</button
		>
	{/if}
</div>

<style>
	/* In the top layer, placed at the pointer by the effects above. */
	.menu {
		position: fixed;
		inset: auto;
		margin: 0;
		width: 17rem;
		padding: var(--space-1);
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	.menu:popover-open {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 1px;
	}
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
		cursor: pointer;
	}
	.item:hover {
		background: var(--wash);
	}
	/* The keyboard's place, told apart from the pointer's: the app's focus ring, inside the row. */
	.item:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
	}
	.danger {
		color: var(--warn-text);
	}
	form {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-2);
	}
	.hint {
		margin: 0;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
