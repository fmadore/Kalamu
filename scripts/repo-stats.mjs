#!/usr/bin/env node
// Records the repository's release downloads and visits, once a day, in the folder given as the
// only argument: the `stats` branch, checked out by .github/workflows/stats.yml. GitHub shows
// downloads only as a running total and keeps visits for 14 days, so the history exists only if
// something writes it down every day. Each file in `data/` is a CSV a spreadsheet or a dashboard
// can read; `README.md` beside it is a summary GitHub renders.
//
//   data/releases.csv  date, tag, published, prerelease, downloads: every release's total that day
//   data/assets.csv    date, tag, asset, downloads: a file's total, written when it changes
//   data/views.csv     date, views, unique_visitors: GitHub's daily figures (UTC)
//
// GITHUB_TOKEN (the workflow's own) reads releases; a public repository's need none. Visits need a token with the Administration
// (read) permission on the repository, in STATS_TOKEN; without one, downloads are still
// recorded. Running twice on the same day replaces that day's rows.

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const outDir = process.argv[2];
const repo = process.env.GITHUB_REPOSITORY;
const token = process.env.GITHUB_TOKEN;
const trafficToken = process.env.STATS_TOKEN;

if (!outDir || !repo) {
	console.error(
		'Usage: GITHUB_REPOSITORY=owner/name [GITHUB_TOKEN=…] [STATS_TOKEN=…] node scripts/repo-stats.mjs <folder>'
	);
	process.exit(2);
}

const today = new Date().toISOString().slice(0, 10);
const dataDir = join(outDir, 'data');
const repoName = repo.split('/')[1];

// --- GitHub API ---------------------------------------------------------------------------

/** @param {string} path @param {string | undefined} auth */
async function api(path, auth) {
	const url = path.startsWith('https://') ? path : `https://api.github.com/repos/${repo}${path}`;
	const response = await fetch(url, {
		headers: {
			Accept: 'application/vnd.github+json',
			...(auth ? { Authorization: `Bearer ${auth}` } : {}),
			'X-GitHub-Api-Version': '2022-11-28',
			'User-Agent': `${repoName}-stats`
		}
	});
	if (!response.ok) {
		throw new Error(`GET ${url}: ${response.status} ${await response.text()}`);
	}
	return { body: await response.json(), link: response.headers.get('link') ?? '' };
}

/** Every page of a list endpoint. @param {string} path @param {string | undefined} auth */
async function apiAll(path, auth) {
	const items = [];
	/** @type {string | undefined} */
	let next = `${path}${path.includes('?') ? '&' : '?'}per_page=100`;
	while (next) {
		const { body, link } = await api(next, auth);
		items.push(...body);
		next = /<([^>]+)>;\s*rel="next"/.exec(link)?.[1];
	}
	return items;
}

// --- CSV ----------------------------------------------------------------------------------

/** @param {string} text @returns {string[][]} */
function parseCsv(text) {
	const rows = [];
	let row = [];
	let field = '';
	let quoted = false;
	for (let i = 0; i < text.length; i++) {
		const c = text[i];
		if (quoted) {
			if (c === '"' && text[i + 1] === '"') {
				field += '"';
				i++;
			} else if (c === '"') {
				quoted = false;
			} else {
				field += c;
			}
		} else if (c === '"') {
			quoted = true;
		} else if (c === ',') {
			row.push(field);
			field = '';
		} else if (c === '\n') {
			row.push(field);
			rows.push(row);
			row = [];
			field = '';
		} else if (c !== '\r') {
			field += c;
		}
	}
	if (field || row.length) {
		row.push(field);
		rows.push(row);
	}
	return rows;
}

