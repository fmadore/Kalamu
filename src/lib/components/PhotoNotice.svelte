<script lang="ts">
	// A warning of the photograph check under its file in New job (#216): two photographs that
	// look like the same page, side by side, the sharper one suggested; or a photograph that looks
	// blurred. Nothing is left out without the researcher, and only from this job's list.
	import type { PhotoView } from '#lib/bindings/PhotoView.js';
	import { messages } from '#lib/i18n/index.js';
	import { warningText } from '#lib/fileNotes.js';
	import type { PhotoWarning } from '#lib/newJobPhotos.svelte.js';

	let {
		warning,
		nameOf,
		viewOf,
		onleaveout,
		onkeep
	}: {
		warning: PhotoWarning;
		nameOf: (sourceId: string) => string;
		viewOf: (sourceId: string) => PhotoView | undefined;
		/** Leaves the photograph out of this job's list. */
		onleaveout: (sourceId: string) => void;
		/** Keeps what the warning is about: both photographs, or the blurred one. */
		onkeep: () => void;
	} = $props();

	const m = messages();
	const title = $derived(warningText(warning, m, nameOf));
	/** The photographs shown: the earlier one first, as in the list. */
	const shown = $derived(
		warning.kind === 'same_page' ? [warning.other, warning.source_id] : [warning.source_id]
	);
	const keep = $derived(warning.kind === 'same_page' ? warning.keep : null);
	const clearly = $derived(warning.kind === 'same_page' && warning.clearly_sharper);
	const set = $derived(
		warning.kind === 'same_page'
			? warning.keep === warning.source_id
				? warning.other
				: warning.source_id
			: warning.source_id
	);
	const jpeg = (data: string) => `data:image/jpeg;base64,${data}`;
</script>

<div class="problem photo" role="group" aria-label={title}>
	<p class="title">{title}</p>
	<div class="photos" class:pair={shown.length === 2}>
		{#each shown as id (id)}
			{@const view = viewOf(id)}
			{@const name = nameOf(id)}
			<figure>
				{#if view}
					<div class="stage">
						<img class="whole" src={jpeg(view.page)} alt={m.warning.photos.whole(name)} />
					</div>
					<div class="stage">
						<img class="close" src={jpeg(view.detail)} alt={m.warning.photos.closeUp(name)} />
					</div>
				{/if}
				<figcaption>
					<span class="mono">{name}</span>
					{#if id === keep && clearly}<span class="chip ok">{m.warning.photos.sharperMark}</span
						>{/if}
				</figcaption>
			</figure>
		{/each}
	</div>
	<p>
		{#if warning.kind === 'same_page'}
			{clearly ? m.warning.photos.sharper(nameOf(warning.keep)) : m.warning.photos.asSharp}
		{:else}
			{m.warning.photos.blurredWhy}
		{/if}
		{m.warning.photos.stays}
	</p>
	<div class="actions">
		{#if warning.kind === 'same_page'}
			<button class="btn sm" onclick={() => onleaveout(set)}
				>{clearly
					? m.warning.photos.keepSharper
					: m.warning.photos.keepOnly(nameOf(warning.keep))}</button
			>
			<button class="btn sm" onclick={onkeep}>{m.warning.photos.keepBoth}</button>
		{:else}
			<button class="btn sm" onclick={onkeep}>{m.warning.photos.keepIt}</button>
			<button class="btn sm" onclick={() => onleaveout(warning.source_id)}
				>{m.warning.photos.leaveOut}</button
			>
		{/if}
	</div>
</div>

<style>
	/* The file list's notice treatment (warn text on the warn wash), grown to hold the photos. */
	.photo {
		display: grid;
		gap: var(--space-2);
		margin: 0 var(--space-2) var(--space-1) 1.125rem;
		padding: var(--space-3);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
		font-size: var(--type-xs);
	}
	.photo p {
		margin: 0;
		max-width: 75ch;
	}
	.title {
		font-size: var(--type-sm);
		font-weight: 600;
	}
	.photos {
		display: grid;
		grid-template-columns: minmax(0, 16rem);
		gap: var(--space-3);
	}
	.photos.pair {
		grid-template-columns: repeat(2, minmax(0, 16rem));
	}
	figure {
		display: grid;
		gap: var(--space-1);
		margin: 0;
		min-width: 0;
	}
	/* A photograph lies on the stage, dark in both themes (the Lightbox Rule). */
	.stage {
		display: grid;
		place-items: center;
		padding: var(--space-2);
		border-radius: var(--radius-control);
		background: var(--stage);
	}
	.stage img {
		display: block;
		max-width: 100%;
		border-radius: 2px;
		box-shadow:
			0 1px 2px rgba(0, 0, 0, 0.45),
			0 8px 20px rgba(0, 0, 0, 0.35);
	}
	.whole {
		max-height: 13rem;
		object-fit: contain;
	}
	.close {
		width: 100%;
		height: auto;
	}
	figcaption {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1) var(--space-2);
		align-items: center;
		overflow-wrap: anywhere;
		color: var(--text-bright);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
</style>
