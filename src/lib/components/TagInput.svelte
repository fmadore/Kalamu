<script lang="ts">
	// A tag typed or chosen among those the project already uses, added with Enter or the
	// button. The store settles its spelling: one differing only in capitals is the same tag.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';

	let { known, onadd }: { known: string[]; onadd: (tag: string) => Promise<void> } = $props();
	const m = messages();
	const listId = $props.id();

	let draft = $state('');
	const typed = $derived(draft.trim());

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		if (!typed) return;
		const tag = typed;
		draft = '';
		await onadd(tag);
	}
</script>

<form class="tag-input" onsubmit={submit}>
	<input
		type="text"
		list={listId}
		bind:value={draft}
		placeholder={m.projects.addTag}
		aria-label={m.projects.addTag}
		title={m.projects.addTagHint}
	/>
	<datalist id={listId}>
		{#each known as tag (tag)}
			<option value={tag}></option>
		{/each}
	</datalist>
	<button class="btn sm" type="submit" disabled={!typed}
		><Icon name="tag" size={14} />{m.projects.tagButton}</button
	>
</form>

<style>
	.tag-input {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
	}
	input {
		width: 11rem;
	}
</style>
