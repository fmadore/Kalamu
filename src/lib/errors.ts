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
		const sentence = (m.error as Record<string, string | undefined>)[value.id];
		if (sentence) return value.detail ? `${sentence} (${value.detail})` : sentence;
		return value.detail ?? value.id;
	}
	if (value instanceof Error) return value.message;
	return String(value);
}
