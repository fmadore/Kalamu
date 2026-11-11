<script lang="ts">
	import { onMount } from 'svelte';
	import { MediaQuery } from 'svelte/reactivity';
	import AllJobs from '#lib/components/AllJobs.svelte';
	import AnalysisReview from '#lib/components/AnalysisReview.svelte';
	import ContentsReview from '#lib/components/ContentsReview.svelte';
	import CloseDialog from '#lib/components/CloseDialog.svelte';
	import JobsView from '#lib/components/JobsView.svelte';
	import NewJob from '#lib/components/NewJob.svelte';
	import ProjectsView from '#lib/components/ProjectsView.svelte';
	import PromptsView from '#lib/components/PromptsView.svelte';
	import ReviewView from '#lib/components/ReviewView.svelte';
	import SettingsView from '#lib/components/SettingsView.svelte';
	import GlossaryView from '#lib/components/GlossaryView.svelte';
	import WorkflowsView from '#lib/components/WorkflowsView.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import { isBack, primary } from '#lib/platform.js';
	import { api, on, onDragging } from '#lib/tauri.js';
	import {
		app,
		applyLanguage,
		applyProgress,
		back,
		dismissError,
		go,
		refreshJobs,
		refreshKeys,
		refreshOffline,
		refreshRelease,
		refreshWaiting,
		shortcutsHeld,
		takeToastAction,
		toast,
		type View
	} from '#lib/state.svelte.js';
	import { keep, recall } from '#lib/stored.js';

	// Read again when the language changes; the screens below are drawn anew with it.
	const m = $derived.by(() => (app.locale, messages()));
	let loadError = $state<string | null>(null);
	let startView: View | null = null;

	// The theme is always set on the page, so the dark tokens are written once (`app.css`):
	// the system's until the settings are read, then the one they ask for, following the
	// system's as it changes when they say so.
	const systemDark = new MediaQuery('prefers-color-scheme: dark');
	$effect(() => {
		const theme = app.settings?.theme ?? 'system';
		const dark = theme === 'dark' || (theme !== 'light' && systemDark.current);
		document.documentElement.setAttribute('data-theme', dark ? 'dark' : 'light');
	});

	// The job list folds to a rail of icons, leaving the room to the work; remembered on this
	// computer, and unfolded when storage is unavailable.
	const SIDEBAR_KEY = 'kalamu.sidebar.collapsed';
	let collapsed = $state(recall(SIDEBAR_KEY, (v) => v === 1, false));
	$effect(() => keep(SIDEBAR_KEY, collapsed ? 1 : 0));
	// A narrow window sets the sidebar above the work, folded to one row until it is unfolded
	// there; the choice made in a wide window is kept for when the window widens again.
	const narrow = new MediaQuery('max-width: 760px');
	let narrowOpen = $state(false);
	const folded = $derived(narrow.current ? !narrowOpen : collapsed);
	function fold(to: boolean) {
		if (narrow.current) narrowOpen = !to;
		else collapsed = to;
	}

	/** Whether a key is typed into a field, where the arrows and letters are the field's. */
	const typing = (e: Event) =>
		(e.target as HTMLElement | null)?.closest?.('input, textarea, select, [contenteditable]');

	// The window's shortcuts. A screen's own come first (the review's are on the document) and
	// take the key, as a review takes Alt and the arrows to turn its pages.
	function onKey(e: KeyboardEvent) {
		if (e.defaultPrevented || shortcutsHeld()) return;
		if (isBack(e)) {
			if (typing(e)) return;
			e.preventDefault();
			back();
			return;
		}
		if (!primary(e) || e.altKey || e.shiftKey) return;
		const key = e.key.toLowerCase();
		if (key === 'n') {
			e.preventDefault();
			go({ name: 'new' });
		} else if (key === 'b') {
			e.preventDefault();
			fold(!folded);
		} else if (key === ',') {
			// On a Mac the application menu's Settings… takes the key first (`openSettings`).
			e.preventDefault();
			go({ name: 'settings' });
		} else if (key === 'f') {
			// The screen's filter or search field, marked `data-find`.
			const field = document.querySelector<HTMLInputElement>('main [data-find]');
			if (!field) return;
			e.preventDefault();
			field.focus();
			field.select();
		}
	}

	/** The mouse's back button goes back, as in a browser. */
	function onMouseUp(e: MouseEvent) {
		if (e.button !== 3 || shortcutsHeld()) return;
		e.preventDefault();
		back();
	}

	onMount(() => {
		const unlisten: (() => void)[] = [];
		void (async () => {
			// Sample data for looking at the interface in a browser; development builds only.
			if (
				import.meta.env.DEV &&
				location.search.includes('mock') &&
				!('__TAURI_INTERNALS__' in window)
			) {
				const mock = await import('#lib/dev/mock.js');
				mock.installMock();
				startView = mock.mockView();
				app.filesOver = mock.mockDragging();
			}
			try {
				const [info, catalog, rules, settings] = await Promise.all([
					api.appInfo(),
					api.getCatalog(),
					api.taskRules(),
					api.getSettings()
				]);
				app.info = info;
				app.catalog = catalog;
				app.rules = rules;
				app.settings = settings;
				applyLanguage();
				// Before the first reading, so a graphics processor found meanwhile is not missed.
				unlisten.push(await on.offlineChanged(() => void refreshOffline()));
				// The model list refreshed at start-up may answer after the first reading.
				unlisten.push(
					await on.catalogChanged(() => {
						void (async () => {
							app.catalog = await api.getCatalog();
							app.rules = await api.taskRules();
							await refreshRelease();
						})().catch(() => undefined);
					})
				);
				await Promise.all([
					refreshKeys(),
					refreshJobs(),
					refreshOffline(),
					refreshWaiting(),
					refreshRelease()
				]);
				// Settings that could not be read were moved aside: said once, until dismissed.
				if (info.settings_set_aside) {
					toast(m.settings.setAside(info.settings_set_aside), { kind: 'error' });
				}
				// Jobs cut short by the last close: show them first.
				if (startView) go(startView);
				else if (info.interrupted.length > 0) go({ name: 'jobs', jobId: info.interrupted[0] });
				else if (!app.keys.some((k) => k.saved)) go({ name: 'settings' });
			} catch (e) {
				loadError = describeError(e, m);
			}
			unlisten.push(await on.progress(applyProgress));
			unlisten.push(await onDragging((over) => (app.filesOver = over)));
			unlisten.push(await on.closeRequested(() => (app.closeRequested = true)));
			unlisten.push(
				await on.openSettings(() => {
					if (!shortcutsHeld()) go({ name: 'settings' });
				})
			);
		})();
		return () => unlisten.forEach((u) => u());
	});
