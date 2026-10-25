<script lang="ts">
	// The interface's icons: Lucide, at one rendered stroke weight whatever the size. Each is
	// decorative; the control that holds it carries the accessible name. A new icon is one import
	// and one entry below.
	import ArrowDown from '@lucide/svelte/icons/arrow-down';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import Check from '@lucide/svelte/icons/check';
	import ChevronLeft from '@lucide/svelte/icons/chevron-left';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import FileText from '@lucide/svelte/icons/file-text';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import Mic from '@lucide/svelte/icons/mic';
	import Pause from '@lucide/svelte/icons/pause';
	import Pencil from '@lucide/svelte/icons/pencil';
	import Play from '@lucide/svelte/icons/play';
	import Plus from '@lucide/svelte/icons/plus';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import Settings from '@lucide/svelte/icons/settings';
	import TextSearch from '@lucide/svelte/icons/text-search';
	import X from '@lucide/svelte/icons/x';
	import ZoomIn from '@lucide/svelte/icons/zoom-in';
	import ZoomOut from '@lucide/svelte/icons/zoom-out';

	const ICONS = {
		back: ArrowLeft,
		down: ArrowDown,
		up: ArrowUp,
		check: Check,
		previous: ChevronLeft,
		chevron: ChevronRight,
		edit: Pencil,
		folder: FolderOpen,
		prompts: ScrollText,
		settings: Settings,
		close: X,
		pause: Pause,
		play: Play,
		plus: Plus,
		text_extraction: FileText,
		audio_transcription: Mic,
		analysis: TextSearch,
		'zoom-in': ZoomIn,
		'zoom-out': ZoomOut
	};
	/** Shapes read as solid marks rather than outlines. */
	const SOLID = new Set<keyof typeof ICONS>(['play', 'pause']);

	let { name, size = 16 }: { name: keyof typeof ICONS; size?: number } = $props();
	const Glyph = $derived(ICONS[name]);
</script>

<Glyph
	{size}
	strokeWidth={1.5}
	absoluteStrokeWidth
	fill={SOLID.has(name) ? 'currentColor' : 'none'}
	aria-hidden="true"
	class="icon"
/>

<style>
	:global(.icon) {
		display: block;
		flex: none;
	}
</style>
