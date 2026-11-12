<script lang="ts">
	// The model field: a button showing the chosen model with its tier's bars, opening a list of
	// the models under their tiers, each with its one-line description, so a model is read about
	// before it is chosen. A select-only combobox: the arrow keys, Home and End move through the
	// list, a letter jumps to the next model starting with it, Enter or Space chooses, Escape or
	// a click elsewhere closes, and the focus goes back to the button. In the top layer, under the
	// button or over it when the window has no room below.
	import Icon from './Icon.svelte';
	import TierMark from './TierMark.svelte';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import { modelDescription } from '#lib/catalogText.js';
	import { giveFocusBack } from '#lib/keys.js';
	import { tierOf, type ModelGroup } from '#lib/tiers.js';

	let {
		label,
		groups,
		chosen,
		headingOf,
		nameOf,
		providerOf = () => null,
		unavailable = () => null,
		onchoose
	}: {
		label: string;
		groups: ModelGroup[];
		/** The model chosen, if the catalogue lists it. */
		chosen: ModelInfo | undefined;
		headingOf: (group: ModelGroup) => string;
		/** A model's name as listed, with what follows it, such as "(preview)". */
		nameOf: (model: ModelInfo) => string;
		/** The provider written beside a model's name, where several providers are listed. */
		providerOf?: (model: ModelInfo) => string | null;
		/** Why a model cannot be chosen (its provider has no key), or nothing. */
		unavailable?: (model: ModelInfo) => string | null;
		onchoose: (model: ModelInfo) => void;
	} = $props();

	const id = $props.id();
	const keyOf = (model: ModelInfo) => `${model.provider}/${model.id}`;

	let trigger = $state<HTMLButtonElement | null>(null);
	let list = $state<HTMLElement | null>(null);
	let open = $state(false);

	const options = () =>
		list
			? [...list.querySelectorAll<HTMLElement>('[role="option"]:not([aria-disabled="true"])')]
			: [];

	/** Under the button, as wide as it at least, kept inside the window. */
	function place() {
		if (!trigger || !list) return;
		const at = trigger.getBoundingClientRect();
		const width = Math.min(Math.max(at.width, 320), innerWidth - 16);
		const below = innerHeight - at.bottom - 12;
		const above = at.top - 12;
		const down = below >= 280 || below >= above;
		const room = Math.max(160, Math.min(448, down ? below : above));
		list.style.width = `${width}px`;
		list.style.maxHeight = `${room}px`;
		list.style.left = `${Math.max(8, Math.min(at.left, innerWidth - width - 8))}px`;
		list.style.top = down ? `${at.bottom + 4}px` : '';
		list.style.bottom = down ? '' : `${innerHeight - at.top + 4}px`;
	}

	function toggled(e: ToggleEvent) {
		open = e.newState === 'open';
		if (!open) {
			giveFocusBack(trigger, list);
			return;
		}
		place();
		const all = options();
		const start = all.find((o) => o.getAttribute('aria-selected') === 'true') ?? all[0];
		start?.focus({ preventScroll: true });
		start?.scrollIntoView({ block: 'center' });
	}

	function choose(model: ModelInfo) {
		if (unavailable(model)) return;
		onchoose(model);
		list?.hidePopover();
	}

	function triggerKeys(e: KeyboardEvent) {
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			list?.showPopover();
		}
	}

	function listKeys(e: KeyboardEvent) {
		const all = options();
		const at = all.indexOf(document.activeElement as HTMLElement);
		let to = -1;
		if (e.key === 'ArrowDown') to = Math.min(at + 1, all.length - 1);
		else if (e.key === 'ArrowUp') to = Math.max(at - 1, 0);
		else if (e.key === 'Home') to = 0;
		else if (e.key === 'End') to = all.length - 1;
		else if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			(document.activeElement as HTMLElement | null)?.click();
			return;
		} else if (e.key === 'Tab') {
			list?.hidePopover();
			return;
		} else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
			// The next model whose name starts with the letter typed, after the one in focus.
			const letter = e.key.toLocaleLowerCase();
			const after = [...all.slice(at + 1), ...all.slice(0, at + 1)];
			to = all.indexOf(
				after.find((o) => o.dataset.name?.toLocaleLowerCase().startsWith(letter)) ?? all[at]
			);
		} else return;
		e.preventDefault();
		all[to]?.focus();
		all[to]?.scrollIntoView({ block: 'nearest' });
	}
</script>

