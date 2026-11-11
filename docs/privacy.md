# Privacy Policy — Kalamu

**Effective date:** 11 November 2026, which applies from the first version to translate
texts: the text a translation reads is sent as a correction's is, to the provider the user
chose. The policy of 10 November 2026 did not say so; it applies from the first version to write the
offline models' working files (a page image, a text being read) in a folder of the app's own,
emptied at each start, and to keep an unfinished model download so that it continues where it
stopped; to tell a copy installed from the releases page that a newer version is out, from the
catalogue file it already downloads (nothing new is fetched or sent); to keep a settings file
it cannot read beside the new one rather than replace it; to keep a month of logs; and to let
the user clear the stored results no job uses. The policy of 7 November 2026 did not say so;
it applies from the first version to offer the offline language models: small models from
Google downloaded from Hugging Face at the user's request and run on the computer, for
analyses, tables of contents and corrections. The policy of 6 November
2026, which applies to version 1.5.0, did not list them; it added the offline OCR models. The
second policy of 4 November 2026, which applies to version
1.4.1, did not list them; it added the question asked before files are first sent to each cloud
provider; for the Mac App Store build, the folders the user grants it, kept as bookmarks, and
its data folder inside the app's own container; and the file paths the logs may hold. Version 1.0.0, the first Microsoft
Store release, was published under the policy
of 30 October 2026, which did not list projects or workflows; the policy of 1 November 2026 did
not list the dictionary's proposed merges, the first of 2 November 2026 did not cover macOS,
the second named the data folder after the app's bundle identifier, the third did not list the
items' pages, the fourth, which applies to version 1.2.0, did not list the search index, and
that of 3 November 2026, which applies to version 1.3.0, did not list Tesseract, and the first
of 4 November 2026, which applies to version 1.4.0, did not cover the Mac App Store build.

## What this app is

Kalamu is a desktop application for Windows and macOS, for researchers. It extracts text from scanned or
handwritten documents, corrects existing text, makes detailed tables of contents of long
documents, transcribes recordings and videos, and analyses texts, in one of two ways the user
chooses for each job:

- through a cloud AI provider the user selects and configures with their own API key: Google
  Gemini, OpenAI, Anthropic Claude or Mistral; or
- offline, on the user's computer: Whisper for transcription, Tesseract and small OCR vision
  models (LightOnOCR-3, GLM-OCR) for text extraction from page images, and small language
  models (Gemma 4 E4B and E2B) for analyses, tables of contents, corrections and translations.

The developer operates no server for the app and receives nothing from it.

## Files the user chooses

Kalamu reads only the files and folders the user adds to a job, and the YouTube links the
user pastes.

**Before the first job, workflow run or dictionary proposal sent to a cloud provider**, the app
asks once, naming the provider, saying what will be sent (the pages, recordings or text, with
any personal information they hold), that it goes under the user's own account and the
provider's terms, and where this policy is. Nothing is sent to a provider the user has not
agreed to; the answer is kept in the settings. Offline engines are never asked about, since
they send nothing.

**With a cloud provider**, each page, recording segment or text is sent directly from the
user's computer to that provider, with the instructions for the task (the prompt and, for an
analysis, the fields to answer), and processed under the user's account and the provider's
terms. Before sending, the app may convert a piece into a form the provider accepts, for
example cutting a PDF into pages or a recording into segments; the original files are never
changed. To correct or translate existing text, the app sends the text it reads from the file
(a PDF's text layer, or a text file's pages or passages) rather than the file. For a detailed table of
contents, each page goes to the provider chosen to read the pages, and the entries read on
them, as text, to the provider chosen to combine them, which may be another provider. With a
YouTube link, Gemini fetches the public video from YouTube itself; Kalamu does not download the
video. When the user asks a model to **propose merges** in a project's dictionary, the app
sends that provider the names and terms of the field chosen, as the analyses wrote them, with
the titles of up to three of the project's documents naming each, and the pairs the user kept
apart; not the documents themselves. This is sent as a job of its own, with an estimate first,
only when the user asks.

