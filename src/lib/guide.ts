// The guide for researchers new to these tools (docs/guide/, published on GitHub Pages), opened
// in the browser from the first-launch welcome, the Glossary and Settings › About.

import { messages } from './i18n/index.js';

/** The guide's English home page; each language's page is named, relative to it, in its catalog. */
export const GUIDE_URL = 'https://kalamu.frederickmadore.com/guide/';

/** The guide in the interface language, or in English for a language it is not written in;
 *  with `page`, that page of it (`other-servers`). */
export function guideUrl(page?: string): string {
	const home = messages().shell.guidePage;
	return new URL(page ? `${home}${page}.html` : home, GUIDE_URL).href;
}
