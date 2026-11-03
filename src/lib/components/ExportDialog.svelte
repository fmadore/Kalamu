<script lang="ts">
	// Exports the descriptions of the documents shown: Kalamu's own columns, or those of a saved
	// export profile, chosen, ordered and renamed here. A profile is kept for every project and
	// holds none of this one's values; what it may be is checked in Rust as it is edited.
	import type { ExportProfile } from '#lib/bindings/ExportProfile.js';
	import type { ProfileError } from '#lib/bindings/ProfileError.js';
	import type { ProfileStarter } from '#lib/bindings/ProfileStarter.js';
	import { FIELDS } from '#lib/described.js';
	import { describeError } from '#lib/errors.js';
	import { messages } from '#lib/i18n/index.js';
	import {
		DOCUMENT_VALUES,
		draftProfile,
		hasFixedTexts,
		keyValue,
		moved,
		valueKey
	} from '#lib/profiles.js';
	import { toast } from '#lib/state.svelte.js';
	import { api, ask, pickDescriptionsExport, pickPresetFile, pickSaveFile } from '#lib/tauri.js';
	import Icon from './Icon.svelte';

	let {
		projectId,
		collectionId,
		name,
		onclose
	}: {
		projectId: string;
		collectionId?: string;
		/** The project's or the collection's name, for the file. */
		name: string;
		onclose: () => void;
	} = $props();
	const m = messages();
	const p = m.projects.profiles;

	/** Kalamu's own export: every part, with where it came from. */
	const KALAMU = '';
	let profiles = $state<ExportProfile[]>([]);
	let chosen = $state(KALAMU);
	let draft = $state<ExportProfile | null>(null);
	let problem = $state<ProfileError | null>(null);
	let error = $state<string | null>(null);
	let busy = $state(false);

	const current = $derived(profiles.find((x) => x.id === chosen) ?? null);

	async function load() {
		try {
			profiles = await api.listExportProfiles();
		} catch (e) {
			error = describeError(e, m);
		}
	}
	void load();

	// Checked as it is edited, so the column at fault is named before saving.
	$effect(() => {
		if (!draft) return;
		const snapshot = $state.snapshot(draft) as ExportProfile;
		let stale = false;
		api.checkExportProfile(snapshot).then(
			(found) => {
				if (!stale) problem = found;
			},
			() => {}
		);
		return () => {
			stale = true;
		};
	});

	function problemText(found: ProfileError): string {
		if (found.kind === 'unnamed') return p.problems.unnamed(found.column);
		if (found.kind === 'bad_heading') return p.problems.bad_heading(found.column);
		if (found.kind === 'duplicate') return p.problems.duplicate(found.column, found.first);
		return p.problems[found.kind];
	}
	const faulty = $derived(problem && 'column' in problem ? problem.column - 1 : -1);

	async function act(work: () => Promise<void>) {
		busy = true;
		error = null;
		try {
			await work();
		} catch (e) {
			error = describeError(e, m);
		} finally {
			busy = false;
		}
	}

	async function start(starter: ProfileStarter) {
		await act(async () => {
			draft = draftProfile(p.newProfile, await api.profileStarter(starter));
		});
	}

	function edit() {
		if (current) draft = structuredClone($state.snapshot(current) as ExportProfile);
	}

	async function save() {
		if (!draft) return;
		const profile = $state.snapshot(draft) as ExportProfile;
		await act(async () => {
			const saved = await api.saveExportProfile(profile);
			await load();
			chosen = saved.id;
			draft = null;
			toast(p.saved);
		});
	}

	async function duplicate() {
		if (!current) return;
		const id = current.id;
		const copy = p.copyName(current.name);
		await act(async () => {
			const saved = await api.duplicateExportProfile(id, copy);
			await load();
			chosen = saved.id;
		});
	}

	async function remove() {
		if (!current || !(await ask(p.deleteConfirm(current.name)))) return;
		const id = current.id;
		await act(async () => {
			await api.deleteExportProfile(id);
			chosen = KALAMU;
			await load();
		});
	}

	async function share() {
		if (!current) return;
		const profile = current;
		const fixed = hasFixedTexts(profile) && (await ask(p.shareFixed));
		const path = await pickSaveFile(`${profile.name}.json`);
		if (!path) return;
		await act(async () => {
			await api.shareExportProfile(profile.id, fixed, path);
			toast(p.shared);
		});
	}

	async function importFile() {
		const path = await pickPresetFile();
		if (!path) return;
		await act(async () => {
			const imported = await api.importExportProfile(path);
			await load();
			chosen = imported.id;
			toast(p.imported(imported.name));
		});
	}

	async function exportNow() {
		const path = await pickDescriptionsExport(`${name} descriptions`);
		if (!path) return;
		await act(async () => {
			await api.exportDescriptions(projectId, collectionId ?? null, chosen || null, path);
			toast(m.projects.describe.exported);
			onclose();
		});
	}

	function setValue(i: number, key: string) {
		if (!draft) return;
		draft.columns[i].value = keyValue(key, draft.columns[i].value);
	}

	function move(i: number, by: number) {
		if (draft) draft.columns = moved(draft.columns, i, by);
	}

	function keys(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (draft) draft = null;
			else onclose();
		}
	}
