<script lang="ts">
	// The fields of an analysis: what to find in each document. Each field is a name, the kind of
	// answer it takes and, in the researcher's words, what belongs in it. Common fields are one
	// click away; keys are given by the core when the analysis is saved.
	import Icon from './Icon.svelte';
	import type { AnalysisSchema } from '#lib/bindings/AnalysisSchema.js';
	import type { Field } from '#lib/bindings/Field.js';
	import type { FieldKind } from '#lib/bindings/FieldKind.js';
	import { messages } from '#lib/i18n/index.js';

	let { schema = $bindable() }: { schema: AnalysisSchema } = $props();
	const m = messages();

	type Kind = FieldKind['type'];
	const kinds: Kind[] = ['text', 'list', 'choice', 'choices'];

	const values = (kind: FieldKind) =>
		kind.type === 'choice' || kind.type === 'choices' ? kind.values : [];

	function kindOf(type: Kind, previous: FieldKind): FieldKind {
		if (type === 'text') return { type };
		if (type === 'list') return { type, max: previous.type === 'list' ? previous.max : null };
		return { type, values: values(previous) };
	}

	function update(i: number, change: Partial<Field>) {
		schema.fields[i] = { ...schema.fields[i], ...change };
	}

	function move(i: number, by: number) {
		const fields = [...schema.fields];
		const [field] = fields.splice(i, 1);
		fields.splice(i + by, 0, field);
		schema.fields = fields;
	}

	type Template = keyof typeof m.fields.templates;
	const templates: Template[] = [
		'summary',
		'people',
		'places',
		'organisations',
		'keywords',
		'tone',
		'custom'
	];

	function add(template: Template) {
		const { label, instructions } = m.fields.templates[template];
		const kind: FieldKind =
			template === 'summary' || template === 'custom'
				? { type: 'text' }
				: template === 'tone'
					? { type: 'choice', values: [...m.fields.toneValues] }
					: { type: 'list', max: null };
		const field: Field = {
			key: '',
			label: template === 'custom' ? '' : label,
			instructions,
			kind,
			reason: template === 'tone'
		};
		schema.fields = [...schema.fields, field];
	}
</script>

<section class="fields" aria-labelledby="fields-title">
	<h2 id="fields-title">{m.fields.title}</h2>
	<p class="muted small">{m.fields.hint}</p>

	<ol>
		{#each schema.fields as field, i (i)}
			<li class="field-row">
				<div class="top">
					<label class="field name">
						<span>{m.fields.name}</span>
						<input
							type="text"
							value={field.label}
							oninput={(e) => update(i, { label: e.currentTarget.value })}
						/>
					</label>
					<label class="field">
						<span>{m.fields.kind}</span>
						<select
							value={field.kind.type}
							onchange={(e) =>
								update(i, { kind: kindOf(e.currentTarget.value as Kind, field.kind) })}
						>
							{#each kinds as kind (kind)}
								<option value={kind}>{m.fields.kinds[kind]}</option>
							{/each}
						</select>
					</label>
					{#if field.kind.type === 'list'}
						{@const kind = field.kind}
						<label class="field max">
							<span>{m.fields.max}</span>
							<input
								type="number"
								min="1"
								max="200"
								value={kind.max ?? ''}
								oninput={(e) => {
									const n = Number.parseInt(e.currentTarget.value, 10);
									update(i, { kind: { ...kind, max: Number.isFinite(n) && n > 0 ? n : null } });
								}}
							/>
						</label>
					{/if}
					<div class="order">
						<button
							class="btn sm icon"
							disabled={i === 0}
							aria-label={m.fields.up}
							title={m.fields.up}
							onclick={() => move(i, -1)}><Icon name="up" /></button
						>
						<button
							class="btn sm icon"
							disabled={i === schema.fields.length - 1}
							aria-label={m.fields.down}
							title={m.fields.down}
							onclick={() => move(i, 1)}><Icon name="down" /></button
						>
						<button
							class="btn sm icon ghost"
							aria-label={m.fields.remove(field.label)}
							title={m.fields.remove(field.label)}
							onclick={() => (schema.fields = schema.fields.filter((_, j) => j !== i))}
							><Icon name="close" /></button
						>
					</div>
				</div>
				{#if field.kind.type === 'choice' || field.kind.type === 'choices'}
					{@const kind = field.kind}
					<label class="field">
						<span>{m.fields.values}</span>
						<input
							type="text"
							placeholder={m.fields.valuesPlaceholder}
							value={kind.values.join(', ')}
							onchange={(e) =>
								update(i, {
									kind: {
										...kind,
										values: e.currentTarget.value
											.split(',')
											.map((v) => v.trim())
											.filter(Boolean)
									}
								})}
						/>
					</label>
				{/if}
				<label class="field">
					<span>{m.fields.instructions}</span>
					<textarea
						rows="2"
						placeholder={m.fields.instructionsPlaceholder}
						value={field.instructions}
						oninput={(e) => update(i, { instructions: e.currentTarget.value })}></textarea>
				</label>
				<label class="inline">
					<input
						type="checkbox"
						checked={field.reason}
						onchange={(e) => update(i, { reason: e.currentTarget.checked })}
					/>
					{m.fields.reason}
				</label>
			</li>
		{/each}
	</ol>

	<div class="add">
		<span class="label">{m.fields.add}</span>
		{#each templates as template (template)}
			<button class="btn sm" onclick={() => add(template)}
				><Icon name="plus" size={14} />{m.fields.templates[template].label}</button
			>
		{/each}
	</div>
</section>

<style>
	.fields {
		display: grid;
		gap: var(--space-2);
		padding-top: var(--space-3);
		border-top: 1px solid var(--rule);
	}
	.small {
		font-size: var(--type-sm);
	}
	ol {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
	}
	.field-row {
		display: grid;
		gap: var(--space-2);
		padding: var(--space-3) 0;
		border-bottom: 1px solid var(--line);
	}
	.top {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		align-items: end;
	}
	.name {
		flex: 1 1 12rem;
	}
	.max input {
		width: 5rem;
	}
	.order {
		display: flex;
		gap: var(--space-1);
		margin-left: auto;
	}
	.btn.icon {
		width: var(--control-sm);
		padding: 0;
	}
	textarea {
		font-size: var(--type-sm);
		min-height: 3.25rem;
	}
	.inline {
		display: inline-flex;
		gap: var(--space-2);
		align-items: center;
		font-size: var(--type-sm);
	}
	.add {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		align-items: center;
		padding-top: var(--space-2);
	}
</style>