/** @param {unknown} value */
function csvField(value) {
	const s = String(value ?? '');
	return /[",\n\r]/.test(s) ? `"${s.replaceAll('"', '""')}"` : s;
}

/** @typedef {Record<string, string>} Row */

/** @param {string} name @returns {Promise<Row[]>} */
async function readTable(name) {
	let text;
	try {
		text = await readFile(join(dataDir, name), 'utf8');
	} catch {
		return [];
	}
	const [header, ...rows] = parseCsv(text);
	if (!header) return [];
	return rows.map((cells) => Object.fromEntries(header.map((h, i) => [h, cells[i] ?? ''])));
}

/** @param {string} name @param {string[]} columns @param {Row[]} rows */
async function writeTable(name, columns, rows) {
	const lines = [
		columns.join(','),
		...rows.map((r) => columns.map((c) => csvField(r[c])).join(','))
	];
	await writeFile(join(dataDir, name), `${lines.join('\n')}\n`);
}

/** Rows sorted by date, then by the other columns given. @param {Row[]} rows @param {string[]} keys */
function sortRows(rows, keys) {
	return rows.sort((a, b) => {
		for (const k of ['date', ...keys]) {
			const order = (a[k] ?? '').localeCompare(b[k] ?? '', 'en', { numeric: true });
			if (order) return order;
		}
		return 0;
	});
}

// --- Recording ----------------------------------------------------------------------------

await mkdir(dataDir, { recursive: true });

/** @typedef {{ name: string, download_count: number }} Asset */
/** @typedef {{ tag_name: string, draft: boolean, prerelease: boolean, published_at: string | null, created_at: string, assets: Asset[] }} Release */
/** @type {Release[]} */
const releases = (await apiAll('/releases', token)).filter((r) => !r.draft);

{
	const rows = (await readTable('releases.csv')).filter((r) => r.date !== today);
	for (const r of releases) {
		rows.push({
			date: today,
			tag: r.tag_name,
			published: (r.published_at ?? r.created_at).slice(0, 10),
			prerelease: String(r.prerelease),
			downloads: String(r.assets.reduce((sum, a) => sum + a.download_count, 0))
		});
	}
	await writeTable(
		'releases.csv',
		['date', 'tag', 'published', 'prerelease', 'downloads'],
		sortRows(rows, ['tag'])
	);
}

{
	// Only changes are written: an asset's total on any day is its latest row up to that day.
	const rows = (await readTable('assets.csv')).filter((r) => r.date !== today);
	const last = new Map();
	for (const r of sortRows([...rows], [])) last.set(`${r.tag}\n${r.asset}`, r.downloads);
	for (const r of releases) {
		for (const a of r.assets) {
			const downloads = String(a.download_count);
			if (last.get(`${r.tag_name}\n${a.name}`) !== downloads) {
				rows.push({ date: today, tag: r.tag_name, asset: a.name, downloads });
			}
		}
	}
	await writeTable(
		'assets.csv',
		['date', 'tag', 'asset', 'downloads'],
		sortRows(rows, ['tag', 'asset'])
	);
}

if (trafficToken) {
	// Each day appears in 14 answers in a row; the latest one wins, so today's partial count is
	// corrected by tomorrow's run.
	const { body } = await api('/traffic/views', trafficToken);
	const byDate = new Map((await readTable('views.csv')).map((r) => [r.date, r]));
	for (const day of body.views) {
		const date = day.timestamp.slice(0, 10);
		byDate.set(date, { date, views: String(day.count), unique_visitors: String(day.uniques) });
	}
	await writeTable(
		'views.csv',
		['date', 'views', 'unique_visitors'],
		sortRows([...byDate.values()], [])
	);
} else {
	console.log('::warning::STATS_TOKEN is not set: visits to the repository were not recorded.');
}

// --- Summary (README.md on the stats branch) ----------------------------------------------

/** @param {string} date @param {number} days */
function addDays(date, days) {
	const d = new Date(`${date}T00:00:00Z`);
	d.setUTCDate(d.getUTCDate() + days);
	return d.toISOString().slice(0, 10);
}

/** @param {string} s */
const cell = (s) => String(s).replaceAll('|', '\\|');
const fmt = (/** @type {number} */ n) => n.toLocaleString('en-US');

/** A Mermaid chart of daily values, or a sentence when there are too few days. */
function chart(
	/** @type {string} */ title,
	/** @type {string[]} */ dates,
	/** @type {{ kind: 'bar' | 'line', values: number[] }[]} */ series
) {
	if (dates.length < 2) return '_Not enough days recorded yet for a chart._\n';
	const labels = dates.map((d) => `"${d.slice(5)}"`).join(', ');
	const lines = series.map((s) => `    ${s.kind} [${s.values.join(', ')}]`).join('\n');
	return `\`\`\`mermaid\nxychart-beta\n    title "${title}"\n    x-axis [${labels}]\n${lines}\n\`\`\`\n`;
}

const releaseRows = await readTable('releases.csv');
const releaseDates = [...new Set(releaseRows.map((r) => r.date))].sort();
/** @type {Map<string, Map<string, number>>} date → tag → downloads */
const totals = new Map();
for (const r of releaseRows) {
	if (!totals.has(r.date)) totals.set(r.date, new Map());
	totals.get(r.date)?.set(r.tag, Number(r.downloads));
}

/** A release's downloads in the `days` days up to today, or null if the history is too short. */
function downloadsSince(
	/** @type {string} */ tag,
	/** @type {string} */ published,
	/** @type {number} */ days
) {
	const now = totals.get(today)?.get(tag) ?? 0;
	const cutoff = addDays(today, -days);
	if (published > cutoff) return now;
	const before = releaseDates.filter((d) => d <= cutoff).at(-1);
	if (!before) return null;
	return now - (totals.get(before)?.get(tag) ?? 0);
}

const isVersion = (/** @type {string} */ tag) => /^v?\d/.test(tag);
const current = releaseRows
	.filter((r) => r.date === today)
	.sort(
		(a, b) =>
			b.published.localeCompare(a.published) || b.tag.localeCompare(a.tag, 'en', { numeric: true })
	);

function releaseTable(/** @type {Row[]} */ rows) {
	const lines = [
		'| Release | Published | Downloads | Last 7 days | Last 30 days |',
		'| --- | --- | ---: | ---: | ---: |'
	];
	for (const r of rows) {
		const week = downloadsSince(r.tag, r.published, 7);
		const month = downloadsSince(r.tag, r.published, 30);
		const name = `[${cell(r.tag)}](https://github.com/${repo}/releases/tag/${encodeURIComponent(r.tag)})`;
		lines.push(
			`| ${name}${r.prerelease === 'true' ? ' (pre-release)' : ''} | ${r.published} | ${fmt(Number(r.downloads))} | ${week === null ? '…' : fmt(week)} | ${month === null ? '…' : fmt(month)} |`
		);
	}
	return `${lines.join('\n')}\n`;
}

// Daily downloads: each recorded day minus the one before, release by release, so a release
// that appears counts from zero and one that disappears takes nothing away.
const recent = releaseDates.filter((d) => d > addDays(today, -60));
const dailyDates = [];
const dailyDownloads = [];
for (const date of recent) {
	const previous = releaseDates[releaseDates.indexOf(date) - 1];
	if (!previous) continue;
	const before = totals.get(previous) ?? new Map();
	let sum = 0;
	for (const [tag, n] of totals.get(date) ?? []) sum += Math.max(0, n - (before.get(tag) ?? 0));
	dailyDates.push(date);
	dailyDownloads.push(sum);
}

const views = (await readTable('views.csv')).filter((r) => r.date > addDays(today, -60));
const sumLast14 = (/** @type {Row[]} */ rows, /** @type {string} */ column) =>
	rows.filter((r) => r.date > addDays(today, -14)).reduce((s, r) => s + Number(r[column]), 0);
const versions = current.filter((r) => isVersion(r.tag));
const others = current.filter((r) => !isVersion(r.tag));
const latest = versions.find((r) => r.prerelease !== 'true') ?? versions[0];
const totalDownloads = current.reduce((s, r) => s + Number(r.downloads), 0);
const versionDownloads = versions.reduce((s, r) => s + Number(r.downloads), 0);

let md = `# ${repoName}: statistics\n\n`;
md += `Updated ${today} by the daily *Repository statistics* workflow (\`.github/workflows/stats.yml\` on the default branch). `;
md += `The figures are kept in [\`data/\`](data/) as CSV files, one row per day, for a spreadsheet or a dashboard.\n\n`;
md += '| Downloads, versions | Downloads, all releases | Views, last 14 days |\n';
md += '| ---: | ---: | ---: |\n';
md += `| ${fmt(versionDownloads)} | ${fmt(totalDownloads)} | ${trafficToken ? fmt(sumLast14(views, 'views')) : '…'} |\n\n`;

md += '## Downloads per version\n\n';
md +=
	'Every file of a release counts once each time it is downloaded from GitHub. "…" means the history does not go back that far yet.\n\n';
md += versions.length ? releaseTable(versions) : '_No releases yet._\n';
if (others.length) {
	md += `\n<details><summary>Other releases (${others.length}): tools built for the app, not versions of it</summary>\n\n`;
	md += releaseTable(others);
	md += '\n</details>\n';
}

md += '\n## Downloads per day, last 60 days\n\n';
md += chart('Downloads per day (all releases)', dailyDates, [
	{ kind: 'bar', values: dailyDownloads }
]);

if (latest) {
	const assets = (await readTable('assets.csv')).filter((r) => r.tag === latest.tag);
	const byAsset = new Map();
	for (const r of sortRows(assets, [])) byAsset.set(r.asset, Number(r.downloads));
	md += `\n## Files of ${cell(latest.tag)}\n\n| File | Downloads |\n| --- | ---: |\n`;
	for (const [asset, n] of [...byAsset].sort((a, b) => b[1] - a[1]))
		md += `| ${cell(asset)} | ${fmt(n)} |\n`;
}

md += '\n## Visits to the repository, last 60 days\n\n';
if (!trafficToken) {
	md += '_Not recorded: the workflow needs a `STATS_TOKEN` secret to read traffic._\n';
} else {
	md +=
		"Bars are views, the line unique visitors (GitHub's daily figures, UTC; today is still counting).\n\n";
	md += chart(
		'Views and unique visitors',
		views.map((r) => r.date),
		[
			{ kind: 'bar', values: views.map((r) => Number(r.views)) },
			{ kind: 'line', values: views.map((r) => Number(r.unique_visitors)) }
		]
	);
}

await writeFile(join(outDir, 'README.md'), md);
console.log(`Recorded ${repo} downloads and visits for ${today} in ${outDir}.`);
