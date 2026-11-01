<script lang="ts">
	// One dictionary row opened: its label to rename, the forms it gathers (each can be taken
	// out again) and the results that listed them, each opening its review.
	import Icon from './Icon.svelte';
	import type { DictionaryRow } from '#lib/bindings/DictionaryRow.js';
	import type { Evidence } from '#lib/bindings/Evidence.js';
	import { messages } from '#lib/i18n/index.js';
	import { go } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		row,
		evidence,
		act
	}: {
		projectId: string;
		row: DictionaryRow;
		evidence: Evidence[];
		act: (action: () => Promise<unknown>) => Promise<void>;
	} = $props();
	const m = messages();

	let renaming = $state<string | null>(null);

	function rename(event: SubmitEvent) {
		event.preventDefault();
		if (!row.entry || renaming === null) return;
		const entry = row.entry;
		const label = renaming;
		renaming = null;
		void act(() => api.dictionaryRelabel(projectId, entry, label));
	}
</script>

{#if row.entry}
	{#if renaming !== null}
		<form class="rename" onsubmit={rename}>
			<input type="text" bind:value={renaming} aria-label={m.dictionary.label} />
			<button class="btn sm" type="submit" disabled={!renaming.trim()}>{m.dictionary.save}</button>
			<button class="btn ghost sm" type="button" onclick={() => (renaming = null)}
				>{m.dictionary.cancel}</button
			>
		</form>
	{:else}
		<button class="btn ghost sm" onclick={() => (renaming = row.label)}
			><Icon name="edit" size={14} />{m.dictionary.rename}</button
		>
	{/if}
{/if}
<h3>{m.dictionary.forms}</h3>
<ul>
	{#each row.forms as f (f.form)}
		<li>
			<span>{f.form}</span>
			<span class="muted num">{m.dictionary.formDocuments(f.documents)}</span>
			{#if row.entry && row.forms.length > 1}
				<button
					class="untie"
					aria-label={m.dictionary.takeOut(f.form)}
					title={m.dictionary.takeOut(f.form)}
					onclick={() => act(() => api.dictionarySplit(projectId, row.entry ?? '', [f.form]))}
					><Icon name="unlink" size={13} /></button
				>
			{/if}
		</li>
	{/each}
</ul>
<h3>{m.dictionary.evidence}</h3>
{#if evidence.length === 0}
	<p class="muted">{m.dictionary.noEvidence}</p>
{:else}
	<ul>
		{#each evidence as e (`${e.job_id}/${e.source_id}/${e.setup_label}`)}
			<li>
				<button
					class="link"
					title={m.dictionary.openReview}
					onclick={() => go({ name: 'review', jobId: e.job_id, sourceId: e.source_id })}
					>{e.document}</button
				>
				<span class="muted">{e.job_name} · {e.setup_label}</span>
				{#each e.forms as form (form)}<span class="chip">{form}</span>{/each}
			</li>
		{/each}
	</ul>
{/if}

<style>
	h3 {
		margin: var(--space-3) 0 var(--space-1);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	.rename {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: var(--space-1);
	}
	li {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.untie {
		display: inline-flex;
		padding: 0 2px;
		border: 0;
		background: none;
		color: var(--text-muted);
		cursor: pointer;
	}
	.untie:hover {
		color: var(--text-bright);
	}
</style>
