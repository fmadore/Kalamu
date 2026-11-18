<script lang="ts">
	// A value not known yet, or that does not apply: a dash where it would stand, said in words
	// to a screen reader rather than read out as "dash". The one place the interface sets this
	// dash: it is typesetting, the same in every language, and never read out.
	import { messages } from '#lib/i18n/index.js';

	/** What the dash stands for, said to a screen reader: "Not known" unless said otherwise. */
	let { label }: { label?: string } = $props();

	const m = messages();
	/** The placeholder's glyph, an en dash, as typesetting. */
	const DASH = '–';
</script>

<span aria-hidden="true">{DASH}</span><span class="sr-only">{label ?? m.common.unknown}</span>
