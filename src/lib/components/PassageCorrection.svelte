<script lang="ts">
	// *Correct a passage with a model*, from a selection in the text review to the dialog and
	// back: the selection followed while a passage can be corrected (`chosen`, for the review's
	// visible button, which calls `correct`), Kalamu's menu on it from a right click, the menu
	// key or Shift+F10, the dialog, and a proposal accepted followed by the message that undoes
	// it, the focus back in the text.
	import { tick } from 'svelte';
	import PassageDialog from './PassageDialog.svelte';
	import PassageMenu from './PassageMenu.svelte';
	import type { Correction } from '#lib/bindings/Correction.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { selectionBox, selectionInText, type Chosen } from '#lib/passage.js';
	import type { Review } from '#lib/review.svelte.js';
	import { shortcutsHeld, toast } from '#lib/state.svelte.js';
	import { api } from '#lib/tauri.js';

	let {
		review,
		sourceId,
		index,
		base,
		textPane,
		offered,
		chosen = $bindable(null)
	}: {
		review: Review;
		sourceId: string;
		index: number;
		/** The text shown, corrections included. */
		base: string;
		textPane: HTMLElement | null;
		/** A passage of the text shown can be corrected: a text read, not being edited. */
		offered: boolean;
		/** The text selected in the text shown, while there is some. */
		chosen?: Chosen | null;
	} = $props();
	const m = messages();

	/** The menu opened from a selection, and where. */
	let menu = $state<{ x: number; y: number; chosen: Chosen } | null>(null);
	/** The passage a model is being asked to correct. */
	let asked = $state<Chosen | null>(null);

	$effect(() => {
		const update = () => (chosen = offered ? selectionInText(textPane, base) : null);
		update();
		document.addEventListener('selectionchange', update);
		return () => document.removeEventListener('selectionchange', update);
	});

	/** Opens the selection's menu at the pointer, or under the selection from the keyboard. */
	function openMenu(at: Chosen, pointer: { x: number; y: number } | null) {
		const box = pointer ? null : selectionBox();
		menu = {
			x: pointer?.x ?? box?.left ?? 0,
			y: pointer?.y ?? box?.bottom ?? 0,
			chosen: at
		};
	}

	/** The menu key, or Shift+F10, on a selection in the text: its menu, under it. */
	function onKey(e: KeyboardEvent) {
		if (shortcutsHeld()) return;
		if ((e.key === 'ContextMenu' || (e.shiftKey && e.key === 'F10')) && offered && !menu) {
			const at = selectionInText(textPane, base);
			if (at) {
				e.preventDefault();
				openMenu(at, null);
			}
		}
	}

	/** A right click on the selection, or the menu key: Kalamu's menu where the selection is in
	 * the text, the system's own everywhere else. */
	function onContextMenu(e: MouseEvent) {
		if (menu) {
			e.preventDefault();
			return;
		}
		if (!offered || !textPane) return;
		const target = e.target instanceof Node ? e.target : null;
		const fromKeys = e.clientX === 0 && e.clientY === 0;
		const inPane = target !== null && textPane.contains(target);
		if (!inPane && !(fromKeys && (target === document.body || target === null))) return;
		const at = selectionInText(textPane, base);
		if (!at) return;
		e.preventDefault();
		const pointer = fromKeys ? null : { x: e.clientX, y: e.clientY };
		// WebKit on macOS opens the menu as the button goes down: the menu waits for it to come
		// up, or that release would close it again at once.
		if (e.buttons === 0) {
			openMenu(at, pointer);
			return;
		}
		const open = () => setTimeout(() => openMenu(at, pointer));
		addEventListener('pointerup', open, { once: true });
	}

	/** Asks a model to correct the passage `at`. */
	export function correct(at: Chosen) {
		menu = null;
		asked = at;
	}

	/** Gives the focus back to the text once the dialog is gone: to the pane, or, where the
	 * pane is no tab stop of its own, to the text's own tab stop. */
	async function backToText() {
		asked = null;
		await tick();
		const pane = textPane;
		if (!pane) return;
		const stop = pane.hasAttribute('tabindex')
			? pane
			: (pane.querySelector<HTMLElement>('.text.numbered [tabindex="0"]') ?? pane);
		if (stop === pane && !pane.hasAttribute('tabindex')) pane.tabIndex = -1;
		stop.focus({ preventScroll: true });
	}

	async function accepted(id: string, before: Correction) {
		asked = null;
		await review.refresh();
		await backToText();
		toast(m.passage.accepted, {
			action: { label: m.common.undo, run: () => void undo(id, before) }
		});
	}

	async function undo(id: string, before: Correction) {
		try {
			await api.undoPassage(id, before);
			await review.refresh();
			toast(m.passage.undone);
		} catch (e) {
			review.error = describeError(e, m);
		}
	}
</script>

<svelte:window onkeydown={onKey} oncontextmenu={onContextMenu} />

{#if menu}
	{@const open = menu}
	<PassageMenu
		text={base.slice(open.chosen.start, open.chosen.end)}
		x={open.x}
		y={open.y}
		oncorrect={() => correct(open.chosen)}
		onclose={() => (menu = null)}
	/>
{/if}
{#if asked && review.job && review.setups[0]}
	<PassageDialog
		job={review.job}
		setupId={review.setups[0].id}
		{sourceId}
		{index}
		{base}
		chosen={asked}
		onaccepted={accepted}
		onclose={backToText}
	/>
{/if}
