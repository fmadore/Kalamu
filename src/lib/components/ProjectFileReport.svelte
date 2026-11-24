<script lang="ts">
	// What a check of a project file found (#246): a short verdict first, then what the file
	// holds and what opening it would not bring back as it was. Rust decides each part; this
	// only says it.
	import type { ProjectFileCheck } from '#lib/bindings/ProjectFileCheck.js';
	import { formatDateTime, messages } from '#lib/i18n/index.js';

	let { check, verdict = true }: { check: ProjectFileCheck; verdict?: boolean } = $props();
	const m = messages();
	const t = m.projectFile;
</script>

<div class="report">
	{#if verdict}
		<p class="verdict" class:bad={check.verdict !== 'whole'}>{t.verdict[check.verdict]}</p>
	{/if}
	<p class="muted">
		{t.checkedFile(check.name, formatDateTime(check.saved_at), check.app_version)}
		{t.documents(check.documents, check.jobs)}.
	</p>

	{#if check.damaged.length > 0}
		<div>
			<p>{t.damagedParts(check.damaged.length)}</p>
			<ul>
				{#each check.damaged as name, i (i)}<li>{name}</li>{/each}
			</ul>
		</div>
	{/if}

	{#if check.missing_texts.length > 0}
		<div>
			<p>{t.checkMissingTexts(check.missing_texts.length)}</p>
			<ul>
				{#each check.missing_texts as text, i (i)}
					<li>
						{t.missingText(
							text.documents.length > 0 ? text.documents.join(', ') : text.file_name,
							text.jobs.join(', ')
						)}
					</li>
				{/each}
			</ul>
		</div>
	{:else if check.texts > 0}
		<p>{t.textsWhole(check.texts)}</p>
	{/if}

	{#if check.originals > 0}<p>{t.originalsIn(check.originals)}</p>{/if}
	{#if check.originals_not_chosen > 0}<p>{t.originalsNotChosen}</p>{/if}
	{#if check.originals_left > 0}<p>{t.originalsNotCarried(check.originals_left)}</p>{/if}

	{#if check.to_find.length > 0}
		<div>
			<p>{t.toFind(check.to_find.length, m.projects.missing)}</p>
			<ul>
				{#each check.to_find as title, i (i)}<li>{title}</li>{/each}
			</ul>
		</div>
	{/if}

	{#if check.renamed.length > 0}
		<div>
			<p>{t.renamed(check.renamed.length)}</p>
			<ul>
				{#each check.renamed as file, i (i)}
					<li>{t.renamedFile(file.documents.join(', '), file.from, file.to)}</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>

<style>
	.report {
		display: grid;
		gap: var(--space-2);
	}
	.report p {
		margin: 0;
	}
	.verdict {
		font-weight: 600;
		color: var(--text-bright);
	}
	.verdict.bad {
		color: var(--danger-text);
	}
	ul {
		margin: var(--space-1) 0 0;
		padding-left: var(--space-4);
		overflow-wrap: anywhere;
	}
</style>
