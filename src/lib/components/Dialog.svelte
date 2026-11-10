<script lang="ts">
	// A dialog over a dimmed window: the screen behind it waits until it is answered. Focus moves
	// into it as it opens, Tab and Shift+Tab stay inside it, and focus goes back to what opened it
	// once it closes. Escape does what `onescape` says, when it is given. Everything else in the
	// window is inert while it is open, and the window's shortcuts wait.
	import type { Snippet } from 'svelte';
	import { holdShortcuts } from '#lib/state.svelte.js';

	let {
		labelledby,
		describedby,
		alert = false,
		size = 'narrow',
		onescape,
		children
	}: {
		/** The id of the dialog's heading. */
		labelledby: string;
		/** The id of the words that say what is asked, for an alert. */
		describedby?: string;
		/** A question the researcher must answer before going on, such as closing the window. */
		alert?: boolean;
		/** Narrow for a question, medium for a short form, wide for an editor. */
		size?: 'narrow' | 'medium' | 'wide';
		onescape?: () => void;
		children: Snippet;
	} = $props();

	const FOCUSABLE =
		'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])';

	/** What can take focus in the dialog, in order, leaving out what is hidden. */
	function focusable(box: HTMLElement): HTMLElement[] {
		return [...box.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
			(el) => el.getClientRects().length > 0
		);
	}

	/** Makes everything in the window but the dialog inert: every sibling of the dialog and of
	 * each of its ancestors. Returns what gives it back. */
	function inertAround(box: HTMLElement): () => void {
		const made: Element[] = [];
		for (let el: Element | null = box; el && el !== document.body; el = el.parentElement) {
			for (const sibling of el.parentElement?.children ?? []) {
				if (sibling === el || sibling.hasAttribute('inert')) continue;
				sibling.setAttribute('inert', '');
				made.push(sibling);
			}
		}
		return () => made.forEach((el) => el.removeAttribute('inert'));
	}

	/** Takes focus in, keeps Tab inside, and gives focus back to the opener on closing. */
	function trap(box: HTMLElement) {
		const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
		const release = holdShortcuts();
		const revive = inertAround(box);
		(focusable(box)[0] ?? box).focus();
		function keys(e: KeyboardEvent) {
			if (e.key !== 'Tab') return;
			const all = focusable(box);
			const first = all[0];
			const last = all.at(-1);
			const at = document.activeElement;
			if (!first || !last) {
				e.preventDefault();
				return;
			}
			if (e.shiftKey && (at === first || !box.contains(at))) {
				e.preventDefault();
				last.focus();
			} else if (!e.shiftKey && (at === last || !box.contains(at))) {
				e.preventDefault();
				first.focus();
			}
		}
		box.addEventListener('keydown', keys);
		return () => {
			box.removeEventListener('keydown', keys);
			revive();
			release();
			if (opener?.isConnected) opener.focus();
		};
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape' && onescape) onescape();
	}}
/>

<div class="scrim" role="presentation">
	<div
		class="dialog {size}"
		role={alert ? 'alertdialog' : 'dialog'}
		aria-modal="true"
		aria-labelledby={labelledby}
		aria-describedby={describedby}
		tabindex="-1"
		{@attach trap}
	>
		{@render children()}
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
		padding: var(--space-4);
	}
	.dialog {
		max-height: 90vh;
		overflow: auto;
		padding: var(--space-5);
		display: grid;
		gap: var(--space-4);
		box-shadow:
			0 0 0 1px var(--line),
			var(--shadow-flyout);
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
	}
	.dialog:focus {
		outline: none;
	}
	.narrow {
		max-width: 32rem;
	}
	.medium {
		width: min(40rem, 100%);
	}
	.wide {
		width: min(48rem, 100%);
	}
</style>
