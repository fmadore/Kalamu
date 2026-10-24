<script lang="ts">
	import { messages } from '#lib/i18n/index.js';
	import { api } from '#lib/tauri.js';
	import { app } from '#lib/state.svelte.js';

	const m = messages();
	let quitting = $state(false);

	async function quit() {
		quitting = true;
		await api.quitApp();
	}
</script>

<div class="scrim" role="presentation">
	<div
		class="dialog card"
		role="alertdialog"
		aria-labelledby="close-title"
		aria-describedby="close-body"
	>
		<h2 id="close-title">{m.close.title}</h2>
		<p id="close-body">{m.close.body}</p>
		<div class="actions">
			<button class="btn" onclick={() => (app.closeRequested = false)}>{m.close.stay}</button>
			<button class="btn primary" disabled={quitting} onclick={quit}>{m.close.quit}</button>
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
		max-width: 28rem;
		padding: var(--space-5);
		display: grid;
		gap: var(--space-4);
		box-shadow: var(--shadow-flyout);
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
