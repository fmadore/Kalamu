## Checking a download

`SHA256SUMS` lists the SHA-256 fingerprint of every file of this release. To check a file you
downloaded, put `SHA256SUMS` beside it and run, in that folder:

- **Windows** (PowerShell): `Get-FileHash .\Kalamu_<version>_x64-setup.exe`, and compare the
  hash it prints with the file's line in `SHA256SUMS` (letter case aside).
- **macOS** (Terminal): `shasum -a 256 -c SHA256SUMS --ignore-missing`, which prints `OK` for
  each file present.
- **Linux**: `sha256sum -c SHA256SUMS --ignore-missing`.

Each file also carries a build attestation: a statement signed through GitHub that it was built
by Kalamu's release workflow from this release's tagged commit. With the
[GitHub CLI](https://cli.github.com/):

```
gh attestation verify Kalamu_<version>_x64-setup.exe --repo fmadore/Kalamu
```

It prints `Verification succeeded!` and the workflow and commit the file came from. The Windows
installers are not code-signed; the macOS disk image is signed with Kalamu's Developer ID and
notarised by Apple.
