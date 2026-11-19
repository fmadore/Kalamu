<script lang="ts">
	// A model's answer with its changes marked against the words it would replace: removed words
	// struck through, added ones underlined, each named for a screen reader; the answer as it is
	// while there are no changes to mark. Drawn inside its caller's own paragraph.
	import type { DiffSpan } from '#lib/bindings/DiffSpan.js';

	let {
		spans,
		answer,
		removed,
		added
	}: { spans: DiffSpan[]; answer: string; removed: string; added: string } = $props();
</script>

<!-- prettier-ignore -->
{#each spans as s, i (i)}{#if s.kind === 'removed'}<del><span class="sr-only">{removed}: </span>{s.text}</del>{:else if s.kind === 'added'}<ins><span class="sr-only">{added}: </span>{s.text}</ins>{:else}{s.text}{/if}{:else}{answer}{/each}

<style>
	del {
		color: var(--danger-text);
		text-decoration-thickness: 2px;
	}
	ins {
		text-decoration: none;
		background: var(--hand-wash);
		color: var(--text-bright);
		border-bottom: 2px solid var(--hand);
	}
</style>
