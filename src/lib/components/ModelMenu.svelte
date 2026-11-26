<script lang="ts">
	// The model field: a button showing the chosen model with its tier's bars, opening a list of
	// the models under their tiers, each with its one-line summary, so a model is read about
	// before it is chosen. A select-only combobox whose list opens with a search field at its top,
	// in focus: typing narrows the list (every word, in a model's name, its id or its provider's
	// name, whatever the case and accents), a heading left with no model goes, and a line says when
	// nothing is found. The arrow keys move through the models shown (the field keeps the focus and
	// names the model it is on, `aria-activedescendant`), Enter chooses, Escape empties the field or,
	// empty, closes, as a click elsewhere does, and the focus goes back to the button. The field is
	// emptied whenever the list closes. In the top layer, under the button or over it when the
	// window has no room below, following it when the page scrolls.
	import Icon from './Icon.svelte';
	import TierMark from './TierMark.svelte';
	import type { ModelInfo } from '#lib/bindings/ModelInfo.js';
	import { modelSummary } from '#lib/catalogText.js';
	import { messages } from '#lib/i18n/index.js';
	import { giveFocusBack } from '#lib/keys.js';
	import { findModels } from '#lib/modelSearch.js';
	import { tierOf, type ModelGroup } from '#lib/tiers.js';

	let {
		label,
		groups,
		chosen,
		headingOf,
		nameOf,
		providerOf = () => null,
		providerName = (model) => providerOf(model) ?? '',
		summaryOf = (model) => (model.summary || model.description ? modelSummary(model) : ''),
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
		/** The provider's name a search finds a model by, whether or not it is written. */
		providerName?: (model: ModelInfo) => string;
		/** A model's one line under its name; empty for none. */
		summaryOf?: (model: ModelInfo) => string;
		/** Why a model cannot be chosen (its provider has no key), or nothing. */
		unavailable?: (model: ModelInfo) => string | null;
		onchoose: (model: ModelInfo) => void;
	} = $props();

	const m = messages();
	const id = $props.id();
	const keyOf = (model: ModelInfo) => `${model.provider}/${model.id}`;

	let trigger = $state<HTMLButtonElement | null>(null);
	let popup = $state<HTMLElement | null>(null);
	let list = $state<HTMLElement | null>(null);
	let field = $state<HTMLInputElement | null>(null);
	let open = $state(false);
	let query = $state('');
	/** The model the keys are on, by its key. */
	let active = $state<string | null>(null);

	const shown = $derived(findModels(groups, query, providerName));
	/** The models shown that can be chosen, in the list's order: what the arrow keys move through. */
	const choosable = $derived(shown.flatMap((g) => g.models).filter((model) => !unavailable(model)));
	/** Each model shown, numbered for its element's id. */
	const indexOf = $derived(
		new Map(shown.flatMap((g) => g.models).map((model, i) => [keyOf(model), i]))
	);
	const optionId = (key: string) => `${id}-option-${indexOf.get(key) ?? 0}`;
	const activeId = $derived(active !== null && indexOf.has(active) ? optionId(active) : undefined);

	/** Under the button, as wide as it at least, kept inside the window. */
	function place() {
		if (!trigger || !popup) return;
		const at = trigger.getBoundingClientRect();
		const width = Math.min(Math.max(at.width, 320), innerWidth - 16);
		const below = innerHeight - at.bottom - 12;
		const above = at.top - 12;
		const down = below >= 280 || below >= above;
		const room = Math.max(160, Math.min(448, down ? below : above));
		popup.style.width = `${width}px`;
		popup.style.maxHeight = `${room}px`;
		popup.style.left = `${Math.max(8, Math.min(at.left, innerWidth - width - 8))}px`;
		popup.style.top = down ? `${at.bottom + 4}px` : '';
		popup.style.bottom = down ? '' : `${innerHeight - at.top + 4}px`;
	}

	// While open, the list follows the button when the page scrolls or the window changes size: it
	// sits in the top layer, so it would otherwise stay where it opened, over another field.
	$effect(() => {
		if (!open) return;
		const follow = (e: Event) => {
			if (!(e.target instanceof Node && popup?.contains(e.target))) place();
		};
		addEventListener('scroll', follow, { capture: true, passive: true });
		addEventListener('resize', follow);
		return () => {
			removeEventListener('scroll', follow, { capture: true });
			removeEventListener('resize', follow);
		};
	});

	/** Scrolls the list alone, never the page behind it, so that an option shows below its
	 * group's sticky heading: in the middle on opening, just into view when moving. */
	function reveal(key: string | null, centre = false) {
		const option = key !== null ? document.getElementById(optionId(key)) : null;
		if (!list || !option) return;
		const heading = option.closest('.group')?.querySelector<HTMLElement>('.heading');
		const covered = heading?.offsetHeight ?? 0;
		const top = option.offsetTop;
		const bottom = top + option.offsetHeight;
		if (centre) {
			list.scrollTop = top - (list.clientHeight - option.offsetHeight) / 2;
		} else if (top - covered < list.scrollTop) {
			list.scrollTop = top - covered;
		} else if (bottom > list.scrollTop + list.clientHeight) {
			list.scrollTop = bottom - list.clientHeight;
		}
	}

	function toggled(e: ToggleEvent) {
		open = e.newState === 'open';
		if (!open) {
			// Emptied on closing, so the list opens whole the next time.
			query = '';
			active = null;
			giveFocusBack(trigger, popup);
			return;
		}
		place();
		const start =
			choosable.find((model) => chosen && keyOf(model) === keyOf(chosen)) ?? choosable[0];
		active = start ? keyOf(start) : null;
		field?.focus({ preventScroll: true });
		reveal(active, true);
	}

	/** Typing: the keys go to the first model the words find. */
	function typed() {
		active = choosable[0] ? keyOf(choosable[0]) : null;
		if (list) list.scrollTop = 0;
	}

	function choose(model: ModelInfo) {
		if (unavailable(model)) return;
		onchoose(model);
		popup?.hidePopover();
	}

	function triggerKeys(e: KeyboardEvent) {
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			popup?.showPopover();
		}
	}

	function fieldKeys(e: KeyboardEvent) {
		const at = choosable.findIndex((model) => keyOf(model) === active);
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			e.preventDefault();
			if (choosable.length === 0) return;
			const to =
				e.key === 'ArrowDown' ? Math.min(at + 1, choosable.length - 1) : Math.max(at - 1, 0);
			active = keyOf(choosable[to]);
			reveal(active);
		} else if (e.key === 'PageDown' || e.key === 'PageUp') {
			e.preventDefault();
			if (choosable.length === 0) return;
			const to = e.key === 'PageDown' ? choosable.length - 1 : 0;
			active = keyOf(choosable[to]);
			reveal(active);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const model = choosable[at];
			if (model) choose(model);
		} else if (e.key === 'Escape' && query) {
			// What was typed goes first; the list, and a dialog around it, stay open.
			e.preventDefault();
			e.stopPropagation();
			query = '';
			typed();
			const start = choosable.find((model) => chosen && keyOf(model) === keyOf(chosen));
			if (start) {
				active = keyOf(start);
				reveal(active, true);
			}
		} else if (e.key === 'Tab') {
			popup?.hidePopover();
		}
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
		popovertarget="{id}-popup"
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
	<div bind:this={popup} id="{id}-popup" class="popup" popover="auto" ontoggle={toggled}>
		<div class="search">
			<Icon name="search" size={16} />
			<input
				bind:this={field}
				bind:value={query}
				type="search"
				placeholder={m.newJob.findModelPlaceholder}
				aria-label={m.newJob.findModel}
				aria-controls="{id}-list"
				aria-autocomplete="list"
				aria-activedescendant={activeId}
				autocomplete="off"
				spellcheck="false"
				oninput={typed}
				onkeydown={fieldKeys}
			/>
		</div>
		<div bind:this={list} id="{id}-list" class="list" role="listbox" aria-labelledby="{id}-label">
			{#each shown as group (group.key)}
				<div class="group" role="group" aria-labelledby="{id}-{group.key}">
					<div class="heading" id="{id}-{group.key}">
						{#if group.tier}<TierMark tier={group.tier} />{/if}{headingOf(group)}
					</div>
					{#each group.models as model (keyOf(model))}
						{@const why = unavailable(model)}
						{@const selected = chosen !== undefined && keyOf(model) === keyOf(chosen)}
						{@const summary = why ?? summaryOf(model)}
						<!-- The keys are the search field's, above; the pointer leaves it the focus. -->
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<div
							id={optionId(keyOf(model))}
							class="option"
							class:active={active === keyOf(model)}
							role="option"
							tabindex="-1"
							aria-selected={selected}
							aria-disabled={why !== null}
							onmousedown={(e) => e.preventDefault()}
							onclick={() => choose(model)}
						>
							<span class="name">
								{nameOf(model)}{#if providerOf(model)}<span class="provider">
										{providerOf(model)}
									</span>{/if}
							</span>
							{#if selected}<span class="check" aria-hidden="true">
									<Icon name="check" size={14} />
								</span>{/if}
							{#if summary}
								<span class="desc">{summary}</span>
							{/if}
						</div>
					{/each}
				</div>
			{/each}
		</div>
		{#if shown.length === 0}
			<p class="none" role="status">{m.newJob.noModelFound}</p>
		{/if}
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

	/* In the top layer, placed by `place`: the search field on top, the list scrolling under it. */
	.popup {
		position: fixed;
		inset: auto;
		margin: 0;
		padding: 0;
		border: 0;
		border-radius: var(--radius-card);
		background: var(--surface-2);
		box-shadow: var(--shadow-pop);
		font-family: var(--font-ui);
		font-size: var(--type-sm);
		color: var(--text-body);
		overflow: hidden;
	}
	.popup:popover-open {
		display: flex;
		flex-direction: column;
	}
	/* Drawn as the language search boxes are: an icon, then the field, a line under them. */
	.search {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex: none;
		padding: var(--space-1) var(--space-3);
		border-bottom: 1px solid var(--line-soft);
		color: var(--text-muted);
	}
	.search input {
		flex: 1;
		min-width: 0;
		min-height: var(--control-md);
		padding: 0;
		border: 0;
		background: none;
		box-shadow: none;
		color: var(--text-bright);
		font: inherit;
		outline: none;
	}
	.search:focus-within {
		color: var(--text-secondary);
	}
	.list {
		flex: 1 1 auto;
		min-height: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: var(--space-1);
	}
	.none {
		margin: 0;
		padding: var(--space-3);
		font-size: var(--type-sm);
		color: var(--text-muted);
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
	.option.active {
		background: var(--wash);
	}
	/* The model the keys are on, as a focused one was drawn. */
	.option.active {
		box-shadow: inset 0 0 0 1.5px var(--text-bright);
	}
	.option[aria-disabled='true'] {
		cursor: not-allowed;
		opacity: 0.5;
	}
	.option[aria-disabled='true']:hover {
		background: none;
	}
	/* A server's model can have a long name (`hf.co/bartowski/…:Q4_K_M`): it wraps anywhere. */
	.name {
		min-width: 0;
		overflow-wrap: anywhere;
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
	/* Shown whole and never resized under the pointer: the catalogue keeps each description short
	   enough for the list to stay readable. */
	.desc {
		grid-column: 1;
		font-size: var(--type-xs);
		line-height: 1.4;
		color: var(--text-muted);
	}
</style>
