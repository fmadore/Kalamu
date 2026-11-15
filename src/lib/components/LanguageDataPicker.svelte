<script lang="ts">
	// The language data not downloaded yet, found by typing its name rather than scrolled
	// through: a search box over the list, each language with its size. Choosing one downloads
	// it. Its keys are the job's language picker's (`FindBox`): arrows move, Enter takes the first
	// found.
	import FindBox from './FindBox.svelte';
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

	let query = $state('');
	let open = $state(false);

	const found = $derived.by(() => {
		const q = searchKey(query.trim());
		return q === '' ? items : items.filter((i) => searchKey(i.name).includes(q));
	});

	function pick(id: string) {
		query = '';
		open = false;
		onpick(id);
	}

	/** Enter in the box downloads the first found, unless another download runs. */
	function takeFirst(): boolean {
		if (found.length === 0 || disabled) return false;
		pick(found[0].id);
		return true;
	}
</script>

<!-- The box sits at the foot of Settings: the list opened below it is scrolled into view. -->
<FindBox
	bind:query
	bind:open
	label={m.settings.findLanguageData}
	placeholder={m.settings.findLanguageData}
	items="button"
	width="26rem"
	clipX
	reveal
	onenter={takeFirst}
>
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
</FindBox>

<style>
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
	button:hover:not(:disabled) {
		background: var(--wash);
		color: var(--text-bright);
	}
	/* The keyboard's place, told apart from the pointer's: the app's focus ring, inside the row. */
	button:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(-1 * var(--focus-width));
		color: var(--text-bright);
	}
	/* Unavailable, yet legible: muted beside the bright items, with no hover. */
	button:disabled {
		color: var(--text-muted);
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