**In the cheaper mode** (economy), pieces are sent to the provider's batch service instead of
one by one. The provider keeps the batch, its input and its answers for a time set by its own
rules (from about a day to several weeks), in the user's account, and Kalamu collects the
answers when it next opens.

**Offline**, with Whisper, recordings are converted and transcribed on the computer and never
uploaded; with Tesseract or an offline OCR model, PDF pages and images are drawn and read on the
computer and never uploaded; with an offline language model, texts and page images are read
and analysed on the computer and never uploaded. The OCR and language models run in llama.cpp's
programs, shipped with the app, which open no connection; on Windows, Whisper may run in
`whisper-vulkan`, a program shipped with the app for the graphics processor, which receives the
audio through the computer's own memory and opens no connection either.

Files and results may contain personal information about the people named or heard in them.
Choose a provider whose terms suit the material, or work offline.

## What the app keeps on the computer

In the app's local data folder, which Settings shows: on Windows `%LOCALAPPDATA%\Kalamu Data`
for the installer, and the same folder inside the package's own for the Store app
(`%LOCALAPPDATA%\Packages\49346FMadore.Kalamu_…\LocalCache\Local\Kalamu Data`); on macOS
`~/Library/Application Support/Kalamu`, with the logs in `~/Library/Logs/Kalamu`, and for the
Mac App Store app the same folders inside its container
(`~/Library/Containers/io.github.fmadore.kalamu/Data/Library/…`). Versions 1.0
to 1.1.2 named the folder after the app's bundle identifier, `io.github.fmadore.kalamu`, and
the Windows test releases up to 0.2, named Scriptorium, kept it in
`%LOCALAPPDATA%\io.github.fmadore.scriptorium`; the first start of a newer version moves that
folder to the new name, and the keys stored under Scriptorium's name in Credential Manager to
Kalamu's, on the same computer only:

- **The job database** (`jobs.sqlite`): each job's name, the paths, sizes, modification times
  and fingerprints of its files, its settings and progress, the text and answers returned for each piece, the user's
  corrections and review marks, the names given to speakers, and what each call cost.
  Results are kept by a fingerprint of the file and settings, so the same work is never paid
  for twice; deleting a job removes the job, its progress and corrections, while the stored
  results stay until the user clears those no job uses (Settings › General, *Clear unused
  results*) or the app is uninstalled. The same database keeps the user's **projects**:
  their names and descriptions, the source description their documents share, their
  collections, the titles, tags, source descriptions (repository, shelfmark, citation and the
  like, as the user types them), paths, sizes and fingerprints of their documents, which jobs read each document, and the results the user marked as
  preferred, and its **dictionary**: the entries the user made of the names its analyses
  listed (labels, the forms they gather, review status), the pairs kept apart, each request for
proposed merges (the job that asked and the forms each of its texts held), and the proposals
turned down. The texts sent for proposals are kept with the frozen text below. A project only
  refers to files where they lie on the PC: it never copies, moves or sends them. Deleting a project, a collection or a document removes how it was organised; the
  files, the jobs and their results stay. It also keeps each **workflow run**: the workflow's
  steps and prompts as they were when it started, the paths, sizes and fingerprints of the
  files it was started on, the job that ran each step, and when the user chose to continue
  after a step. Forgetting a run removes this record; its jobs and their results stay.
- **The search index** (`search.sqlite`): a copy of the text of the results a project search
  has looked through (the corrected text, the speakers' names, and analyses' answers when they
  were searched), folded for matching, with where each passage stands. It is made from the job
  database and never sent; deleting a job removes its text from the index, and *Prepare the
  text again* in a project's search empties it, to be made again at the next search. Searches
  themselves are not kept.
- **Frozen text** (`snapshots`): when the user chooses *Use this text in a new job*, or a
  workflow step reads an earlier step's text, a copy of the corrected text it reads, so that
  later corrections do not change what it read. It is sent only to the provider of the job or
  step that reads it, as the text of any document would be, and kept until the app is
  uninstalled.
- **Items' pages** (`snapshots`): when the user sends items of a table of contents (an
  article, a letter) to a new job, a PDF of each item's pages, cut from the user's file, so
  that a later change to the item does not change what the job read. It is sent only to the
  provider of that job, as the file itself would be, and kept until the app is uninstalled.
  The items themselves (their titles, pages and fields) are kept in the job database with the
  table of contents, and deleted with it.
