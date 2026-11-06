// The bytes of an asset of one of this repository's GitHub releases: the bundled tools' builds
// (FFmpeg, llama.cpp), which scripts/fetch-ffmpeg.mjs and scripts/fetch-llama.mjs check against
// their pinned SHA-256 before installing.

/**
 * The release asset's bytes. With GH_TOKEN or GITHUB_TOKEN set (CI, or a private repository) it
 * goes through the API, which accepts the token; otherwise through the public download link.
 */
export async function releaseAsset(tag, asset) {
	const token = process.env.GH_TOKEN ?? process.env.GITHUB_TOKEN;
	if (!token) {
		const url = `https://github.com/fmadore/Kalamu/releases/download/${tag}/${asset}`;
		const response = await fetch(url);
		if (!response.ok) {
			throw new Error(`${url}: ${response.status} (a private repository needs GH_TOKEN)`);
		}
		return Buffer.from(await response.arrayBuffer());
	}
	const api = 'https://api.github.com/repos/fmadore/Kalamu';
	const auth = { Authorization: `Bearer ${token}`, 'X-GitHub-Api-Version': '2022-11-28' };
	const release = await fetch(`${api}/releases/tags/${tag}`, { headers: auth });
	if (!release.ok) throw new Error(`release ${tag}: ${release.status}`);
	const found = (await release.json()).assets.find((a) => a.name === asset);
	if (!found) throw new Error(`${asset} is not in release ${tag}`);
	const response = await fetch(found.url, {
		headers: { ...auth, Accept: 'application/octet-stream' }
	});
	if (!response.ok) throw new Error(`${asset}: ${response.status}`);
	return Buffer.from(await response.arrayBuffer());
}
