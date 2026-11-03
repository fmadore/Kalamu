<script lang="ts">
	// A review opened from the list of what to check says which item it is and why it is listed,
	// and steps to the previous or next item, in this review or another, or back to the list as
	// it was set. Marking the page reviewed stays the review's own button.
	import Icon from './Icon.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { placeOf, reasonText } from '#lib/queue.js';
	import { app, go } from '#lib/state.svelte.js';

	let { jobId }: { jobId: string } = $props();
	const m = messages();
	const q = m.queue;

	const trail = $derived(app.queue);
	const item = $derived(trail?.items[trail.at]);
	/** Shown while the review is the item's: opened from the list, not from elsewhere since. */
	const here = $derived(
		item !== undefined &&
			app.view.name === 'review' &&
			item.job_id === jobId &&
			item.source_id === app.view.sourceId
	);

	function step(delta: number) {
		if (!trail) return;
		const at = trail.at + delta;
		const next = trail.items[at];
		if (!next) return;
		trail.at = at;
		go({
			name: 'review',
			jobId: next.job_id,
			sourceId: next.source_id,
			index: next.index ?? undefined
		});
	}

	function back() {
		if (!trail) return;
		go({
			name: 'projects',
			projectId: trail.projectId,
			pane: 'queue',
			selection: trail.scope === 'selection' ? trail.selection : undefined
		});
	}
</script>

{#if here && trail && item}
	<div class="queue-banner" role="status">
		<Icon name="flag" size={16} />
		<p>
			<strong class="num">{q.banner.position(trail.at + 1, trail.items.length)}</strong>
			<span>{item.document}, {placeOf(item, m)}, {item.setup_label}:</span>
			{item.signals.map((s) => reasonText(s, m, item.piece === null)).join('; ')}
		</p>
		<span class="buttons">
			<button class="btn sm ghost" onclick={() => step(-1)} disabled={trail.at === 0}
				><Icon name="previous" size={14} />{q.banner.previous}</button
			>
			<button class="btn sm" onclick={() => step(1)} disabled={trail.at >= trail.items.length - 1}
				>{q.banner.next}<Icon name="chevron" size={14} /></button
			>
			<button class="btn sm ghost" onclick={back}>{q.banner.back}</button>
		</span>
	</div>
{/if}

<style>
	.queue-banner {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2) var(--space-3);
		padding: var(--space-2) var(--space-4);
		border-bottom: 1px solid var(--line);
		background: var(--chrome);
		color: var(--text-secondary);
		font-size: var(--type-sm);
	}
	p {
		flex: 1 1 20rem;
		margin: 0;
		min-width: 0;
	}
	strong {
		margin-right: var(--space-2);
		color: var(--text-bright);
		font-weight: 500;
	}
	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
	}
</style>
