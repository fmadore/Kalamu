<script lang="ts">
	// A value not known yet, or that does not apply: a dash where it would stand, said in words
	// to a screen reader rather than read out as "dash".
	import { messages } from '#lib/i18n/index.js';

	const m = messages();
</script>

<span aria-hidden="true">–</span><span class="sr-only">{m.common.unknown}</span>
