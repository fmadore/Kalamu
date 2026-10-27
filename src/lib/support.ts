// Voluntary support. Everything the app does stays free; this is one link, at the end of
// Settings, that unlocks nothing.

import { openUrl } from './tauri.js';

/** The creator's Buy Me a Coffee page, also in the README and `.github/FUNDING.yml`. */
export const SUPPORT_URL = 'https://buymeacoffee.com/fmadore';

/** Open the support page in the default browser (a new tab in the browser preview). */
export function openSupportPage(): Promise<void> {
	return openUrl(SUPPORT_URL);
}
