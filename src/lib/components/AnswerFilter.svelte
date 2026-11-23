<script lang="ts">
	// The filter of a project's documents by what an analysis answered, beside the tag filter,
	// shown once an analysis answered a choice; while an answer is chosen, *Export the whole table*
	// writes that analysis as one table, every document its question was asked of included.
	import Icon from './Icon.svelte';
	import type { ChoiceAnswers } from '#lib/bindings/ChoiceAnswers.js';
	import type { Action } from '#lib/action.svelte.js';
	import { askForFolders, pickTableSave, tableExported } from '#lib/dialogs.js';
	import { distinctTitles } from '#lib/answerGroups.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';
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

	// A question asked under two versions of its prompt is two groups, each named by when it
	// was first asked (`answerGroups`).
	const groups = $derived.by(() => {
		const options = answerOptions(choices, m);
		const titles = distinctTitles(
			options.map((g) => g.title),
			choices,
			(asked) => m.screening.askedSince(formatDateTime(asked))
		);
		return options.map((g, i) => ({ ...g, title: titles[i] }));
	});

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
			const renamed = await api.exportAnswers(projectId, group.job_id, group.setup_id, path);
			toast(tableExported(m.screening.exported, renamed));
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
		<!-- The whole table, not the rows the filter shows: said in its name and its tooltip. -->
		<button
			class="btn ghost sm"
			disabled={action.busy}
			title={m.screening.exportTableHint}
			onclick={exportAnswers}
		>
			<Icon name="download" size={14} />{m.screening.exportTable}
		</button>
	{/if}
{/if}
