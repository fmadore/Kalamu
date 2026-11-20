// The sample data's photographs from a reading room (#216): with `?photos`, Add folder picks a
// folder where IMG_0142 and IMG_0143 are the same page taken twice (the second sharper) and
// IMG_0150 is blurred, and `check_photos` answers as the engine would.

import type { InputWarning } from '../bindings/InputWarning.js';
import type { PhotoChecks } from '../bindings/PhotoChecks.js';
import type { SourceFile } from '../bindings/SourceFile.js';
import SAMPLE_SCAN from './sample-scan.jpg?inline';

export const READING_ROOM = 'C:\\Archives\\Salle de lecture';
export const READING_ROOM_PHOTOS = [
	'IMG_0142.jpg',
	'IMG_0143.jpg',
	'IMG_0144.jpg',
	'IMG_0150.jpg'
].map((name) => `${READING_ROOM}\\${name}`);

const scan = SAMPLE_SCAN.replace(/^data:[^,]*,/, '');

/** What the engine would find among these photographs. */
function findings(sources: SourceFile[]): PhotoChecks {
	const named = (name: string) => sources.find((s) => s.file_name === name);
	const warnings: InputWarning[] = [];
	const [first, second, blurred] = [
		named('IMG_0142.jpg'),
		named('IMG_0143.jpg'),
		named('IMG_0150.jpg')
	];
	if (first && second) {
		warnings.push({
			source_id: second.id,
			kind: 'same_page',
			other: first.id,
			keep: second.id,
			clearly_sharper: true
		});
	}
	if (blurred) warnings.push({ source_id: blurred.id, kind: 'blurred' });
	const shown = [first, second, blurred].filter((s): s is SourceFile => !!s);
	return {
		warnings,
		views: shown.map((s) => ({ source_id: s.id, page: scan, detail: scan }))
	};
}

export function photoCommand(cmd: string, a: Record<string, unknown>): unknown {
	switch (cmd) {
		case 'check_photos':
			// A moment, as the engine takes to read the photographs.
			return new Promise((resolve) =>
				setTimeout(() => resolve(findings(a.sources as SourceFile[])), 300)
			);
		case 'stop_photo_checks':
			return null;
		default:
			return undefined;
	}
}
