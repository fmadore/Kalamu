<script lang="ts">
	// The filter of a project's documents by what an analysis answered, beside the tag filter,
	// shown once an analysis answered a choice; while an answer is chosen, *Export this table*
	// writes that analysis as one table, every document its question was asked of included.
	import Icon from './Icon.svelte';
	import type { ChoiceAnswers } from '#lib/bindings/ChoiceAnswers.js';
	import type { Action } from '#lib/action.svelte.js';
	import { askForFolders, pickTableSave } from '#lib/dialogs.js';
	import { messages } from '#lib/i18n/index.js';
	import { answerOptions, chosenAnswer, tableName } from '#lib/screening.js';
	import { toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		projectId,
		projectName,
		choices,
		value,
		action,
		onpick
	}: {
		projectId: string;
		projectName: string;
		choices: ChoiceAnswers[];
		/** The answer filtered on, as `answerKey` names it; empty for none. */
		value: string;
		action: Action;
		onpick: (key: string) => void;
	} = $props();
	const m = messages();

	const groups = $derived(answerOptions(choices, m));

	/** The analysis filtered on, one row per document with the researcher's tags. */
	async function exportAnswers() {
		const found = chosenAnswer(choices, value);
		if (!found) return;
		const { group } = found;
		const path = await pickTableSave(tableName(projectName, group));
		if (!path) return;
		await action.run(async () => {
			// The CSV goes beside the workbook: in the Mac App Store's build, its folder is asked for.
			if (!(await askForFolders([path]))) return;
			await api.exportAnswers(projectId, group.job_id, group.setup_id, path);
			toast(m.screening.exported);
		}, null);
	}
</script>

{#if groups.length > 0}
	<select bind:value={() => value, (v) => onpick(v)} aria-label={m.screening.filter}>
		<option value="">{m.screening.anyAnswer}</option>
		{#each groups as g, i (i)}
			<optgroup label={g.title}>
				{#each g.options as o (o.key)}
					<option value={o.key}>{o.label}</option>
				{/each}
			</optgroup>
		{/each}
	</select>
	{#if value}
		<button class="btn ghost sm" disabled={action.busy} onclick={exportAnswers}
			><Icon name="download" size={14} />{m.screening.exportTable}</button
		>
	{/if}
{/if}