<div class="field">
	<span id="{id}-label">{label}</span>
	<button
		bind:this={trigger}
		id="{id}-button"
		type="button"
		role="combobox"
		class="trigger"
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-controls="{id}-list"
		aria-labelledby="{id}-label {id}-button"
		popovertarget="{id}-list"
		onkeydown={triggerKeys}
	>
		{#if chosen}
			<TierMark tier={tierOf(chosen)} />
			<span class="value">{nameOf(chosen)}</span>
			{#if providerOf(chosen)}<span class="provider">{providerOf(chosen)}</span>{/if}
		{:else}
			<span class="value"></span>
		{/if}
		<span class="chevron" aria-hidden="true"><Icon name="expand" size={16} /></span>
	</button>
	<div
		bind:this={list}
		id="{id}-list"
		class="list"
		popover="auto"
		role="listbox"
		tabindex="-1"
		aria-labelledby="{id}-label"
		ontoggle={toggled}
		onkeydown={listKeys}
	>
		{#each groups as group (group.key)}
			<div class="group" role="group" aria-labelledby="{id}-{group.key}">
				<div class="heading" id="{id}-{group.key}">
					{#if group.tier}<TierMark tier={group.tier} />{/if}{headingOf(group)}
				</div>
				{#each group.models as model (keyOf(model))}
					{@const why = unavailable(model)}
					{@const selected = chosen !== undefined && keyOf(model) === keyOf(chosen)}
					<!-- The list's keys are handled by the list itself, above. -->
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<div
						class="option"
						role="option"
						tabindex="-1"
						aria-selected={selected}
						aria-disabled={why !== null}
						data-name={model.name}
						onclick={() => choose(model)}
					>
						<span class="name"
							>{nameOf(model)}{#if providerOf(model)}<span class="provider"
									>{providerOf(model)}</span
								>{/if}</span
						>
						{#if selected}<span class="check" aria-hidden="true"
								><Icon name="check" size={14} /></span
							>{/if}
						{#if why}
							<span class="desc">{why}</span>
						{:else if model.description}
							<span class="desc">{modelDescription(model)}</span>
						{/if}
					</div>
				{/each}
			</div>
		{/each}
	</div>
</div>

<style>
	/* Drawn as the selects beside it are: the same border, height, type and focus ring. */
	.trigger {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
		min-width: 0;
		min-height: var(--control-md);
		padding: var(--space-1) var(--space-2) var(--space-1) var(--space-3);
		border: 1px solid var(--control-line);
		border-radius: var(--radius-control);
		background: var(--surface-2);
		color: var(--text-bright);
		font: inherit;
		font-size: var(--type-sm);
		text-align: start;
		cursor: pointer;
		transition:
			border-color 120ms ease-out,
			box-shadow 120ms ease-out;
	}
	.trigger:hover {
		border-color: var(--control-line-hover);
	}
	.trigger:focus-visible,
	.trigger[aria-expanded='true'] {
		outline: none;
		border-color: var(--text-bright);
		box-shadow: 0 0 0 3px var(--wash);
	}
	.trigger :global(.tier) {
		color: var(--text-secondary);
	}
	/* The provider takes only the room the name leaves (its basis is nothing), so the name stays
	   whole as long as it fits. */
	.value {
		flex: 0 1 auto;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.trigger .provider {
		flex: 1 1 0;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--text-muted);
	}
	.chevron {
		display: grid;
		margin-left: auto;
		color: var(--text-secondary);
	}

	/* In the top layer, placed by `place`. */
	.list {
		position: fixed;
		inset: auto;
		margin: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: var(--space-1);
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-body);
	}
	.group + .group {
		margin-top: var(--space-1);
		padding-top: var(--space-1);
		border-top: 1px solid var(--line-soft);
	}
	/* Each tier's heading stays on top while its models scroll under it. */
	.heading {
		position: sticky;
		top: calc(-1 * var(--space-1));
		z-index: 1;
		background: var(--surface-2);
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-2) var(--space-1);
		font-size: var(--type-xs);
		font-weight: 500;
		color: var(--text-muted);
	}
	.option {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		column-gap: var(--space-2);
		row-gap: 0.125rem;
		padding: var(--space-2);
		border-radius: var(--radius-control);
		cursor: pointer;
		outline: none;
	}
	.option:hover,
	.option:focus-visible,
	.option:focus {
		background: var(--wash);
	}
	.option:focus-visible {
		box-shadow: inset 0 0 0 1.5px var(--text-bright);
	}
	.option[aria-disabled='true'] {
		cursor: not-allowed;
		opacity: 0.5;
	}
	.option[aria-disabled='true']:hover {
		background: none;
	}
	.name {
		font-weight: 500;
		color: var(--text-bright);
	}
	.name .provider {
		margin-left: var(--space-2);
		font-weight: 400;
		color: var(--text-muted);
	}
	.check {
		display: grid;
		align-self: center;
		grid-row: span 2;
		color: var(--text-bright);
	}
	/* Two lines at most: the whole description is under the fields once the model is chosen. */
	.desc {
		grid-column: 1;
		display: -webkit-box;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		overflow: hidden;
		font-size: var(--type-xs);
		line-height: 1.4;
		color: var(--text-muted);
	}
</style>
