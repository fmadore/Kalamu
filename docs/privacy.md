---
layout: page
current: privacy
title: Privacy policy
lede: What the app sends, to whom, and what it keeps on your computer.
description: Kalamu's privacy policy. What the app sends, to whom, and what it keeps on the computer.
---

**Effective date:** 25 November 2026. For earlier versions and what changed in each, see
[Changes to this policy](#changes-to-this-policy) below.

## What this app is

Kalamu is a desktop application for Windows and macOS built for researchers. It extracts text from
scanned or handwritten documents, corrects existing text, builds detailed tables of contents for
long documents, transcribes audio and video, and analyses texts, using whichever mode you choose
for each job:

- **Cloud AI providers** configured with your own API key: Google Gemini, OpenAI, Anthropic
  Claude, or Mistral; or
- **Offline on your computer**: Whisper for transcription, Tesseract and small OCR vision models
  (LightOnOCR-3, GLM-OCR) for reading page images, and small language models (Gemma 4 E4B and
  E2B) for analyses, tables of contents, corrections, and translations.

The developer operates no server for Kalamu and receives no data from it.

## Files you choose

Kalamu reads only the files and folders you add to a job, and the YouTube links you paste.

**First-use confirmation for cloud providers.** Before sending your first job, workflow run, or
dictionary merge request to any cloud provider, Kalamu asks once for confirmation. The prompt
names the provider, explains what will be sent (pages, recordings, or text, including any
personal information they contain), notes that requests run under your own account and the
provider's terms, and links to this policy. Nothing is sent to a provider until you confirm;
your choice is saved in your settings. Offline engines never ask because they send nothing.

**With a cloud provider**, each page, audio segment, or text passage is sent directly from your
computer to that provider together with the task instructions (the prompt and, for analyses,
your field definitions), processed under your own account and the provider's terms. Before
sending, Kalamu may convert an input into a format the provider accepts (for example, splitting a
PDF into pages or cutting a recording into segments) without modifying your original files. For
text correction or translation, Kalamu sends only the extracted text (from a PDF text layer or
text file) rather than the original file. For a detailed table of contents, each page goes to
the page-reading provider, and the resulting page entries go as text to the merging provider
(which may be a different provider). For a YouTube link, Gemini fetches the public video
directly from YouTube; Kalamu does not download it. When you ask a model to **propose merges**
in a project dictionary, Kalamu sends only the extracted names and terms for the chosen field,
the titles of up to three documents mentioning each name, and any pairs you marked to keep
apart, never the documents themselves. This runs as its own job with a cost estimate upfront,
only when you request it.

**In cheaper mode (economy)**, pieces are submitted to your provider's batch service rather than
sent one by one. The provider retains the batch, its inputs, and its answers in your account for
a period set by its own rules (from roughly a day to several weeks), and Kalamu collects the
results when you next open the app. At OpenAI and Mistral, batches also create files in your
account storage (requests, answers, and errors). Once the answers are saved on your computer, or
the batch is cancelled or fails, Kalamu deletes only the files belonging to that batch; it never
lists or modifies anything else in your account. If the provider cannot be reached, Kalamu
retries on subsequent launches a few times, after which the files remain until removed by the
provider's retention policy (30 days at OpenAI; Mistral states no expiry) or deleted manually.
Batches created by earlier versions are not cleaned up automatically. A Claude batch creates no
separate files, though Anthropic retains the batch itself (requests and answers) for up to 29
days; Kalamu deletes finished, cancelled, or failed Claude batches in the same way, retrying on
later launches if needed (batches from earlier versions are not deleted). A Gemini batch under
20 MB is sent in a single request and leaves no file, governed by Google's batch retention rules.
Larger Gemini batches are uploaded as a request file to your Gemini API file storage, and Google
writes the answers to a second file there. Google deletes uploaded files after 48 hours and does
not publish a retention period for answer files; Kalamu deletes both once the answers are saved
locally or the batch is cancelled, fails, or expires (and deletes any orphaned upload
immediately or at next launch).

**Offline**, Whisper converts and transcribes recordings locally; Tesseract and local OCR models
render and read PDF pages and images locally; and local language models read and analyse texts
and page images locally. Nothing is uploaded. The OCR and language models run inside bundled
llama.cpp executables that open no network connections. On Windows, Whisper may run in
`whisper-vulkan`, a bundled helper for your graphics processor that receives audio through local
memory and likewise opens no network connection.

Source files and results may contain personal information about the people named or recorded in
them. Choose a provider whose terms suit your material, or work offline.

### Search by meaning and the project map

A project's search by meaning and document map rely on numerical representations (embeddings)
computed from the text. Nothing is computed until you choose a model and start preparation.

- **With a local model** (EmbeddingGemma 2, downloaded on request from Hugging Face), nothing is
  sent off your computer. Passages are written briefly to a scratch folder inside the app's data
  folder, read by the bundled program, and deleted.
- **With a cloud model** (Google Gemini, OpenAI, or Mistral), Kalamu asks for confirmation for
  that project and model, naming the provider and explaining what will be sent. Once you confirm
  and start preparation, the corrected text of the project's documents (extractions,
  transcriptions, corrections, and translations; not analyses) is sent to that provider in chunks
  of roughly 1,200 characters, along with the model name and your API key. Later updates send
  only text that changed, and only when you ask. Afterwards, each query you type into search by
  meaning is sent to the same provider. No filenames, project names, or source descriptions are
  sent. The 2D map and nearest-document rankings are computed locally and send nothing.
