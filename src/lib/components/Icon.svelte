<script lang="ts">
	// The interface's icons: Lucide, at one rendered stroke weight whatever the size. Each is
	// decorative; the control that holds it carries the accessible name. A new icon is one import
	// and one entry below.
	import ArrowDown from '@lucide/svelte/icons/arrow-down';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import FilePlus from '@lucide/svelte/icons/file-plus';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import PanelLeftClose from '@lucide/svelte/icons/panel-left-close';
	import PanelLeftOpen from '@lucide/svelte/icons/panel-left-open';
	import Pause from '@lucide/svelte/icons/pause';
	import Play from '@lucide/svelte/icons/play';
	import Plus from '@lucide/svelte/icons/plus';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import Search from '@lucide/svelte/icons/search';
	import Settings from '@lucide/svelte/icons/settings';
	import Star from '@lucide/svelte/icons/star';
	import X from '@lucide/svelte/icons/x';
	import ZoomIn from '@lucide/svelte/icons/zoom-in';
	import ZoomOut from '@lucide/svelte/icons/zoom-out';

	const ICONS = {
		back: ArrowLeft,
		down: ArrowDown,
		up: ArrowUp,
		chevron: ChevronRight,
		collapse: PanelLeftClose,
		expand: PanelLeftOpen,
		jobs: ListChecks,
		new: FilePlus,
		prompts: ScrollText,
		settings: Settings,
		close: X,
		pause: Pause,
		play: Play,
		plus: Plus,
		search: Search,
		star: Star,
		'zoom-in': ZoomIn,
		'zoom-out': ZoomOut
	};
	/** Shapes read as solid marks rather than outlines. */
	const SOLID = new Set<keyof typeof ICONS>(['play', 'pause']);

	let {
		name,
		size = 16,
		filled = false
	}: { name: keyof typeof ICONS; size?: number; filled?: boolean } = $props();
	const Glyph = $derived(ICONS[name]);
</script>

<Glyph
	{size}
	strokeWidth={1.5}
	absoluteStrokeWidth
	fill={filled || SOLID.has(name) ? 'currentColor' : 'none'}
	aria-hidden="true"
	class="icon"
/>

<style>
	:global(.icon) {
		display: block;
		flex: none;
	}
</style>