</script>

<svelte:window onkeydown={keys} />

<div class="scrim" role="presentation">
	<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="export-title">
		<h2 id="export-title">{p.title}</h2>
		<p class="muted">{p.lede}</p>

		{#if !draft}
			<div class="pick">
				<label class="field">
					<span>{p.columns}</span>
					<select bind:value={chosen}>
						<option value={KALAMU}>{p.kalamu}</option>
						{#each profiles as profile (profile.id)}
							<option value={profile.id}>{profile.name}</option>
						{/each}
					</select>
				</label>
				{#if current}
					<div class="row">
						<button class="btn ghost sm" disabled={busy} onclick={edit}
							><Icon name="edit" size={14} />{p.edit}</button
						>
						<button class="btn ghost sm" disabled={busy} onclick={duplicate}>{p.duplicate}</button>
						<button class="btn ghost sm" disabled={busy} onclick={share}>{p.share}</button>
						<button class="btn ghost danger sm" disabled={busy} onclick={remove}>{p.delete}</button>
					</div>
				{/if}
				<div class="row">
					<button class="btn ghost sm" disabled={busy} onclick={() => start('every')}
						>{p.newProfile} · {p.starters.every}</button
					>
					<button class="btn ghost sm" disabled={busy} onclick={() => start('dublin_core')}
						>{p.newProfile} · {p.starters.dublin_core}</button
					>
					<button class="btn ghost sm" disabled={busy} onclick={importFile}>{p.import}</button>
				</div>
			</div>
		{:else}
			<div class="editor">
				<label class="field">
					<span>{p.name}</span>
					<input type="text" bind:value={draft.name} />
				</label>
				<table class="data-table">
					<thead>
						<tr><th></th><th>{p.heading}</th><th>{p.value}</th><th></th></tr>
					</thead>
					<tbody>
						{#each draft.columns as column, i (i)}
							<tr class:faulty={i === faulty}>
								<td class="mono">{i + 1}</td>
								<td>
									<input
										type="text"
										bind:value={column.heading}
										aria-label={`${p.heading} ${i + 1}`}
										aria-invalid={i === faulty}
									/>
								</td>
								<td>
									<select
										value={valueKey(column.value)}
										onchange={(e) => setValue(i, e.currentTarget.value)}
										aria-label={`${p.value} ${i + 1}`}
									>
										<optgroup label={p.groups.document}>
											{#each DOCUMENT_VALUES as v (v)}
												<option value={v}>{p.values[v]}</option>
											{/each}
										</optgroup>
										<optgroup label={p.groups.parts}>
											{#each FIELDS as f (f)}
												<option value={`part:${f}`}>{m.projects.describe.fields[f]}</option>
											{/each}
										</optgroup>
										<optgroup label={p.groups.from}>
											{#each FIELDS as f (f)}
												<option value={`from:${f}`}
													>{p.fromPart(m.projects.describe.fields[f])}</option
												>
											{/each}
										</optgroup>
										<optgroup label={p.groups.other}>
											<option value="fixed">{p.values.fixed}</option>
										</optgroup>
									</select>
									{#if column.value.kind === 'fixed'}
										<input
											type="text"
											class="fixed"
											bind:value={column.value.text}
											placeholder={p.fixedText}
											aria-label={`${p.fixedText} ${i + 1}`}
										/>
									{/if}
								</td>
								<td class="moves">
									<button
										class="btn ghost icon sm"
										disabled={i === 0}
										onclick={() => move(i, -1)}
										aria-label={p.moveUp(i + 1)}
										title={p.moveUp(i + 1)}><Icon name="up" size={14} /></button
									>
									<button
										class="btn ghost icon sm"
										disabled={i === draft.columns.length - 1}
										onclick={() => move(i, 1)}
										aria-label={p.moveDown(i + 1)}
										title={p.moveDown(i + 1)}><Icon name="down" size={14} /></button
									>
									<button
										class="btn ghost icon sm"
										onclick={() => draft?.columns.splice(i, 1)}
										aria-label={p.remove(i + 1)}
										title={p.remove(i + 1)}><Icon name="close" size={14} /></button
									>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
				<div>
					<button
						class="btn ghost sm"
						onclick={() => draft?.columns.push({ heading: '', value: { kind: 'title' } })}
						><Icon name="plus" size={14} />{p.addColumn}</button
					>
				</div>
				<div class="options">
					<label class="field">
						<span>{p.separator}</span>
						<input type="text" class="mono" bind:value={draft.list_separator} />
						<small class="muted">{p.separatorHint}</small>
					</label>
					<label class="field">
						<span>{p.missing}</span>
						<input type="text" class="mono" bind:value={draft.missing} />
						<small class="muted">{p.missingHint}</small>
					</label>
				</div>
				<label class="check">
					<input type="checkbox" bind:checked={draft.join_lines} />
					<span>{p.joinLines}</span>
				</label>
				{#if problem}<p class="notice" role="status">{problemText(problem)}</p>{/if}
			</div>
		{/if}

		{#if error}<p class="chip danger" role="alert">{error}</p>{/if}

		<div class="actions">
			{#if draft}
				<button class="btn" onclick={() => (draft = null)}>{p.cancel}</button>
				<button class="btn primary" disabled={busy || problem !== null} onclick={save}
					>{p.save}</button
				>
			{:else}
				<button class="btn" onclick={onclose}>{p.close}</button>
				<button class="btn primary" disabled={busy} onclick={exportNow}
					><Icon name="download" size={14} />{p.export}</button
				>
			{/if}
		</div>
	</div>
</div>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		background: var(--modal-scrim);
		display: grid;
		place-items: center;
		z-index: 50;
		padding: var(--space-4);
	}
	.dialog {
		width: min(48rem, 100%);
		max-height: 90vh;
		overflow: auto;
		padding: var(--space-5);
		display: grid;
		gap: var(--space-4);
		box-shadow:
			0 0 0 1px var(--line),
			var(--shadow-flyout);
		background: var(--surface-2);
		border-radius: var(--radius-overlay);
	}
	.pick,
	.editor {
		display: grid;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	td input,
	td select {
		width: 100%;
	}
	th:nth-child(2) {
		width: 45%;
	}
	td .fixed {
		margin-top: var(--space-1);
	}
	.moves {
		white-space: nowrap;
	}
	tr.faulty td {
		background: var(--danger-wash);
	}
	.options {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(14rem, 1fr));
		gap: var(--space-3);
	}
	.check {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-2);
	}
</style>
