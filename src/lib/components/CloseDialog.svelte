<script lang="ts">
	import { messages } from '#lib/i18n/index.js';
	import { api, notify } from '#lib/tauri.js';
	import { app } from '#lib/state.svelte.js';

	const m = messages();
	let quitting = $state(false);
	/** Pieces sent and not yet back, which quitting waits for: they are paid for. */
	const inFlight = $derived(
		Object.values(app.live).reduce(
			(n, live) => n + Object.values(live.pieces).filter((p) => p.status === 'running').length,
			0
		)
	);

	async function quit() {
		quitting = true;
		await api.quitApp();
	}

	/** Asked again while it waits, the app quits at once. */
	async function quitNow() {
		await api.quitApp();
	}

	async function background() {
		app.closeRequested = false;
		await api.hideWindow();
		void notify(m.app.name, m.close.hidden, true);
	}
</script>

<div class="scrim" role="presentation">
	<div
		class="dialog"
		role="alertdialog"
		aria-labelledby="close-title"
		aria-describedby="close-body"
	>
		<h2 id="close-title">{m.close.title}</h2>
		<p id="close-body" aria-live="polite">
			{quitting && inFlight > 0 ? m.close.waiting(inFlight) : m.close.body}
		</p>
		<div class="actions">
			{#if quitting}
				<button class="btn" onclick={quitNow}>{m.close.quitNow}</button>
			{:else}
				<button class="btn" onclick={() => (app.closeRequested = false)}>{m.close.stay}</button>
				<button class="btn" onclick={quit}>{m.close.quit}</button>
				<button class="btn primary" onclick={background}>{m.close.background}</button>
			{/if}
		</div>
	</div>
</div>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: var(--modal-scrim);
		display: grid;
		place-items: center;
		z-index: 50;
	}
	.dialog {
		max-width: 32rem;
		padding: var(--space-5);
		display: grid;
		gap: var(--space-4);
		box-shadow:
			0 0 0 1px var(--line),
			var(--shadow-flyout);
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