- **Requests for proposed merges** (`dictionary`): the `provenance.json` of each, naming the
  model, the prompt and what the request cost, in a folder per project. Nothing else is written
  for them: the answers are kept in the job database.
- **Settings** (`settings.json`): preferences such as formats, destination, theme, favourite
  languages, the last folder chosen, and the cloud providers the user agreed to send files to.
  A settings file the app cannot read is moved aside as `settings.json.bad` (with a number
  when an earlier one is there), never overwritten, and the app says where; it stays until
  the user deletes it.
- **Prompts, pipelines and workflows** the user saves (`presets`, `pipelines`, `workflows`),
  and a copy of the model catalogue. An exported workflow, like an exported pipeline, holds its
  steps, models, settings and prompts, never keys, files, folders or results.
- **Export profiles** the user saves (`export-profiles`): the columns of a descriptions export,
  their headings and any fixed text typed for every row. A shared profile holds its columns,
  and its fixed texts only when the user asks; never a project's descriptions.
- **Whisper models**, downloaded only when the user asks (`whisper-models`). A download that
  stops (the connection lost, cancelled, the app closed) keeps what arrived beside the model,
  under its name ending in `.part`, so the next download continues it; removing the model
  removes it too. This holds for every model and language file below.
- **Tesseract language data**, one file per language, downloaded only when the user asks
  (`tesseract-data`).
- **Offline OCR models**, each with its image encoder, downloaded only when the user asks, and
  the seconds a page took with each on this computer, on the processor and, on Windows, on the
  graphics processor, named as its driver names it, and on a Snapdragon the graphics
  processor's compiled programs, so later pages need not compile them again (`ocr-models`).
- **Offline language models**, each with its image and audio encoder, downloaded only when the
  user asks (`language-models`).
- **The offline models' working files** (`offline-scratch`): while an offline OCR or language
  model runs, the page image it reads, or the text, its instructions and the fields asked for,
  are written there, and removed once it has answered. What a crash or a forced quit left is
  removed when the app next starts.
- **Recording pieces converted for listening** (`listening`): the review's player plays most
  recordings straight from the original file; for a format it cannot play, the piece is
  converted on the computer and the copy kept, so it plays at once next time. The folder is
  kept under 256 MB by removing the copies heard longest ago.
- **Folders and files the user granted** to the Mac App Store app (`grants.json`): each path
  with the bookmark macOS gives for it, so the app can reach the folder again at the next launch
  without asking. A folder out of reach at launch (a disk not connected) is kept for the next.
  The other builds keep nothing of the kind.
- **Logs**, for troubleshooting: what the app did and the errors it met, with job identifiers
  and the paths of the files and folders concerned, and, when an offline engine fails, the last
  lines of its diagnostics, only those that read as a program's own messages. They contain no
  API keys and no document text. One file is written a day, and those older than thirty days
  are deleted.

Results are also written as files where the user chooses: next to the original files or in a
chosen folder, each with a `provenance.json` that records the model, prompt and settings used
and, for a source that is a document of a project, the project's name, the document's title
and the description of its source the user typed.

Uninstalling the Microsoft Store app removes its local data folder. On macOS, moving the app
to the Bin leaves its data folder, and the Mac App Store app's container, until the user
deletes them. Exported files stay where they were written.

## API keys

No key is needed for offline transcription, text extraction or analysis. Provider keys are stored in the system's keychain
(Windows Credential Manager, or the login Keychain on macOS) and read only by the app's native
core. A key is sent only to the provider that issued
it, to authenticate requests. It is never written to the job database, the logs or
`provenance.json`, and the developer does not receive, copy or store it.

## Other connections

Besides the provider the user chooses, the app connects to:

- **GitHub** (`raw.githubusercontent.com`), at startup, to check for a newer catalogue of
  models and prices. It downloads a public file and sends nothing about the user. The same
  file names the latest version of Kalamu: a copy installed from the releases page says when
  a newer one is out, with a link to its page on GitHub that opens in the browser only if the
  user follows it. Nothing more is fetched or sent for it, and the Microsoft Store and Mac App
  Store apps, which their stores keep up to date, never say it.