</script>

<svelte:window onkeydown={onKey} onmouseup={onMouseUp} />

<!-- Every screen takes its text once, when drawn: a change of language draws them again. -->
{#key app.locale}
	<div class="shell" class:collapsed={folded}>
		<Sidebar bind:collapsed={() => folded, fold} />

		<main>
			{#if loadError}
				<div class="sheet"><p class="chip danger">{loadError}</p></div>
			{:else if !app.catalog || !app.settings}
				<div class="sheet"><p class="muted">{m.common.loading}</p></div>
			{:else if app.view.name === 'new'}
				<!-- Opened afresh each time it is asked for: with the form left, or blank. -->
				{#key app.view}
					<NewJob opening={app.view} />
				{/key}
			{:else if app.view.name === 'jobs'}
				<JobsView jobId={app.view.jobId} />
			{:else if app.view.name === 'allJobs'}
				<AllJobs />
			{:else if app.view.name === 'review'}
				{@const { jobId, sourceId, index, find } = app.view}
				{@const task = app.jobs.find((j) => j.id === jobId)?.task}
				<!-- Another source is another review, loaded afresh; another page of it is not. -->
				{#key `${jobId}/${sourceId}`}
					{#if task === 'analysis'}
						<AnalysisReview {jobId} {sourceId} {find} />
					{:else if task === 'contents'}
						<ContentsReview {jobId} {sourceId} start={index} {find} />
					{:else}
						<ReviewView {jobId} {sourceId} start={index} {find} />
					{/if}
				{/key}
			{:else if app.view.name === 'projects'}
				<ProjectsView
					projectId={app.view.projectId}
					collectionId={app.view.collectionId}
					documentId={app.view.documentId}
					pane={app.view.pane}
					selection={app.view.selection}
				/>
			{:else if app.view.name === 'workflows'}
				<WorkflowsView
					workflowId={app.view.workflowId}
					example={app.view.example}
					runId={app.view.runId}
					paths={app.view.paths}
					sources={app.view.sources}
					project={app.view.project}
				/>
			{:else if app.view.name === 'prompts'}
				<PromptsView presetId={app.view.presetId} />
			{:else if app.view.name === 'settings'}
				<SettingsView />
			{:else if app.view.name === 'glossary'}
				<GlossaryView opening={app.view} />
			{/if}
		</main>

		<!-- Files dragged over the window, over the screen that takes them. -->
		{#if app.filesOver && app.dropTarget}
			<div class="drop-target" aria-hidden="true">
				<span class="drop-label"><Icon name="add-file" size={16} />{app.dropTarget.label}</span>
			</div>
		{/if}
	</div>

	{#if app.closeRequested}
		<CloseDialog />
	{/if}
{/key}

<!-- Mounted empty, so a message put into them is read out (VoiceOver misses a live region that
     arrives with its text): a failure at once, and kept until dismissed; a confirmation politely. -->
<div class="toasts">
	<div role="alert">
		{#if app.toastError}
			{#key app.toastError.n}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="toast error"
					onkeydown={(e) => {
						if (e.key === 'Escape') dismissError();
					}}
				>
					<span class="dot" aria-hidden="true"></span>
					<span class="message">{app.toastError.text}</span>
					<button
						class="dismiss"
						aria-label={m.common.dismissMessage}
						title={m.common.dismissMessage}
						onclick={dismissError}><Icon name="close" size={14} /></button
					>
				</div>
			{/key}
		{/if}
	</div>
	<div role="status">
		{#if app.toast}
			<div class="toast" class:with-action={app.toastAction}>
				{app.toast}{#if app.toastAction}<button class="btn ghost sm" onclick={takeToastAction}
						>{app.toastAction.label}</button
					>{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.shell {
		display: grid;
		grid-template-columns: 18rem minmax(0, 1fr);
		grid-template-rows: minmax(0, 1fr);
		height: 100vh;
	}
	.shell.collapsed {
		grid-template-columns: 3.25rem minmax(0, 1fr);
	}
	/* Positioned, so what is placed absolutely inside a screen (the links' screen-reader text)
	   scrolls with it instead of stretching the window below the shell. */
	main {
		grid-column: -2;
		grid-row: -2;
		position: relative;
		overflow: auto;
		min-width: 0;
		background: var(--surface-0);
	}
	/* The messages, bottom right over the work: a failure kept above a passing confirmation. */
	.toasts {
		position: fixed;
		bottom: var(--space-5);
		right: var(--space-5);
		z-index: 40;
		display: grid;
		justify-items: end;
		gap: var(--space-2);
		max-width: min(28rem, calc(100vw - 2 * var(--space-5)));
	}
	.toast {
		padding: var(--space-3) var(--space-4);
		background: var(--ink);
		color: var(--on-ink);
		border-radius: var(--radius-control);
		box-shadow: var(--shadow-flyout);
		font-size: var(--type-sm);
	}
	/* A failure: the same ink, marked by the danger dot, with its own way to put it away. */
	.toast.error {
		display: flex;
		align-items: baseline;
		gap: var(--space-3);
		padding-right: var(--space-2);
	}
	.dot {
		flex: none;
		width: 6px;
		height: 6px;
		border-radius: 3px;
		background: var(--on-ink-danger);
		transform: translateY(-0.1em);
	}
	.message {
		flex: 1;
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.dismiss {
		flex: none;
		display: grid;
		place-items: center;
		align-self: start;
		width: var(--control-sm);
		height: var(--control-sm);
		margin: calc(-1 * var(--space-1)) 0;
		padding: 0;
		border: 0;
		border-radius: var(--radius-tag);
		background: transparent;
		color: var(--on-ink-muted);
		cursor: pointer;
		transition: background-color 120ms ease-out;
	}
	.dismiss:hover {
		color: var(--on-ink);
		background: color-mix(in srgb, var(--on-ink) 14%, transparent);
	}
	.dismiss:focus-visible {
		color: var(--on-ink);
		outline-color: var(--on-ink);
		outline-offset: -2px;
	}
	/* A confirmation's action, a ghost button in the toast's own colours. */
	.toast.with-action {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.with-action .btn {
		flex: none;
		margin: calc(-1 * var(--space-1)) calc(-1 * var(--space-2)) calc(-1 * var(--space-1)) auto;
		color: var(--on-ink);
	}
	.with-action .btn:hover {
		background: color-mix(in srgb, var(--on-ink) 14%, transparent);
	}
	.with-action .btn:focus-visible {
		outline-color: var(--on-ink);
	}
	/* Over the work area, in the cell `main` takes: a dashed line, as a slot that adds
	   something, and what dropping does. The window still takes the drop. */
	.drop-target {
		grid-column: -2;
		grid-row: -2;
		display: grid;
		place-items: center;
		margin: var(--space-3);
		border: 2px dashed var(--line-strong);
		border-radius: var(--radius-card);
		background: var(--wash);
		pointer-events: none;
		z-index: 30;
	}
	.drop-label {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		border-radius: var(--radius-control);
		background: var(--surface-2);
		box-shadow: var(--shadow-raised);
		color: var(--text-bright);
		font-size: var(--type-sm);
		font-weight: 500;
	}
	/* A narrow window: the list sits above the work, short enough to leave it room. */
	@media (max-width: 760px) {
		.shell {
			grid-template-columns: minmax(0, 1fr);
			grid-template-rows: auto minmax(0, 1fr);
		}
		.shell.collapsed {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
