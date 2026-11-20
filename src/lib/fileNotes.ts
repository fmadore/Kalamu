// A file's notes as New job and a job's files word them: its input checks, and how long a
// recording lasts.

import type { InputWarning } from './bindings/InputWarning.js';
import type { Messages } from './i18n/index.js';

/** A warning in words; `nameOf` gives the file name of a source a warning points at (the
 * photograph a retake is the same page as). */
export function warningText(
	w: InputWarning,
	m: Messages,
	nameOf: (sourceId: string) => string = () => ''
): string {
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
		case 'tool_missing':
			return m.warning.tool_missing;
		case 'no_sound_track':
			return m.warning.no_sound_track;
		case 'long_recording':
			return m.warning.long_recording(duration(w.minutes * 60_000, m), w.pieces);
		case 'quiet_recording':
			return m.warning.quiet_recording(w.mean_db);
		case 'same_page':
			return m.warning.same_page(nameOf(w.other), nameOf(w.source_id));
		case 'blurred':
			return m.warning.blurred(nameOf(w.source_id));
	}
}

/** "1 h 05 min", "12 min", "45 s". */
export function duration(ms: number, m: Messages): string {
	const total = Math.round(ms / 1000);
	const h = Math.floor(total / 3600);
	const min = Math.floor((total % 3600) / 60);
	if (h > 0) return m.newJob.hoursMinutes(h, String(min).padStart(2, '0'));
	if (min > 0) return m.newJob.minutes(min);
	return m.newJob.seconds(total);
}
