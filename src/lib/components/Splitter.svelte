<script lang="ts">
	// A handle on the boundary between two panes. Drag it, or focus it and use the arrow keys, to
	// move the boundary; a double click puts it back. The parent owns the sizes: it places the
	// handle at `position` (a share of its own width) and is told where the boundary should go.
	let {
		label,
		position,
		onset,
		onreset
	}: {
		label: string;
		position: number;
		onset: (position: number) => void;
		onreset: () => void;
	} = $props();

	let handle = $state<HTMLElement | null>(null);
	let dragging = $state(false);

	function at(clientX: number): number | null {
		const box = handle?.parentElement?.getBoundingClientRect();
		return box && box.width > 0 ? (clientX - box.left) / box.width : null;
	}

	function down(e: PointerEvent) {
		if (e.button !== 0 || !handle) return;
		e.preventDefault();
		handle.setPointerCapture(e.pointerId);
		dragging = true;
		document.documentElement.classList.add('resizing');
	}

	function move(e: PointerEvent) {
		if (!dragging) return;
		const p = at(e.clientX);
		if (p !== null) onset(p);
	}

	function up() {
		dragging = false;
		document.documentElement.classList.remove('resizing');
	}

	function key(e: KeyboardEvent) {
		const step = e.shiftKey ? 0.1 : 0.02;
		const next =
			e.key === 'ArrowLeft'
				? position - step
				: e.key === 'ArrowRight'
					? position + step
					: e.key === 'Home'
						? 0
						: e.key === 'End'
							? 1
							: null;
		if (next === null) return;
		e.preventDefault();
		onset(next);
	}
</script>

<!-- A focusable separator is a widget in ARIA, moved with the arrow keys; Svelte reads the role as static. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
	bind:this={handle}
	class="splitter"
	class:dragging
	style:left="{position * 100}%"
	role="separator"
	aria-orientation="vertical"
	aria-label={label}
	aria-valuemin={0}
	aria-valuemax={100}
	aria-valuenow={Math.round(position * 100)}
	tabindex="0"
	title={label}
	onpointerdown={down}
	onpointermove={move}
	onpointerup={up}
	onpointercancel={up}
	onlostpointercapture={up}
	ondblclick={onreset}
	onkeydown={key}
></div>

<style>
	/* A wide, invisible grip centred on the panes' border; the line shows on hover and drag. */
	.splitter {
		position: absolute;
		top: 0;
		bottom: 0;
		z-index: 6;
		width: 9px;
		transform: translateX(-50%);
		cursor: col-resize;
		touch-action: none;
	}
	.splitter::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: 3px;
		width: 3px;
		border-radius: 2px;
		background: transparent;
		transition: background-color 120ms ease-out;
	}
	.splitter:hover::after {
		background: var(--line-hover);
		transition-delay: 150ms;
	}
	.splitter.dragging::after,
	.splitter:focus-visible::after {
		background: var(--text-bright);
		transition-delay: 0ms;
	}
	.splitter:focus-visible {
		outline: none;
	}
</style>
