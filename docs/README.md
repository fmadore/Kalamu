# Documentation

## What the app does

| Document | What it covers |
| --- | --- |
| [Home page](index.html) | The public home page at [kalamu.frederickmadore.com](https://kalamu.frederickmadore.com/): one self-contained HTML file with its fonts and pictures in `assets/home/`, its wording taken from the [store listing](store-listing.md) and the README, its screenshots from `store/screenshots/`; change those first, then the page. The French version is [fr/index.html](fr/index.html), with the same styles and script and its text from the French store listing and `fr.ts` |
| [Guide](guide/index.md) | How to use Kalamu, chapter by chapter; published at [kalamu.frederickmadore.com/guide](https://kalamu.frederickmadore.com/guide/) in the home page's look (`_layouts/guide.html`, `_includes/`, `assets/site.css`); in French in [guide/fr](guide/fr/index.md), published at [kalamu.frederickmadore.com/guide/fr](https://kalamu.frederickmadore.com/guide/fr/), the same layout listing each language's chapters and quoting the interface's own French labels |
| Search engines | The site is served at its own domain (`CNAME`). `_includes/seo.html` gives the guide and the privacy policy their address, the other language and link previews; the two home pages write the same tags in their own markup, with the app described for search engines (schema.org). [sitemap.xml](sitemap.xml) lists the home pages, the guide and the privacy policy, never the developers' documents; [robots.txt](robots.txt) points to it. A new public page goes into the sitemap. |
| [Specification](specification.md) | What the app must do, and why: tasks, review, comparison, resuming, privacy |
| [Output formats](formats.md) | Every export format and `provenance.json` |
| [Privacy policy](privacy.md) | What the app sends, to whom, and what it keeps; published for the Store listing in the same look (`_layouts/page.html`) |

## Plans and decisions

| Document | What it covers |
| --- | --- |
| [Roadmap](roadmap.md) | What is built, what is next, and the way to the Store release |
| [Decisions](decisions/) | Dated records of what was decided and why |

## Building

| Document | What it covers |
| --- | --- |
| [Development](development.md) | Building, checks, live runs, the browser preview with sample data |
| [AGENTS.md](../AGENTS.md) | Architecture and rules for coding agents |
| [Design](../DESIGN.md) | The Graphite and highlighter design system |
| [FFmpeg](ffmpeg.md) | The bundled LGPL build and how its licence is met |
| [Icon options](design/icon-options.html) | Earlier icon concepts, kept for the record |

## Releasing

| Document | What it covers |
| --- | --- |
| [Microsoft Store](microsoft-store.md) | Status, the policies that apply, what is missing, the acceptance checklist and the Partner Center steps |
| [MSIX packaging](packaging-msix.md) | The Store package: what is committed, the icons, building and installing it locally, what CI does |
| [Development: releases](development.md#releases) | Version files, release notes and the tag that builds the installers and packages |
| [Release notes](release-1.10.1.md) | Latest release notes ([1.10.1](release-1.10.1.md), [1.10.0](release-1.10.0.md), [1.9.0](release-1.9.0.md), [1.8.0](release-1.8.0.md), [1.7.0](release-1.7.0.md), [1.6.0](release-1.6.0.md), [1.5.0](release-1.5.0.md), [1.4.1](release-1.4.1.md), [1.4.0](release-1.4.0.md), [1.3.0](release-1.3.0.md), [1.2.0](release-1.2.0.md), [1.1.2](release-1.1.2.md), [1.1.1](release-1.1.1.md), [1.1.0](release-1.1.0.md), [1.0.0](release-1.0.0.md), [0.4.0](release-0.4.0.md), [0.3.0](release-0.3.0.md)) |
| [Mac App Store](mac-app-store.md) | The Apple setup, GitHub secrets, App Store Connect answers and the steps for each version |
| [A day on a rented Mac](mac-test-day.md) | The checklist for a day on real Apple hardware, which Claude can run there: the disk image, what was written since 1.6.0, the keyboard and VoiceOver, a crash, and the TestFlight build |
| [A test pass on Windows](windows-test-pass.md) | The checklist for the packaged app before a version is tagged, which Claude can run on the PC: what is left to try by hand and the live runs on the researcher's accounts |
| [Store listing](store-listing.md) | Paste-ready listing text, certification and review notes and screenshot captions, for both stores |
| [Store licence terms](store-license-terms.md) | The terms shown on the Store listing, replacing Microsoft's standard ones |
| [Security policy](../SECURITY.md) | Reporting vulnerabilities, credentials, installer signing |
| [Repository statistics](repository-statistics.md) | Downloads per version and visits to the repository, recorded daily on the `stats` branch |
