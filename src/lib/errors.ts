// Turning what the core reports into a sentence, as in Live Translation: the core sends an id
// and technical detail, the catalog owns the sentence. An unknown id falls back to the detail,
// so a newer core is never silent in an older interface.

import type { AppError } from './bindings/AppError.js';
import type { Messages } from './i18n/index.js';

export function isAppError(value: unknown): value is AppError {
	return (
		typeof value === 'object' &&
		value !== null &&
		typeof (value as AppError).id === 'string' &&
		((value as AppError).detail === undefined || typeof (value as AppError).detail === 'string')
	);
}

export function describeError(value: unknown, m: Messages): string {
	if (isAppError(value)) {
		const known = (m.error as Record<string, string | undefined>)[value.id];
		const sentence = known
			? value.detail
				? `${known} (${value.detail})`
				: known
			: (value.detail ?? value.id);
		return typeof value.step === 'number' ? m.common.atStep(value.step, sentence) : sentence;
	}
	if (value instanceof Error) return value.message;
	return String(value);
}

/** The step of a workflow an error names, numbered from 1; none for any other error. */
export function errorStep(value: unknown): number | null {
	return isAppError(value) && typeof value.step === 'number' ? value.step : null;
}
