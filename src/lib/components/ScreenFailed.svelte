<script lang="ts">
	// What a screen shows in its place when it broke as it was drawn: what happened, that what
	// was saved is kept, and the way out (back, another screen in the sidebar, or the window
	// loaded again). The failure itself went to the log (`unexpected.ts`). Opening another screen
	// draws it afresh, so the sidebar always works.
	import { messages } from '#lib/i18n/index.js';
	import { app, back, canGoBack } from '#lib/state.svelte.js';

	/** Draws the screen again, for when another is opened. */
	let { reset }: { reset: () => void } = $props();

	const m = messages();
	/** The screen that failed: once the view is another, it is drawn in this one's place. */
	const failed = app.view;
	$effect(() => {
		if (app.view !== failed) reset();
	});
</script>

<header class="masthead">
	<div>
		<h1>{m.unexpected.screenTitle}</h1>
	</div>
</header>
<div class="sheet">
	<div class="failed" role="alert">
		<p>{m.unexpected.screenBody}</p>
		<div class="actions">
			{#if canGoBack()}
				<button class="btn primary" onclick={back}>{m.unexpected.back}</button>
			{/if}
			<button class="btn" onclick={() => location.reload()}>{m.unexpected.reload}</button>
		</div>
	</div>
</div>

<style>
	.failed {
		display: grid;
		gap: var(--space-4);
		justify-items: start;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
</style>
