<script lang="ts">
	// How a long interview was read, above its answers: one quiet line saying in how many parts,
	// and for a part that could not be read a notice naming its minutes (or its turns), why, and
	// Read it again, which puts the part back in the queue; the parts are then brought together
	// again (`parts.ts`).
	import type { PartsRead } from '#lib/parts.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		jobId,
		parts,
		onerror
	}: {
		jobId: string;
		parts: PartsRead;
		/** Says why the part could not be put back in the queue. */
		onerror: (message: string) => void;
	} = $props();
	const m = messages();

	let retrying = $state(false);
	async function readAgain() {
		retrying = true;
		try {
			await api.retryFailed(jobId);
			toast(m.cited.readingAgain);
		} catch (e) {
			onerror(describeError(e, m));
		} finally {
			retrying = false;
		}
	}
</script>

<div class="parts" role="note">
	<p>{m.cited.readInParts(parts.count)}</p>
	{#each parts.unread as part (part.number)}
		<p class="unread">
			{m.cited.partNotRead(part.from, part.to, part.first, part.last)}
			{#if part.error}<span class="why">{part.error}</span>{/if}
		</p>
	{/each}
	{#if parts.unread.length > 0}
		<button class="btn sm" disabled={retrying} onclick={readAgain}>{m.cited.readAgain}</button>
	{/if}
</div>

<style>
	/* One quiet line, and a notice for a part not read. */
	.parts {
		display: grid;
		gap: var(--space-2);
		justify-items: start;
		padding-bottom: var(--space-3);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-secondary);
	}
	p {
		margin: 0;
	}
	.unread {
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-control);
		background: var(--warn-wash);
		color: var(--warn-text);
	}
	.why {
		display: block;
		font-family: var(--font-mono);
		font-size: var(--type-xs);
	}
</style>
