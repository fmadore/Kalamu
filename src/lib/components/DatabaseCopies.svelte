<script lang="ts">
	// Settings, General: the copies of the job database, where they are, and one opened in
	// place of the database after asking. Quiet: nothing to do here on an ordinary day.
	import type { DatabaseCopies } from '#lib/bindings/DatabaseCopies.js';
	import { newestCopy, openCopy } from '#lib/databaseCopies.js';
	import { formatDate, messages } from '#lib/i18n/index.js';
	import { forSystem } from '#lib/platform.js';
	import { app } from '#lib/state.svelte.js';
	import { api, listenWhileMounted, on, revealInFolder } from '#lib/tauri.js';

	const m = messages();
	let view = $state<DatabaseCopies | null>(null);

	async function load() {
		view = await api.databaseCopies().catch(() => null);
	}

	$effect(() => {
		void load();
		return listenWhileMounted(() => on.databaseChecked(() => void load()));
	});

	// Opening a copy restarts Kalamu, which waits until no job runs.
	const jobsRunning = $derived(app.jobs.some((j) => j.status === 'running'));
	const hasSetAside = $derived(view?.copies.some((c) => c.kind === 'set_aside') ?? false);
	const newest = $derived(view ? newestCopy(view) : null);

	/** The newest copy shown in its folder, or the folder itself when there is none. */
	function show() {
		if (!view) return;
		void revealInFolder(view.copies[0]?.path ?? view.folder);
	}
</script>

<section class="card block" aria-labelledby="database-copies">
	<h2 id="database-copies">{m.database.title}</h2>
	{#if view}
		{#if view.damaged}
			<p class="notice" role="alert">{m.database.damaged}</p>
		{/if}
		{#if view.skipped}
			<p class="notice" role="status">
				{m.database.skipped(m.settings.size(view.skipped.free))}
			</p>
		{/if}
		<p class="muted small">{m.database.hint}</p>
		<div class="row">
			<span class="path small">{view.folder}</span>
			<button class="btn sm ghost" onclick={show}>{forSystem(m.database.show)}</button>
		</div>
		{#if view.copies.length === 0}
			<p class="muted small">{m.database.none}</p>
		{:else}
			<ul class="copies" aria-label={m.database.listLabel}>
				{#each view.copies as copy (copy.name)}
					<li>
						<span class="small" class:strong={copy.name === newest?.name}>
							{m.database.line(
								formatDate(copy.made),
								m.database.kinds[copy.kind],
								m.settings.size(copy.bytes)
							)}
						</span>
						<button
							class="btn sm"
							disabled={jobsRunning}
							title={jobsRunning ? m.database.openBusy : undefined}
							onclick={() => openCopy(copy)}
						>
							{m.database.open}
						</button>
					</li>
				{/each}
			</ul>
			{#if jobsRunning}<p class="muted small">{m.database.openBusy}</p>{/if}
			{#if hasSetAside}<p class="muted small">{m.database.setAsideHint}</p>{/if}
		{/if}
	{/if}
</section>

<style>
	/* One section like the others in General: a heading, a line of text, a short list. */
	.block {
		padding: var(--space-4) 0 var(--space-5);
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: var(--space-3);
	}
	.block > * {
		margin: 0;
	}
	.small {
		font-size: var(--type-sm);
	}
	.path {
		overflow-wrap: anywhere;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
	}
	.copies {
		list-style: none;
		padding: 0;
		display: grid;
		gap: var(--space-2);
	}
	.copies li {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2) var(--space-3);
		align-items: center;
		justify-content: space-between;
		max-width: 40rem;
	}
	.strong {
		font-weight: 600;
	}
</style>
