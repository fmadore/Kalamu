<script lang="ts">
	import Dialog from './Dialog.svelte';
	import { messages } from '#lib/i18n/index.js';
	import { osNames } from '#lib/platform.js';
	import { describeError } from '#lib/errors.js';
	import { api, notify } from '#lib/tauri.js';
	import { app, toast } from '#lib/state.svelte.js';

	const m = messages();
	let quitting = $state(false);
	/** Pieces sent and not yet back, which quitting waits for: they are paid for. */
	const inFlight = $derived(
		Object.values(app.live).reduce(
			(n, live) => n + Object.values(live.pieces).filter((p) => p.status === 'running').length,
			0
		)
	);

	/** Quits, waiting for the pieces sent; if the app cannot, the question is asked again. */
	async function quit() {
		quitting = true;
		await quitNow();
	}

	/** Asked again while it waits, the app quits at once. */
	async function quitNow() {
		try {
			await api.quitApp();
		} catch (e) {
			quitting = false;
			toast(describeError(e, m), { kind: 'error' });
		}
	}

	/** Escape answers "Stay", unless quitting is already under way. */
	function stay() {
		if (!quitting) app.closeRequested = false;
	}

	async function background() {
		app.closeRequested = false;
		await api.hideWindow();
		void notify(m.app.name, m.close.hidden(osNames().reopen), true);
	}
</script>

<Dialog labelledby="close-title" describedby="close-body" alert onescape={stay}>
	<h2 id="close-title">{m.close.title}</h2>
	<p id="close-body" aria-live="polite">
		{quitting && inFlight > 0 ? m.close.waiting(inFlight) : m.close.body(osNames().tray)}
	</p>
	<div class="actions">
		{#if quitting}
			<button class="btn" onclick={quitNow}>{m.close.quitNow}</button>
		{:else}
			<button class="btn" onclick={stay}>{m.close.stay}</button>
			<button class="btn" onclick={quit}>{m.close.quit}</button>
			<button class="btn primary" onclick={background}>{m.close.background}</button>
		{/if}
	</div>
</Dialog>

<style>
	.actions {
		display: flex;
		flex-wrap: wrap;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
