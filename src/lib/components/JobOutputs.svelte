<script lang="ts">
	// The files a job wrote that are still on disk: each opens in its own program, or shows in
	// its folder.
	import TermLink from './TermLink.svelte';
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { revealInFolder } from '#lib/tauri.js';

	let {
		written,
		onopen
	}: {
		/** The paths of the files written. */
		written: string[];
		/** Opens one of them; the job page reports a failure. */
		onopen: (path: string) => void;
	} = $props();
	const m = messages();
</script>

<section aria-labelledby="written-head" class="files">
	<h2 id="written-head">{m.jobs.outputs}<TermLink term="provenance" /></h2>
	<ul class="written">
		{#each written as path (path)}
			{@const name = path.split(/[\\/]/).pop()}
			<li>
				<button class="link mono" title={m.jobs.openFile} onclick={() => onopen(path)}
					><Icon name="file" size={14} />{name}</button
				>
				<button
					class="btn ghost sm icon"
					title={m.jobs.openFolder}
					aria-label="{m.jobs.openFolder}: {name}"
					onclick={() => revealInFolder(path)}><Icon name="folder" size={14} /></button
				>
			</li>
		{/each}
	</ul>
</section>

<style>
	.files {
		display: grid;
		gap: var(--space-3);
	}
	.written {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-1);
	}
	.written li {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.link {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		border: 0;
		background: none;
		color: var(--text-bright);
		cursor: pointer;
		padding: 0;
	}
	.link:hover {
		text-decoration: underline;
	}
</style>
