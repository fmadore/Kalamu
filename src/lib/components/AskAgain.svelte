<script lang="ts">
	// A question asked of the project's documents is kept with the project: once documents were
	// added since (or one could not be read), one line says how many have not been read against
	// it, and one button opens the dialog with the question, the model and the languages it was
	// asked with, over those documents alone; Start then asks what it always asks. Its answers
	// join the question's in the table (`answered`), the others are not paid for again.
	import ScreeningDialog from './ScreeningDialog.svelte';
	import type { ChoiceAnswers } from '#lib/bindings/ChoiceAnswers.js';
	import { messages } from '#lib/i18n/index.js';
	import type { Again } from '#lib/screeningStart.svelte.js';
	import type { FromProject } from '#lib/state.svelte.js';

	let {
		project,
		choices
	}: {
		project: FromProject;
		choices: ChoiceAnswers[];
	} = $props();
	const m = messages();
	const t = m.screening;

	/** The questions with documents not read against them, one each (a question's first choice). */
	const waiting = $derived(
		choices.filter(
			(g, i) =>
				g.question &&
				g.unread.length > 0 &&
				choices.findIndex((o) => o.job_id === g.job_id && o.setup_id === g.setup_id) === i
		)
	);
	let asking = $state<{ documents: string[]; again: Again } | null>(null);

	function ask(group: ChoiceAnswers) {
		asking = {
			documents: [...group.unread],
			again: {
				question: group.question ?? '',
				presetId: group.preset_id,
				provider: group.provider,
				model: group.model,
				languages: group.languages
			}
		};
	}
</script>

{#each waiting as group (`${group.job_id}/${group.setup_id}`)}
	<p class="again" role="status">
		<span>{t.notReadAgainst(group.unread.length, group.question ?? '')}</span>
		<button class="btn sm" onclick={() => ask(group)}>{t.askAgain}</button>
	</p>
{/each}

{#if asking}
	<ScreeningDialog
		{project}
		documents={asking.documents}
		again={asking.again}
		onclose={() => (asking = null)}
	/>
{/if}

<style>
	.again {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
		margin: 0 0 var(--space-3);
		font-size: var(--type-sm);
		color: var(--text-secondary);
		max-width: 80ch;
	}
</style>
