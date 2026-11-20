// The guide for researchers new to these tools (docs/guide/, published on GitHub Pages), opened
// in the browser from the first-launch welcome, the Glossary and Settings › About.

import { messages } from './i18n/index.js';

/** The guide's English home page; each language's page is named, relative to it, in its catalog. */
export const GUIDE_URL = 'https://fmadore.github.io/Kalamu/guide/';

/** The guide in the interface language, or in English for a language it is not written in. */
export function guideUrl(): string {
	return new URL(messages().shell.guidePage, GUIDE_URL).href;
}
