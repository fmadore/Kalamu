<script lang="ts">
	// The language data not downloaded yet, found by typing its name rather than scrolled
	// through: a search box over the list, each language with its size. Choosing one downloads
	// it. Its keys follow the job's language picker: arrows move, Enter takes the first found.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { searchKey } from '#lib/languages.js';

	let {
		items,
		disabled = false,
		onpick
	}: {
		/** What can be downloaded, in the order shown. */
		items: { id: string; name: string; bytes: number }[];
		/** While another download runs. */
		disabled?: boolean;
		onpick: (id: string) => void;
	} = $props();

	const m = messages();
	const listId = $props.id();

	let query = $state('');
	let open = $state(false);
	let box = $state<HTMLInputElement>();
	let list = $state<HTMLElement>();

	const found = $derived.by(() => {
		const q = searchKey(query.trim());
		return q === '' ? items : items.filter((i) => searchKey(i.name).includes(q));
	});

	// The box sits at the foot of Settings: the list opened below it is scrolled into view.
	$effect(() => {
		if (open && list) list.scrollIntoView({ block: 'nearest' });
	});

	function pick(id: string) {
		query = '';
		open = false;
		onpick(id);
	}

	function onListKey(e: KeyboardEvent) {
		const buttons = [...(list?.querySelectorAll<HTMLButtonElement>('button') ?? [])];
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
			queueMicrotask(() => list?.querySelector<HTMLButtonElement>('button')?.focus());
		} else if (e.key === 'Enter' && found.length > 0 && !disabled) {
			e.preventDefault();
			pick(found[0].id);
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

<div class="find" onfocusout={onFocusOut}>
	<div class="box">
		<Icon name="search" size={16} />
		<input
			bind:this={box}
			bind:value={query}
			type="search"
			placeholder={m.settings.findLanguageData}
			aria-label={m.settings.findLanguageData}
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
		<div class="list" id={listId} bind:this={list} role="group" onkeydown={onListKey}>
			{#if disabled}
				<p class="muted note">{m.settings.oneDownload}</p>
			{/if}
			{#if found.length === 0}
				<p class="muted note">{m.newJob.noLanguageFound}</p>
			{:else}
				<ul>
					{#each found as item (item.id)}
						<li>
							<button {disabled} onclick={() => pick(item.id)}>
								<span class="name">{item.name}</span>
								<span class="size num">{m.settings.size(item.bytes)}</span>
								<Icon name="download" size={14} />
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}
</div>

<style>
	.find {
		position: relative;
		max-width: 26rem;
	}
	.box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding-left: var(--space-3);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		background: var(--surface-2);
		color: var(--text-muted);
	}
	.box:hover {
		border-color: var(--line-hover);
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
		overflow: hidden auto;
		padding: var(--space-1) 0;
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
		box-shadow: var(--shadow-pop);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	button {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		padding: var(--space-2) var(--space-3);
		border: 0;
		background: none;
		cursor: pointer;
		color: var(--text-body);
		font-size: var(--type-sm);
		text-align: left;
	}
	button:hover:not(:disabled),
	button:focus-visible {
		background: var(--wash);
		color: var(--text-bright);
	}
	button:disabled {
		color: var(--text-faint);
		cursor: default;
	}
	.name {
		flex: 1;
		min-width: 0;
	}
	.size {
		color: var(--text-muted);
		font-size: var(--type-xs);
	}
	.note {
		margin: var(--space-2) var(--space-3);
		font-size: var(--type-xs);
	}
</style>