- **Stored on your computer**: the embedding vectors in `vectors.sqlite` inside the app's data
  folder (which you can clear model by model), and in the job database the chosen model, when
  you gave consent, query counts and costs, and preparation counts and costs (deleted when the
  project is deleted). Your search queries themselves are not stored.
- **Provider retention (as published on 18 November 2026)**: OpenAI states that it retains API
  inputs for up to 30 days for abuse detection and does not train on them unless the account opts
  in; Google states that paid Gemini API content is not used to improve its products and is
  logged for a limited period, whereas free-tier content may be used and reviewed by humans; for
  Mistral, see its privacy policy below. Provider terms can change, so verify them directly.

### Correcting a selected passage with a model

When you select text in the review screen and ask a model to correct it, Kalamu sends the
selected passage (up to 4,000 characters) plus up to 300 characters of surrounding context on
each side from the same page or segment, along with the correction prompt, after the same
provider confirmation used for jobs. With a local model, nothing is sent. The request, the
model's suggestion, and whether you accepted it are stored in the job database alongside the job.

### Re-reading a region of a page

When you select or draw a region on a scan and ask a model to re-read it, only that cropped
portion of the page is sent, as a PNG (or JPEG where required), to your chosen provider with the
text-extraction prompt and the job's language settings, after the same provider confirmation.
Nothing else from the document is sent, and with a local model nothing is sent at all. Drawing,
moving, or deleting regions sends nothing. The region coordinates, model, prompt, returned
reading, cost, and whether you used it are stored in the job database with the job.

### Analysing an interview transcript

When you choose *Analyse this interview*, Kalamu sends your reviewed transcript (including
corrections and speaker names), split into numbered turns with speaker labels and timestamps,
together with your research-context sentence and the analysis prompt, after the same provider
confirmation. The audio or video recording itself is never sent. With a local model, nothing is
sent. Quotations returned by the model are verified locally against the transcript and any
unmatched quotes are discarded. Your context sentence, answers, and verified quotations are
saved in the job database. If you choose to save the context sentence for future interviews, it
is stored locally with your prompts and sent with each analysis using that prompt.

Any themes or interview guide entered under *More choices* are sent alongside that sentence (and
saved with it if you choose to keep it). If a transcript is too long to fit in a single request,
it is sent in overlapping sections of whole turns, followed by one synthesis request to the same
provider containing the section answers and verified quotations. Quotations you add manually
from the transcript stay local and send nothing.

### Screening project documents against a research question

