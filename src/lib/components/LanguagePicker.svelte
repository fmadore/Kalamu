<script lang="ts">
	// The languages of a job: chosen ones as chips, then a search box over the offered languages
	// in alphabetical order, favourites first. Starring a language saves it in Settings.
	import FindBox from './FindBox.svelte';
	import Icon from './Icon.svelte';
	import type { LanguageInfo } from '#lib/bindings/LanguageInfo.js';
	import { Action } from '#lib/action.svelte.js';
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
	/** Saving the favourites, what went wrong said in an error toast. */
	const action = new Action({ report: (message) => toast(message, { kind: 'error' }) });

	let query = $state('');
	let open = $state(false);
	let finder = $state<FindBox>();

	const favourites = $derived(app.settings?.favourite_languages ?? []);
	const name = (l: LanguageInfo) => languageName(l.code, l.name);
	const label = (code: string) => (app.catalog ? languageLabel(app.catalog, code) : code);
	const found = $derived(
		findLanguages(
			offered.filter((l) => !selected.includes(l.code)),
			query,
			name
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
		finder?.focus();
	}

	async function toggleFavourite(code: string) {
		if (!app.settings) return;
		const next = favourites.includes(code)
			? favourites.filter((c) => c !== code)
			: [...favourites, code];
		app.settings.favourite_languages = next;
		const settings = $state.snapshot(app.settings);
		await action.run(() => api.saveSettings(settings));
	}

	/** Enter in the box takes the first language found, a favourite before the others. */
	function takeFirst(): boolean {
		if (found.length === 0) return false;
		add((starred[0] ?? others[0]).code);
		return true;
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

	<FindBox
		bind:this={finder}
		bind:query
		bind:open
		label={m.newJob.findLanguage}
		placeholder={m.newJob.addLanguage}
		items="button.pick"
		width="22rem"
		onenter={takeFirst}
	>
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
	</FindBox>

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
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	li {
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
