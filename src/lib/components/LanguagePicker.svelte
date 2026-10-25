<script lang="ts">
	// The languages of a job: chosen ones as chips, then a search box over the offered languages
	// in alphabetical order, favourites first. Starring a language saves it in Settings.
	import Icon from './Icon.svelte';
	import type { LanguageInfo } from '#lib/bindings/LanguageInfo.js';
	import { describeError } from '#lib/errors.js';
	import { languageName, messages } from '#lib/i18n/index.js';
	import { findLanguages, languageLabel } from '#lib/languages.js';
	import { api } from '#lib/tauri.js';
	import { app, toast } from '#lib/state.svelte.js';

	let {
		selected = $bindable(),
		offered,
		single = false,
		unsupported = () => []
	}: {
		selected: string[];
		offered: LanguageInfo[];
		/** One language only (the language of an analysis's answers). */
		single?: boolean;
		/** The chosen models that cannot use a language, by name. */
		unsupported?: (code: string) => string[];
	} = $props();

	const m = messages();
	const listId = $props.id();

	let query = $state('');
	let open = $state(false);
	let box = $state<HTMLInputElement>();
	let list = $state<HTMLElement>();

	const favourites = $derived(app.settings?.favourite_languages ?? []);
	const name = (l: LanguageInfo) => languageName(l.code, l.name);
	const label = (code: string) => (app.catalog ? languageLabel(app.catalog, code) : code);
	const found = $derived(
		findLanguages(
			offered.filter((l) => !selected.includes(l.code)),
			query,
			name,
			m.locale.tag
		)
	);
	const starred = $derived(found.filter((l) => favourites.includes(l.code)));
	const others = $derived(found.filter((l) => !favourites.includes(l.code)));
	/** Favourites not chosen yet, offered as one-click buttons beside the box. */
	const quick = $derived(
		offered.filter((l) => favourites.includes(l.code) && !selected.includes(l.code))
	);

	function add(code: string) {
		selected = single ? [code] : [...selected, code];
		query = '';
		open = false;
		box?.focus();
	}

	async function toggleFavourite(code: string) {
		if (!app.settings) return;
		const next = favourites.includes(code)
			? favourites.filter((c) => c !== code)
			: [...favourites, code];
		app.settings.favourite_languages = next;
		try {
			await api.saveSettings($state.snapshot(app.settings));
		} catch (e) {
			toast(describeError(e, m));
		}
	}

	/** Arrow keys move between the languages, Escape goes back to the box. */
	function onListKey(e: KeyboardEvent) {
		const buttons = [...(list?.querySelectorAll<HTMLButtonElement>('button.pick') ?? [])];
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
			queueMicrotask(() => list?.querySelector<HTMLButtonElement>('button.pick')?.focus());
		} else if (e.key === 'Enter' && found.length > 0) {
			e.preventDefault();
			add((starred[0] ?? others[0]).code);
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

{#snippet row(l: LanguageInfo)}
	{@const fav = favourites.includes(l.code)}
	<li>
		<button class="pick" onclick={() => add(l.code)}>{name(l)}</button>
		<button
			class="star"
			class:on={fav}
			aria-pressed={fav}
			aria-label={fav ? m.newJob.removeFavourite(name(l)) : m.newJob.addFavourite(name(l))}
			onclick={() => toggleFavourite(l.code)}><Icon name="star" size={14} filled={fav} /></button
		>
	</li>
{/snippet}

<div class="picker">
	{#if selected.length > 0}
		<div class="chips">
			{#each selected as code (code)}
				{@const without = unsupported(code)}
				<span class="chip" class:warn={without.length > 0}>
					{label(code)}
					<button
						class="x"
						aria-label="{m.newJob.remove} {label(code)}"
						onclick={() => (selected = selected.filter((c) => c !== code))}
						><Icon name="close" size={12} /></button
					>
				</span>
			{/each}
		</div>
	{/if}

	<div class="find" onfocusout={onFocusOut}>
		<div class="box">
			<Icon name="search" size={16} />
			<input
				bind:this={box}
				bind:value={query}
				type="search"
				placeholder={m.newJob.addLanguage}
				aria-label={m.newJob.findLanguage}
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
				{#if found.length === 0}
					<p class="muted small none">{m.newJob.noLanguageFound}</p>
				{/if}
				{#if starred.length > 0}
					<p class="group">{m.newJob.favouriteLanguages}</p>
					<ul>
						{#each starred as l (l.code)}{@render row(l)}{/each}
					</ul>
				{/if}
				{#if others.length > 0}
					{#if starred.length > 0}<p class="group">{m.newJob.allLanguages}</p>{/if}
					<ul>
						{#each others as l (l.code)}{@render row(l)}{/each}
					</ul>
				{/if}
				{#if favourites.length === 0 && found.length > 0}
					<p class="muted small hint">{m.newJob.favouritesHint}</p>
				{/if}
			</div>
		{/if}
	</div>

	{#if quick.length > 0 && !(single && selected.length > 0)}
		<div class="quick" aria-label={m.newJob.favouriteLanguages} role="group">
			{#each quick as l (l.code)}
				<button class="btn ghost sm" onclick={() => add(l.code)}
					><Icon name="plus" size={14} />{name(l)}</button
				>
			{/each}
		</div>
	{/if}
</div>

<style>
	.picker {
		display: grid;
		gap: var(--space-3);
	}
	.chips,
	.quick {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
	}
	.x {
		display: grid;
		place-items: center;
		border: 0;
		background: none;
		cursor: pointer;
		padding: 2px;
		color: inherit;
	}
	.find {
		position: relative;
		max-width: 22rem;
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
		overflow-y: auto;
		padding: var(--space-1) 0;
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
		box-shadow: var(--shadow-pop);
	}
	.list ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.list li {
		display: flex;
		align-items: stretch;
	}
	.group {
		margin: var(--space-2) var(--space-3) var(--space-1);
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
	}
	.pick {
		flex: 1;
		text-align: left;
		padding: var(--space-2) var(--space-3);
		border: 0;
		background: none;
		cursor: pointer;
		color: var(--text-body);
		font-size: var(--type-sm);
	}
	.pick:hover,
	.pick:focus-visible {
		background: var(--wash);
		color: var(--text-bright);
	}
	.star {
		display: grid;
		place-items: center;
		width: var(--control-sm);
		border: 0;
		background: none;
		cursor: pointer;
		color: var(--text-muted);
	}
	.star:hover {
		color: var(--text-bright);
	}
	.star.on {
		color: var(--text-bright);
	}
	.none,
	.hint {
		margin: var(--space-2) var(--space-3);
		font-size: var(--type-xs);
	}
	.hint {
		padding-top: var(--space-2);
		border-top: 1px solid var(--line);
	}
	.quick .btn {
		gap: var(--space-1);
	}
</style>
