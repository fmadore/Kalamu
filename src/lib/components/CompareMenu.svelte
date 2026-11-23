<script lang="ts">
	// Kalamu's menu in the text compared, the review's third pane, from a right click, the menu
	// key or Shift+F10 there: copy the words selected in it, or the whole text; and on a place
	// where it reads differently from the text shown, use its reading there, or open the
	// readings of that place over the text shown, as a click on it does.
	import ActionMenu, { type Action } from './ActionMenu.svelte';
	import type { Entry } from '#lib/apparatus.js';
	import type { Witness } from '#lib/compare.svelte.js';
	import { messages } from '#lib/i18n/index.js';
	import { copyText, isMenuKey, openFromClick } from '#lib/menu.js';
	import { shortcutsHeld } from '#lib/state.svelte.js';

	let {
		pane,
		compared,
		entries,
		canUse,
		onuse,
		onshow
	}: {
		/** The third pane, where the text compared is shown. */
		pane: HTMLElement | null;
		compared: Witness | undefined;
		entries: Entry[];
		/** Whether a reading can be taken into the text shown now (it is not being edited). */
		canUse: boolean;
		onuse: (entry: Entry, text: string) => void;
		onshow: (id: number) => void;
	} = $props();
	const m = messages();

	let menu = $state<{ x: number; y: number; actions: Action[] } | null>(null);

	/** The words selected in the pane, if the selection is there. */
	function selected(): string {
		const sel = getSelection();
		if (!pane || !sel || sel.isCollapsed || !sel.anchorNode || !sel.focusNode) return '';
		if (!pane.contains(sel.anchorNode) || !pane.contains(sel.focusNode)) return '';
		return sel.toString();
	}

	/** What can be done from `target` in the pane. */
	function actionsAt(target: Element | null): Action[] {
		const witness = compared;
		if (!witness) return [];
		const out: Action[] = [];
		const words = selected();
		if (words) out.push({ label: m.passage.copy, icon: 'copy', run: () => void copyText(words) });
		const id = Number(target?.closest<HTMLElement>('[data-entry]')?.dataset.entry ?? NaN);
		const entry = entries.find((e) => e.id === id);
		const reading = entry?.readings.find((r) => r.siglum === witness.siglum);
		if (entry && reading && canUse) {
			out.push({
				label: m.review.useReading,
				icon: 'check',
				run: () => onuse(entry, reading.text)
			});
		}
		if (entry)
			out.push({ label: m.review.showReadings, icon: 'compare', run: () => onshow(entry.id) });
		out.push({
			label: m.review.copyCompared,
			icon: 'copy',
			run: () => void copyText(witness.text)
		});
		return out;
	}

	function open(target: Element | null, at: { x: number; y: number }) {
		const actions = actionsAt(target);
		if (actions.length > 0) menu = { ...at, actions };
	}

	/** Under the element from the keyboard, which has no pointer. */
	function under(el: Element | null): { x: number; y: number } {
		const box = (el ?? pane)?.getBoundingClientRect();
		return { x: box?.left ?? 0, y: box?.bottom ?? 0 };
	}

	$effect(() => {
		const el = pane;
		if (!el) return;
		const onContext = (e: MouseEvent) => {
			const target = e.target instanceof Element ? e.target : null;
			if (menu) {
				e.preventDefault();
				return;
			}
			openFromClick(e, (at) => open(target, at ?? under(target)));
		};
		const onKey = (e: KeyboardEvent) => {
			if (shortcutsHeld() || !isMenuKey(e) || menu) return;
			e.preventDefault();
			// Kept from the window's own handlers, so only this menu opens.
			e.stopPropagation();
			const target = document.activeElement instanceof Element ? document.activeElement : null;
			open(target, under(target));
		};
		el.addEventListener('contextmenu', onContext);
		el.addEventListener('keydown', onKey);
		return () => {
			el.removeEventListener('contextmenu', onContext);
			el.removeEventListener('keydown', onKey);
		};
	});
</script>

{#if menu}
	<ActionMenu
		x={menu.x}
		y={menu.y}
		label={m.review.compareMenu}
		actions={menu.actions}
		onclose={() => (menu = null)}
	/>
{/if}
