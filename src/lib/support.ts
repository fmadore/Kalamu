// Voluntary support. Everything the app does stays free; this is one link, at the foot of the
// sidebar and at the end of Settings, that unlocks nothing.

/** The creator's Buy Me a Coffee page, also in the README and `.github/FUNDING.yml`. */
export const SUPPORT_URL = 'https://buymeacoffee.com/fmadore';

/**
 * Reporting a problem (Microsoft Store policy 11.16): the GitHub issue form for AI output, opened
 * in the browser with the app's version filled in. Kalamu sends nothing itself.
 */
export function reportUrl(version: string | undefined): string {
	const url = new URL('https://github.com/fmadore/Kalamu/issues/new');
	url.searchParams.set('template', 'output.yml');
	if (version) url.searchParams.set('version', version);
	return url.toString();
}
