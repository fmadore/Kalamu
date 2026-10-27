<script lang="ts">
	// The YouTube videos of a new job, added by pasting their links. Each link is checked and
	// named after the video's title; whether Gemini may read it (an unlisted video) and how long
	// it is are known only once the job runs, so a recognised link is not marked as fine.
	import Icon from './Icon.svelte';
	import type { InspectedFile } from '#lib/bindings/InspectedFile.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { api, openUrl } from '#lib/tauri.js';
	import { splitLinks } from '#lib/youtube.js';

	let {
		links = $bindable(),
		onadd,
		onerror
	}: {
		links: InspectedFile[];
		/** The links just added, to name the job after the first video. */
		onadd?: (added: InspectedFile[]) => void;
		onerror: (message: string) => void;
	} = $props();

	const m = messages();
	let pasted = $state('');
	let checking = $state(false);
	/** Said to screen readers once links are checked. */
	let status = $state('');

	async function add(e: SubmitEvent) {
		e.preventDefault();
		const known = new Set(links.map((l) => l.source?.path ?? l.path));
		const fresh = splitLinks(pasted);
		if (fresh.length === 0) return;
		checking = true;
		try {
			const checked = await api.inspectLinks(fresh);
			// The same video pasted twice, under any form of its link, is added once.
			const added = checked.filter((l) => {
				const key = l.source?.path ?? l.path;
				if (known.has(key)) return false;
				known.add(key);
				return true;
			});
			links = [...links, ...added];
			pasted = '';
			const refused = added.filter((l) => l.error).length;
			status = m.newJob.linksAdded(added.length - refused, refused);
			onadd?.(added);
		} catch (err) {
			onerror(describeError(err, m));
		} finally {
			checking = false;
		}
	}

	/** The address without its scheme, as it is shown. */
	const short = (url: string) => url.replace(/^https?:\/\/(www\.)?/, '');
</script>

<section class="step block" aria-labelledby="links-title">
	<h2 id="links-title">{m.newJob.linksTitle}</h2>
	<form class="paste" onsubmit={add}>
		<label class="sr-only" for="links-field">{m.newJob.linkField}</label>
		<textarea
			id="links-field"
			rows="2"
			bind:value={pasted}
			placeholder={m.newJob.linkPlaceholder}
			spellcheck="false"
			aria-describedby="links-hint"
			onkeydown={(e) => {
				// Enter adds, as in a one-line field; Shift Enter starts another line.
				if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey) {
					e.preventDefault();
					e.currentTarget.form?.requestSubmit();
				}
			}}></textarea>
		<button class="btn" type="submit" disabled={checking || !pasted.trim()}
			><Icon name="plus" size={14} />{checking ? m.newJob.inspecting : m.newJob.addLinks}</button
		>
	</form>
	<p class="hint" id="links-hint">{m.newJob.linksHint}</p>
	<p class="sr-only" aria-live="polite">{status}</p>
	{#if links.length > 0}
		<ul class="files">
			{#each links as link, i (link.path)}
				{@const name = link.source?.title ?? short(link.source?.path ?? link.path)}
				<li class:failed={!!link.error}>
					<div class="file">
						<span class="video">
							<span class="file-name" class:mono={!link.source?.title}>{name}</span>
							{#if link.source?.title}
								<span class="address mono">{short(link.source.path)}</span>
							{/if}
						</span>
						{#if link.source}
							<button
								class="btn sm ghost"
								aria-label={m.newJob.openVideoNamed(name)}
								onclick={() => link.source && openUrl(link.source.path)}
								>{m.newJob.openVideo}</button
							>
						{:else}
							<span></span>
						{/if}
						<button
							class="btn sm icon ghost"
							aria-label="{m.newJob.remove} {name}"
							title={m.newJob.remove}
							onclick={() => (links = links.filter((_, j) => j !== i))}
							><Icon name="close" size={14} /></button
						>
					</div>
					{#if link.error}
						<!-- The link is already in the row, so the sentence alone says what is wrong. -->
						<p class="problem danger">{describeError({ id: link.error.id }, m)}</p>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
	<details class="tips">
		<summary><Icon name="chevron" size={14} />{m.tips.title}</summary>
		<ul>
			<li>{m.tips.youtubePublic}</li>
			<li>{m.tips.youtubePreview}</li>
			<li>{m.tips.results}</li>
		</ul>
	</details>
</section>

<style>
	.block {
		display: grid;
		gap: var(--space-3);
	}
	.paste {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		gap: var(--space-2);
		align-items: start;
	}
	.paste textarea {
		font-family: var(--font-mono);
		font-size: var(--type-sm);
	}
	.hint {
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 60ch;
	}
	/* The videos as a table, like the files of other jobs: link, open, remove. */
	.files {
		list-style: none;
		padding: 0;
		margin: 0;
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		overflow: hidden;
	}
	.files li {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-2) var(--space-2) var(--space-4);
		border-top: 1px solid var(--line-soft);
	}
	.files li:first-child {
		border-top: 0;
	}
	.file {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto;
		gap: var(--space-4);
		align-items: center;
	}
	/* The video's title, its address beneath in mono. */
	.video {
		display: grid;
		gap: 1px;
		min-width: 0;
		padding-left: calc(6px + var(--space-3));
		position: relative;
	}
	.file-name {
		overflow-wrap: anywhere;
		color: var(--text-bright);
		font-size: var(--type-sm);
	}
	/* Recognised, not checked: grey until the job reads the video; red when refused. */
	.video::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0.55em;
		width: 6px;
		height: 6px;
		border-radius: 3px;
		background: var(--line-hover);
	}
	.address {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--type-xs);
		color: var(--text-muted);
	}
	.failed .video::before {
		background: var(--danger);
	}
	.problem {
		margin: 0 var(--space-2) var(--space-1) 1.125rem;
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-control);
		background: var(--danger-wash);
		color: var(--danger-text);
		font-size: var(--type-xs);
	}
	.tips summary {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		min-height: var(--control-sm);
		list-style: none;
		color: var(--text-secondary);
		font-size: var(--type-xs);
		font-weight: 500;
		cursor: pointer;
	}
	.tips summary::-webkit-details-marker {
		display: none;
	}
	.tips summary:hover {
		color: var(--text-bright);
	}
	.tips summary :global(.icon) {
		transition: transform 120ms ease-out;
	}
	.tips[open] summary :global(.icon) {
		transform: rotate(90deg);
	}
	.tips ul {
		margin: var(--space-1) 0 0;
		padding-left: 1.25rem;
		display: grid;
		gap: var(--space-1);
		font-size: var(--type-xs);
		color: var(--text-muted);
		max-width: 48rem;
	}
</style>