- **Hugging Face** (`huggingface.co`), only when the user downloads a Whisper model, an
  offline OCR model or an offline language model, from a fixed version of the model's repository
  there (for the OCR models, `prithivMLmods/LightOnOCR-3-1B-GGUF` and `ggml-org/GLM-OCR-GGUF`;
  for the language models, Google's `google/gemma-4-E4B-it-qat-q4_0-gguf` and
  `google/gemma-4-E2B-it-qat-q4_0-gguf`). The download is checked
  against its size and SHA-256 before use, and nothing about the user is sent.
- **GitHub** (`raw.githubusercontent.com`), only when the user downloads Tesseract's data for a
  language, from the Tesseract project's own repository at a fixed version. The download is
  checked against its size and SHA-256 before use, and nothing about the user is sent.
- **YouTube**, when a link is added: the app asks YouTube's public oEmbed service for the
  video's title. When the user plays a video in the review, YouTube's privacy-enhanced player
  (`youtube-nocookie.com`) loads inside the app; until then, opening a review contacts no one.

These services receive ordinary request information, such as the computer's IP address. When a
proxy is set in the system's network settings, connections go through it.

## Running in the background

Kalamu does not start with the computer. When the user closes the window or quits during a
job, the app asks whether to keep the job running; if so, it stays in the notification area
(the menu bar on macOS) until the job ends or the user quits. On macOS, closing the window
leaves the app open in the Dock, as Mac apps do, until the user quits it. Jobs waiting for a provider's batch resume when the app is next opened.

## Support link

Kalamu shows an optional *Buy me a coffee* link to the developer's page on Buy Me a Coffee
(`https://buymeacoffee.com/fmadore`); the Mac App Store app has none. The app passes that fixed address to the default browser
and nothing else: it sends no information about the user or the device and contacts no payment
service itself. Anything entered on that page is between the user and Buy Me a Coffee, under the
[Buy Me a Coffee privacy policy](https://buymeacoffee.com/privacy-policy). Supporting is optional
and unlocks nothing.

## Report a problem

The flag in each review's header and *Report a problem* in Settings › About open GitHub's choice of
public issue forms (`https://github.com/fmadore/Kalamu/issues/new/choose`) in the default
browser: a bug, feedback, or output that is offensive or harmful. The app passes that address to
the browser and nothing else: it sends no document, result, key, version or other information
itself. What the user writes in the form, and
their GitHub account, are published on GitHub under the
[GitHub privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement);
the form asks not to paste confidential material.

## What the developer collects

Nothing. The developer operates no server, relay, account system, telemetry, advertising,
analytics or crash-reporting service for this app, and receives no files, results, keys, device
identifiers or usage data from it. A problem reported through GitHub reaches the developer only
as the user wrote it there.

## Third-party processing

Content sent to a provider is processed under the user's relationship with that provider:

- [Google Privacy Policy](https://policies.google.com/privacy) and the
  [Gemini API terms](https://ai.google.dev/gemini-api/terms)
- [OpenAI Privacy Policy](https://openai.com/policies/privacy-policy/)
- [Anthropic Privacy Policy](https://www.anthropic.com/legal/privacy)
- [Mistral AI Privacy Policy](https://mistral.ai/terms/#privacy-policy)

Providers' rules on keeping inputs, and on using them for training, differ between free and
paid plans; check them before sending sensitive material. Model downloads are covered by the
[Hugging Face privacy policy](https://huggingface.co/privacy), the catalogue and Tesseract's
language data by the
[GitHub privacy statement](https://docs.github.com/site-policy/privacy-policies/github-general-privacy-statement),
and YouTube by the
[Google Privacy Policy](https://policies.google.com/privacy).

## Children

The app is not directed to children, and the developer knowingly collects no personal
information from anyone.

## Changes to this policy

Material changes are published here with a new effective date. The policy that applies to a
release is the one dated for it in the repository.

## Contact

For privacy questions, open an issue at <https://github.com/fmadore/Kalamu/issues> or use
the support contact shown on the Microsoft Store listing.