When you choose *Which of these speak of my question?*, Kalamu sends the reviewed text of each
selected document (your preferred result with corrections, or the file's own text), divided into
numbered passages, together with your research question and the prompt, after the same provider
confirmation. Scans and photographs are not sent. With a local model, nothing is sent. Cited
passages are verified locally against the document text, and any unmatched passages are
discarded. Your question, the model's answers, and the verified passages are saved in the job
database; the *Kept* and *Set aside* tags stay on your computer with the project.

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
  corrections and review marks, the names given to speakers, the regions the user draws over
  a scan or whose box they change (with the text they give them, never sent), and what each
  call cost.
  Results are kept by a fingerprint of the file and settings, so the same work is never paid
  for twice; deleting a job removes the job, its progress and corrections, while the stored
  results stay until the user clears those no job uses (Settings › General, *Clear unused
  results*) or the app is uninstalled. The same database keeps the user's **projects**:
  their names and descriptions, the source description their documents share, their
  collections, the titles, tags, source descriptions (repository, shelfmark, citation and the
  like, as the user types them), paths, sizes and fingerprints of their documents, which jobs read each document, and the results the user marked as
  preferred, what the user decided in its list of what to check (the items kept aside with
  their notes, the results set aside from the list and when, and the seed and size of each
  sample drawn at random), and its **dictionary**: the entries the user made of the names its analyses
  listed (labels, the forms they gather, review status), the pairs kept apart, each request for
proposed merges (the job that asked and the forms each of its texts held), and the proposals
turned down. The texts sent for proposals are kept with the frozen text below. A project only
  refers to files where they lie on the PC: it never copies, moves or sends them, except the
  original files a project file carried, copied in when it is opened (below). Deleting a project, a collection or a document removes how it was organised; the
  files, the jobs and their results stay, but deleting a project also removes the original
  files copied in for it. It also keeps each **workflow run**: the workflow's
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
- **Original files from a project file** (`originals`): when a `.kalamu` file that holds its
  original files is opened, its scans and recordings are copied here, in a folder of the new
  project's own, and its documents and jobs read them there. They are never sent anywhere by
  this step. Deleting the project removes that folder, keeping only a file another project's
  document refers to; the jobs it brought stay, with their results, and show those files as
  missing. Files the user keeps elsewhere are never touched. A folder whose project no longer
  exists (left by a crash, or by a version before 25 November 2026) is removed when the app
  next starts.
- **Working copies of older project files** (`project-file-scratch`): opening or checking a
  `.kalamu` file saved by an earlier version of Kalamu brings a copy of it up to date here,
  original files included, never changing the user's file; the copy is removed when done, and
  what a crash or a forced quit left is removed when the app next starts. Versions before
  25 November 2026 made this copy in the system's temporary folder.
- **Recording pieces converted for listening** (`listening`): the review's player plays most
  recordings straight from the original file; for a format it cannot play, the piece is
  converted on the computer and the copy kept, so it plays at once next time. The folder is
  kept under 256 MB by removing the copies heard longest ago.
- **Table files exported** (`exports.json`): the path of each spreadsheet, CSV and JSON file a
  table export (descriptions, an analysis's table, the review list, the dictionary) wrote, with
  a SHA-256 fingerprint of its bytes, never its contents. A file whose bytes still match is
  replaced by the next export to the same name; any other file of that name is left as it is.
  The last 500 files are remembered, the oldest forgotten first.
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

### A project saved to a file

*Save to a file…* writes one `.kalamu` file where the user chooses. It holds the names and
descriptions of the project's documents, the text the models read from them, the models'
answers, the user's corrections, review marks, tags, what they kept or set aside, the project
dictionary and what each request cost. When the user ticks *Include the original files*, it
also holds their scans and recordings. It never holds API keys, settings or anything from the
user's other projects. Kalamu sends the file nowhere: it leaves the computer only when the
user moves, copies or sends it. Whoever receives it can read everything in it, so it deserves
the care the documents themselves do. Opening a project file sends nothing either; the
original files it holds are copied into the app's data folder on that computer (`originals`),
and removed from it when the project is deleted.

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

Kalamu includes an optional *Buy me a coffee* link to the developer's page on Buy Me a Coffee
(`https://buymeacoffee.com/fmadore`); the Mac App Store build omits this link. Kalamu passes
that fixed URL to your default browser and nothing else: it sends no information about you or
your device and contacts no payment service itself. Anything entered on that page is between you
and Buy Me a Coffee under the
[Buy Me a Coffee privacy policy](https://buymeacoffee.com/privacy-policy). Supporting
development is optional and does not unlock additional features.

## Report a problem

The flag icon in each review header and *Report a problem* in Settings › About open GitHub's
public issue forms (`https://github.com/fmadore/Kalamu/issues/new/choose`) in your default
browser (for bugs, feedback, or offensive or harmful output). Kalamu passes only that address to
the browser: it sends no document, result, key, version, or other information itself. Anything
you write in the form, along with your GitHub account, is published on GitHub under the
[GitHub privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement);
the form reminds you not to paste confidential material.

## What the developer collects

Nothing. The developer operates no server, relay, account system, telemetry, advertising,
analytics, or crash-reporting service for Kalamu, and receives no files, results, keys, device
identifiers, or usage data from it. A problem reported through GitHub reaches the developer only
as you wrote it there.

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

Kalamu is not directed at children, and the developer knowingly collects no personal information
from anyone.

## Changes to this policy

Material changes are published here with a new effective date. The policy that applies to a
given release is the one dated for it in the repository.

<details>
<summary>Previous versions of this policy</summary>

- **25 November 2026**: deleting a project opened from a `.kalamu` file now removes the original files copied in for it (`originals`), and the copy of an older project file brought up to date is made in the app's data folder (`project-file-scratch`) instead of the system's temporary folder; what a crash left in either is removed at the next start.
- **23 November 2026**: records paths and SHA-256 fingerprints of exported table files (`exports.json`) so re-exporting never overwrites user-modified files.
- **20 November 2026**: adds screening project documents against a research question, multi-part interview analysis, custom interview themes/guides, and saving projects to portable `.kalamu` files.
- **19 November 2026**: adds interview analysis from transcripts (sending transcript text, speaker names, timestamps, and research context to the chosen provider).
- **18 November 2026**: adds search by meaning and the project document map, model-assisted correction of selected passages, user-drawn scan regions and region re-reading, uploading large Gemini batches as files (deleted afterwards), and storing items set aside from *What to check*.
- **17 November 2026**: adds storing review-list notes, random-sample seeds and sizes, exporting the review list, and deleting finished Anthropic Claude batches.
- **15 November 2026**: adds deleting OpenAI and Mistral batch files from account storage once answers are saved locally.
- **11 November 2026**: adds text translation (sending text to the chosen provider in the same way as text correction).
- **10 November 2026**: adds `offline-scratch` temporary files, resumable `.part` model downloads, GitHub-release update notices from the existing catalogue file, preserving unreadable settings as `settings.json.bad`, 30-day log retention, and *Clear unused results*.
- **7 November 2026**: adds offline language models (Google Gemma 4 downloaded from Hugging Face).
- **6 November 2026** (version 1.5.0): adds offline OCR vision models.
- **4 November 2026 (second revision, version 1.4.1)**: adds per-provider first-use confirmation, Mac App Store security-scoped bookmarks (`grants.json`) and container paths, and notes file paths in logs.
- **4 November 2026 (first revision, version 1.4.0)**: adds offline text extraction with Tesseract.
- **3 November 2026 (version 1.3.0)**: adds the local project search index (`search.sqlite`).
- **2 November 2026 (four revisions, including version 1.2.0)**: adds macOS coverage, renames the data folder from the bundle identifier to `Kalamu Data`, and adds frozen item pages (`snapshots`).
- **1 November 2026**: adds dictionary merge proposals and projects/workflows.
- **30 October 2026** (version 1.0.0): initial Microsoft Store release policy.

</details>

## Contact

For privacy questions, open an issue at <https://github.com/fmadore/Kalamu/issues> or use
the support contact shown on the Microsoft Store listing.
