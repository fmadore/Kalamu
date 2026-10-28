# Privacy Policy — Kalamu

**Effective date:** 28 October 2026 (documents the version in development; to be dated again
for the first Store release)

## What this app is

Kalamu is a Windows desktop application for researchers. It extracts text from scanned or
handwritten documents, transcribes recordings and videos, and analyses texts, in one of two
ways the user chooses for each job:

- through a cloud AI provider the user selects and configures with their own API key: Google
  Gemini, OpenAI, Anthropic Claude or Mistral; or
- offline, with Whisper running on the user's computer, for transcription.

The developer operates no server for the app and receives nothing from it.

## Files the user chooses

Kalamu reads only the files and folders the user adds to a job, and the YouTube links the
user pastes.

**With a cloud provider**, each page, recording segment or text is sent directly from the
user's computer to that provider, with the instructions for the task (the prompt and, for an
analysis, the fields to answer), and processed under the user's account and the provider's
terms. Before sending, the app may convert a piece into a form the provider accepts, for
example cutting a PDF into pages or a recording into segments; the original files are never
changed. With a YouTube link, Gemini fetches the public video from YouTube itself; Kalamu
does not download the video.

**In the cheaper mode** (economy), pieces are sent to the provider's batch service instead of
one by one. The provider keeps the batch, its input and its answers for a time set by its own
rules (from about a day to several weeks), in the user's account, and Kalamu collects the
answers when it next opens.

**Offline**, with Whisper, recordings are converted and transcribed on the computer and never
uploaded.

Files and results may contain personal information about the people named or heard in them.
Choose a provider whose terms suit the material, or work offline.

## What the app keeps on the computer

In the app's local data folder (`%LOCALAPPDATA%\io.github.fmadore.kalamu` for the
installer, the package's own folder for the Store app). The test releases up to 0.2, named
Scriptorium, kept it in `%LOCALAPPDATA%\io.github.fmadore.scriptorium`; the first start of
Kalamu moves that folder, and the keys stored under that name in Credential Manager, to the
new names, on the same computer only:

- **The job database** (`jobs.sqlite`): each job's name, the paths and fingerprints of its
  files, its settings and progress, the text and answers returned for each piece, the user's
  corrections and review marks, and the names given to speakers. Results are kept by a
  fingerprint of the file and settings, so the same work is never paid for twice; deleting a
  job removes the job, its progress and corrections, while the stored results stay until the
  app is uninstalled.
- **Settings** (`settings.json`): preferences such as formats, destination, theme, favourite
  languages and the last folder chosen.
- **Prompts and pipelines** the user saves, and a copy of the model catalogue.
- **Whisper models**, downloaded only when the user asks (`whisper-models`).
- **Logs**, for troubleshooting: what the app did and the errors it met, with job identifiers.
  They contain no API keys and no document text.

Results are also written as files where the user chooses: next to the original files or in a
chosen folder, each with a `provenance.json` that records the model, prompt and settings used.

Uninstalling the Store app removes its local data folder. Exported files stay where they were
written.

## API keys

No key is needed for offline transcription. Provider keys are stored in Windows Credential
Manager and read only by the app's native core. A key is sent only to the provider that issued
it, to authenticate requests. It is never written to the job database, the logs or
`provenance.json`, and the developer does not receive, copy or store it.

## Other connections

Besides the provider the user chooses, the app connects to:

- **GitHub** (`raw.githubusercontent.com`), at startup, to check for a newer catalogue of
  models and prices. It downloads a public file and sends nothing about the user.
- **Hugging Face** (`huggingface.co`), only when the user downloads a Whisper model. The
  download is checked against its size and SHA-256 before use.
- **YouTube**, when a link is added: the app asks YouTube's public oEmbed service for the
  video's title. When the user plays a video in the review, YouTube's privacy-enhanced player
  (`youtube-nocookie.com`) loads inside the app; until then, opening a review contacts no one.

These services receive ordinary request information, such as the computer's IP address. When a
proxy is set in Windows, connections go through it.

## Running in the background

Kalamu does not start with Windows. When the user closes the window during a job, the app
asks whether to keep the job running; if so, it stays in the notification area until the job
ends or the user quits. Jobs waiting for a provider's batch resume when the app is next opened.

## Support link

Kalamu shows an optional *Buy me a coffee* link to the developer's page on Buy Me a Coffee
(`https://buymeacoffee.com/fmadore`). The app passes that fixed address to the default browser
and nothing else: it sends no information about the user or the device and contacts no payment
service itself. Anything entered on that page is between the user and Buy Me a Coffee, under the
[Buy Me a Coffee privacy policy](https://buymeacoffee.com/privacy-policy). Supporting is optional
and unlocks nothing.

## What the developer collects

Nothing. The developer operates no server, relay, account system, telemetry, advertising,
analytics or crash-reporting service for this app, and receives no files, results, keys, device
identifiers or usage data from it.

## Third-party processing

Content sent to a provider is processed under the user's relationship with that provider:

- [Google Privacy Policy](https://policies.google.com/privacy) and the
  [Gemini API terms](https://ai.google.dev/gemini-api/terms)
- [OpenAI Privacy Policy](https://openai.com/policies/privacy-policy/)
- [Anthropic Privacy Policy](https://www.anthropic.com/legal/privacy)
- [Mistral AI Privacy Policy](https://mistral.ai/terms/#privacy-policy)

Providers' rules on keeping inputs, and on using them for training, differ between free and
paid plans; check them before sending sensitive material. Model downloads are covered by the
[Hugging Face privacy policy](https://huggingface.co/privacy), and YouTube by the
[Google Privacy Policy](https://policies.google.com/privacy).

## Children

The app is not directed to children, and the developer knowingly collects no personal
information from anyone.

## Changes to this policy

Material changes are published here with a new effective date. The policy that applies to a
release is the one dated for it in the repository.

## Contact

For privacy questions, open an issue at <https://github.com/fmadore/Scriptorium/issues> or use
the support contact shown on the Microsoft Store listing.
