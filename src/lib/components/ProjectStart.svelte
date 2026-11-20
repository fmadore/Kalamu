<script lang="ts">
	// What a project with no documents says in place of its table: add documents first, with the
	// buttons above it, and nothing else to fill in. Quiet text, nothing to dismiss: it goes by
	// itself once the project holds a document.
	import { messages } from '#lib/i18n/index.js';

	const m = messages();
	const s = $derived(m.projects.start);
</script>

<section class="start" aria-labelledby="project-start">
	<h3 id="project-start">{s.title}</h3>
	<p>{s.first(m.projects.addFiles, m.projects.addFolder, m.projects.attachJob)}</p>
	<p class="muted">{s.rest}</p>
</section>

<style>
	.start {
		display: grid;
		gap: var(--space-2);
		max-width: 75ch;
		padding-top: var(--space-2);
	}
	h3 {
		margin: 0;
		font-size: var(--type-md);
		font-weight: 600;
		color: var(--text-bright);
	}
	p {
		margin: 0;
		color: var(--text-secondary);
	}
	p.muted {
		color: var(--text-muted);
	}
</style>
