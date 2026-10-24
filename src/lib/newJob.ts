// The New job form's logic, kept out of the components so it can be tested on its own.

import type { Catalog } from './bindings/Catalog.js';
import type { ExportFormat } from './bindings/ExportFormat.js';
import type { InputWarning } from './bindings/InputWarning.js';
import type { ModelInfo } from './bindings/ModelInfo.js';
import type { SetupRequest } from './bindings/SetupRequest.js';
import type { Task } from './bindings/Task.js';
import type { Messages } from './i18n/index.js';

/** One model set-up as edited in the form. */
export interface Draft {
	provider: string;
	model: string;
	presetId: string | null;
	label: string;
	diarize: boolean;
	timestamps: boolean;
}

export const FORMATS: Record<Task, ExportFormat[]> = {
	text_extraction: ['txt', 'markdown', 'docx', 'json', 'csv'],
	audio_transcription: ['txt', 'markdown', 'docx', 'srt', 'vtt', 'json', 'csv']
};

export function modelsFor(catalog: Catalog, provider: string, task: Task): ModelInfo[] {
	return catalog.models.filter((m) => m.provider === provider && m.tasks.includes(task));
}

export function modelInfo(catalog: Catalog, d: Draft): ModelInfo | undefined {
	return catalog.models.find((m) => m.provider === d.provider && m.id === d.model);
}

/** Dedicated OCR and speech models take no prompt; a missing flag means they do. */
export function acceptsPrompt(model: ModelInfo | undefined): boolean {
	return model?.accepts_prompt !== false;
}

/**
 * A set-up for a usable model: the recommended one first, then any model with a key, avoiding
 * models already in `used` so a second set-up compares against something different.
 */
export function newDraft(
	catalog: Catalog,
	task: Task,
	presetId: string | null,
	hasKey: (provider: string) => boolean,
	used: string[] = []
): Draft {
	const forTask = catalog.models.filter((m) => m.tasks.includes(task));
	const usable = forTask.filter((m) => hasKey(m.provider));
	const fresh = usable.filter((m) => !used.includes(`${m.provider}/${m.id}`));
	const pick =
		fresh.find((m) => m.recommended_for.includes(task)) ??
		fresh[0] ??
		usable.find((m) => m.recommended_for.includes(task)) ??
		usable[0] ??
		forTask[0];
	return {
		provider: pick?.provider ?? '',
		model: pick?.id ?? '',
		presetId,
		label: '',
		diarize: true,
		timestamps: true
	};
}

export function toRequest(catalog: Catalog, task: Task, d: Draft): SetupRequest {
	return {
		provider: d.provider,
		model: d.model,
		preset_id: acceptsPrompt(modelInfo(catalog, d)) ? d.presetId : null,
		label: d.label.trim() || null,
		options: {
			temperature: null,
			diarize: task === 'audio_transcription' && d.diarize,
			timestamps: task === 'audio_transcription' && d.timestamps,
			speakers: null
		}
	};
}

export function warningText(w: InputWarning, m: Messages): string {
	const pages = (list: number[]) =>
		list.length > 6 ? `${list.slice(0, 6).join(', ')}…` : list.join(', ');
	switch (w.kind) {
		case 'double_page_spread':
			return m.warning.double_page_spread(pages(w.pages));
		case 'low_resolution':
			return m.warning.low_resolution(w.dpi, pages(w.pages));
		case 'has_text_layer':
			return m.warning.has_text_layer(w.pages_with_text);
		case 'unsupported_image':
			return m.warning.unsupported_image(w.detail);
		case 'large_file':
			return m.warning.large_file;
	}
}

/** A job name from what was added: the folder's name, or the first file's. */
export function suggestName(added: string[], expanded: string[]): string {
	const last = (p: string) => p.split(/[\\/]/).filter(Boolean).pop() ?? '';
	if (added.length === 1 && expanded.length > 1) return last(added[0]);
	return expanded[0] ? last(expanded[0]) : '';
}
