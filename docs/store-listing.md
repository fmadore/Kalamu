# Store listing text

Paste-ready text for Kalamu's two store listings, kept for the **current submission**, version
1.4.1: the Microsoft Store (Partner Center) first, whose `Kalamu_1.4.1.msixbundle` the release
workflow builds (the first, 1.0.0, was certified on 2 November 2026), then the
[Mac App Store](#mac-app-store-app-store-connect) (App Store Connect), whose first submission is
1.4.1. The submission and update steps are in [microsoft-store.md](microsoft-store.md) and
[mac-app-store.md](mac-app-store.md). Check every block against the app as packaged before
pasting: a listing that promises what a reviewer cannot find fails certification.

Every paragraph inside a block is one unwrapped line, because Partner Center keeps line breaks;
blank lines separate paragraphs. Keep it that way when editing, and do not let a formatter
re-wrap the blocks.

Microsoft's [listing field documentation](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
allows 10,000 characters for Description, 1,500 for What's new, and 20 features of 200
characters each. Short description accepts 1,000 characters, but only the first 270 show in
some views; the one below fits. Keywords allow up to 7 entries of at most 40 characters, with
no more than 21 words across all of them.

**Languages.** English (United States) only for now. The interface is in English and French
from 1.3.0: a French listing, with screenshots in French, can be added with its submission, and
German once the interface is translated.

**Before pasting**, replace the provider model names if the catalogue has changed, and check
that the features listed still match the app.

## English (United States)

### Description — full text

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider of your choice, then check every result against its source. Kalamu works with Google Gemini, OpenAI, Anthropic Claude and Mistral under your own account and API key, so their usage charges apply and are billed by them; the app shows an estimated cost before anything is sent. Recordings can also be transcribed offline with Whisper, and clean print read offline with Tesseract, free, after a one-time download.

Extract text from PDFs and images, printed or handwritten: newspapers, reports, letters, registers and manuscripts. Keep the lines as on the page, to check against the scan, or join them into running text for reading and searching. Before a job starts, Kalamu checks the files and suggests fixes: double pages to split, scans whose resolution is too low, recordings that are too quiet.

Transcribe recordings and videos with speakers and timestamps, and give the speakers their real names. Long recordings are cut at pauses and sent piece by piece. Offline, Whisper transcribes on your PC with no account, key or internet connection, and nothing leaves the computer; speaker labels need a cloud model. With Gemini, public YouTube videos are transcribed from their link, without downloading them.

Make a detailed table of contents of a long document: a newspaper or magazine issue, an archival file scanned in one piece, a book or report. One model reads each page, another, cheaper one combines the pages into one entry per article or item, with its pages and the fields you choose. Correct the noisy text that digital libraries supply, page by page, with every change shown against the original.

Analyse texts with fields you define, as you would brief a research assistant: a summary, the people, places and organisations a document names, its tone with a reason, themes from your own list. Ready-made analyses cover common needs, including the question, argument, methods and findings of a scholarly publication for a literature review. Answers come back in the shape you asked for and can be exported as a spreadsheet with a column per field.

Check every result beside its source: the page next to its text, or the recording next to its transcript, line by line. Corrections are kept apart from the AI output, which is never overwritten, and you mark what you have reviewed. Search a document or a recording across all its pages, corrections included, and go to each match. A glossary explains the words these tools use, from OCR and tokens to error rates and hallucination, for colleagues new to them. Compare two or more models on the same documents: their differences are highlighted in the text, with error rates against the pages you have corrected.

Large jobs are safe and affordable. Work is saved piece by piece: a job interrupted by closing the app, a crash or a lost connection resumes where it stopped, and nothing is sent or paid for twice. Long jobs keep running from the notification area while the window is closed. For large collections, choose the cheaper mode: the provider's batch service answers within a day at about half the price, and Kalamu collects the results when it next opens.

Export to text, Markdown, Word, JSON, CSV, subtitles (SRT and WebVTT), and searchable PDF and ALTO XML, with Tesseract or Mistral OCR. Each output comes with a provenance.json file recording the model, prompt and settings, so your methods can be described and repeated. Save a job's settings as a pipeline to start the next batch in one click, and export it with its full prompts to cite in a methods section or share with colleagues. Chain pipelines into a workflow, extraction then analysis for instance, that stops for your review before the next step reads the text.

Keep a project's material together: documents in collections and tags, each with every result read from it whatever job made it, and the one you prefer marked. Search the text of all its documents at once, corrections included, and open each passage where it stands. A project dictionary gathers the people, places and terms your analyses listed; merge the spellings that are one person under your own label, keep two apart, and let a model propose merges that you accept or turn down.

Before the first job sent to each provider, Kalamu says what will be sent and asks you. Your files go straight from your PC to the provider you choose; offline, nothing leaves the PC. API keys are kept in Windows Credential Manager. The developer receives no files, results, keys or telemetry, and the app sells no subscription, credits or API access. Every result is text generated by the AI model you choose: it can be fluent and wrong, so check a sample against the source before relying on it, and use Report a problem in the review or in Settings to tell the developer about inappropriate output. An optional Buy Me a Coffee link supports development and unlocks nothing.

Requires Windows 11 with the Microsoft Edge WebView2 Runtime, with native x64 and ARM64 packages. The interface is in English and French; documents and recordings can be in any language the chosen model reads. Free and open source under the MIT licence.

Google Gemini and YouTube are trademarks of Google LLC; OpenAI is a trademark of OpenAI OpCo, LLC; Claude is a trademark of Anthropic, PBC; Mistral AI is a trademark of Mistral AI; Microsoft Word is a trademark of the Microsoft group of companies. Kalamu is an independent project, not affiliated with or endorsed by any of them.
```

### What's new in this version — 1.4.1

```text
Before the first job sent to each AI provider, Kalamu now says what will be sent to it and asks you, once for each provider. Whisper and Tesseract, which send nothing, are never asked about.

A page typed by hand reaches ALTO and the searchable PDF; word positions are no longer guessed and shown as measured; photos taken upside down are turned upright before they are read.

The window no longer freezes while a correction or an export is saved during a running job. Dialogs keep the keyboard inside, and a file dropped with its folder is listed once.
```

The 1.4.0 text, for the record:

```text
Extract text offline with Tesseract, for clean print: free, nothing sent, each word with its place on the page. Download a language once in Settings.

Export ALTO XML with each block, line and word in its place, corrections included, from Tesseract or Mistral OCR.

All jobs: one page to search, filter and sort every job, and to rename, add to a project or delete several at once.

The whole window for every view, larger type, and a smoother review: zoom the scan about the pointer, both texts move together, and the readings box can be moved aside.

Also: write a new prompt from nothing; projects in the sidebar, with jobs dragged onto them; Settings in tabs.
```

The 1.3.0 text, for the record:

```text
In French: choose the interface's language in Settings, or let it follow the system's.

Search a whole project: every passage that holds your words, in the text as you corrected it, each opening its review at that place. The search runs on your PC; nothing is sent.

What to check: one list of the pages and answers that still need you, with the reason for each, from a project, a collection or a selection.

Describe each source as an archive does, and export the descriptions in your own columns (Dublin Core terms included), beside each document's corrected text, review status or analysis answers.

Also: Gemini 3.5 Transcribe names its speakers and shows its cost; OpenAI's prompt caching costs less; CSV files open in Excel with their accents; rename a tag across a project; Report a problem leads with bugs and ideas.
```

The 1.2.0 text, for the record:

```text
Items of a table of contents: turn the entries of a newspaper issue or an archival file into items, correct, join or split them, then extract, analyse or export one article or letter on its own.

Simpler prompts: one default per kind of material, for printed, newspaper and handwritten pages in any language and reading direction, Arabic and Ajami included. Your saved pipelines and workflows keep working.

Your jobs and settings now live in a folder named Kalamu Data, moved there once on first start; Settings shows where.

Also: New job shows which models also give a searchable PDF; Mistral Large 4 at its current, lower price; clearer wording across the interface.
```

The 1.1.2 text, for the record:

```text
Projects: gather documents in collections and tags, each with every result read from it, whatever job made it, and the one you prefer.

A project dictionary of the people, places and terms your analyses listed: merge spellings under your own label, keep two apart, export as JSON and CSV. A model can propose merges; nothing is merged until you accept.

Workflows: run pipelines one after another on the same files, stopping for your review before the next step reads the text. Then, at the end of New job, analyses the text once you have checked it.

Also: send any finished text into a new analysis or correction without reading the originals again; several pages per call for text extraction; a thinking level for models that reason; Gemini 3.5 Transcribe, Google's speech-to-text model, for recordings; a menu on each job in the list to rename it, add it to a project or delete it; definitions in a bubble beside each word; corrections opened where you are reading; and fixes to cost counting, workflow steps and zoomed pages.
```

The 1.0.0 text, for the record: first release in the Microsoft Store; search in the review
(Ctrl F); the glossary; Report a problem; one layout for every review; cut-off answers marked;
changed files refused; files written despite a failing page; the third-party licences.

### Features — one feature per line

```text
Extract text from scanned and handwritten PDFs and images with Gemini, OpenAI, Claude or Mistral
Transcribe recordings and videos with timestamps, in the cloud with speakers or offline
Offline, with no account, key or cost: transcription with Whisper, and text extraction of clean print with Tesseract
Transcribe public YouTube videos from their link with Gemini, without downloading them
Analyse texts with your own fields or ready-made ones: people, places, tone, a publication's argument
Make detailed tables of contents of newspaper issues, archival files and books, then read each article on its own
Correct existing OCR text page by page, every change shown against the original
Check every result beside its source, page by page or line by line; corrections never overwrite the AI's text
Search a document, a recording or a whole project at once, corrections included, and open each passage in its source
Projects: collections and tags of documents, each with every result read from it
A project dictionary of the names your analyses found, with merges a model proposes and you decide
Compare models on the same documents, with differences highlighted and error rates
Estimated cost shown before anything is sent, from page counts and published prices
Cheaper mode: send a large job to the provider's batch service at about half the price
Interrupted jobs resume where they stopped, nothing paid twice; long jobs run with the window closed
Save a job's settings as a pipeline, or chain pipelines into a workflow with review stops
Export to text, Markdown, Word, JSON, CSV, SRT, WebVTT, searchable PDF and ALTO XML, each with a provenance file
File checks for double pages, low resolution and quiet recordings, with suggested fixes
API keys kept in Windows Credential Manager; files go straight to the provider you choose
Light and dark themes; native Windows x64 and ARM64; free and open source
```

### Short description

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with Gemini, OpenAI, Claude or Mistral under your own key, or offline with Whisper and Tesseract. Check every result beside its source; corrections never overwrite the AI's text.
```

### Keywords — up to 7

```text
handwritten text recognition
OCR
interview transcription
speech to text
qualitative research
digital humanities
historical documents
```

## Additional system requirements

Enter each line as a separate requirement (under 200 characters):

```text
Windows 11 with the Microsoft Edge WebView2 Runtime; native x64 and ARM64 packages
Cloud features: internet and your own Gemini, OpenAI, Anthropic or Mistral account and API key; the provider bills usage
Offline transcription: internet once to download a Whisper model (about 60 MB to 1.1 GB), then runs on the CPU
Offline text extraction: internet once to download each language for Tesseract (about 3 to 15 MB; 100 MB for languages in Latin letters without their own model), then runs on the CPU
```

Recommended memory: 8 GB; Whisper's larger models need more, and their speed depends on the
processor.

## Notes for certification

Fill in the bracketed parts at submission. These notes are seen only by the certification
team; a test key may be given here, never in the listing.

```text
Product: Kalamu. Version: 1.4.1.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Kalamu extracts text from documents, transcribes recordings and analyses texts with an AI provider the user configures with their own API key (Google Gemini, OpenAI, Anthropic Claude or Mistral), or works offline with Whisper (transcription) and Tesseract (text extraction of print). The app supplies no cloud account or credits; provider usage is billed to the user's own account, as the listing states.

Sample files for testing: https://github.com/fmadore/Kalamu/releases/download/v1.4.1/kalamu-samples.zip, with a README.txt: a 20-second spoken recording (interview.wav, made with Windows speech synthesis), a handwritten French report of 1755 (dodington-1755.jpg, Bibliothèque nationale de France, public domain), a 1911 university newspaper page with its machine-read text layer (daily-nebraskan-1911.pdf, Library of Congress, public domain), a whole 16-page newspaper issue of 1915 for a table of contents (victoria-daily-times-1915.pdf, Internet Archive via Wikimedia Commons, public domain), and an invented newspaper page with its text (bulletin-page.jpg, bulletin.txt).

Offline route, with no account or key:
1. Open Settings, choose the Offline tab, and under Transcription with Whisper download Whisper Base (about 60 MB; internet needed once). The download can be cancelled.
2. Choose New job, then Transcribe audio. Add interview.wav from the sample files. Under Provider, choose Whisper (on this computer), and under Model, Whisper Base. Choose Start.
3. The job runs on the CPU and shows its progress on the job's page. When it is done, choose Review: the transcript appears beside a player; click a line to hear it. Edit a word: the correction is kept apart from the original output. Press Ctrl F and type "Mopti" to find the word in the transcript.
4. Disconnect the network and repeat with another recording to confirm it runs offline. Results are written next to the sample file, as .txt and .docx with a provenance.json.
5. Offline text extraction: in Settings > Offline, under Text extraction with Tesseract, download French (4 MB, once). Choose New job, then Extract text, add bulletin-page.jpg, choose Tesseract (on this computer) and French, and Start. The review shows the page beside its text; the formats include ALTO XML and a searchable PDF.

Cloud features: text extraction (dodington-1755.jpg), correction of existing text (daily-nebraskan-1911.pdf), analyses (bulletin.txt) and cloud transcription need the user's own key, entered in Settings and kept in Windows Credential Manager. [Either: A test Gemini API key with a small spending cap is provided for certification: <key>. Paste it in Settings under Google Gemini, then run New job > Extract text on bulletin-page.jpg (Gemini refuses dodington-1755.jpg without a reason; Claude and Mistral read it), New job > Correct existing text on daily-nebraskan-1911.pdf, and New job > Analyse texts on bulletin.txt with People, places and organisations. Or: No test key is provided; these features need a key from the provider, as the listing says.] Before the first job sent to a provider, a dialog names it, says what is sent and links the privacy policy; Send records consent for that provider. An estimated cost is shown before each job starts. The cheaper mode sends a job to the provider's batch service, which may take up to a day to answer; it is optional and off by default.

YouTube: in Transcribe YouTube, the user pastes a link to a public video; Google's Gemini API reads the video from YouTube itself. Kalamu does not download, store or convert YouTube videos. The app asks YouTube's public oEmbed endpoint for the video title, and the review plays the video in YouTube's privacy-enhanced embedded player, loaded only when the user plays it.

Generative AI (Store Policy 11.16): every result is text generated by the AI model the user chooses. The listing says so, results are always shown beside their source for checking, and Report a problem (the flag in the header of every review, and the button in Settings > About) opens GitHub’s issue forms in the default browser, among them one to report inappropriate output to the developer. The app itself sends nothing.

Voluntary support link (Store Policy 10.8.2): Buy me a coffee, at the foot of the sidebar and in Settings › About, under Support development, opens https://buymeacoffee.com/fmadore in the default browser. Buy Me a Coffee, a third-party service, takes any payment on its own site. The app has no in-app purchase, collects no payment information, and a donation unlocks, removes or changes nothing. The Product declaration "This product allows users to make purchases, but does not use the Microsoft Store commerce system" is ticked for this link.

The full-trust desktop process reads the files the user selects, converts audio with bundled FFmpeg executables (LGPL; source published with each release, licence terms in the listing), runs Whisper and Tesseract on the CPU, stores API keys in Windows Credential Manager and writes results where the user chooses. It connects only to the provider the user selects, to Hugging Face for model downloads the user requests, to GitHub for the public model catalogue and Tesseract's language data the user requests and to YouTube as described above. There is no developer server, telemetry, driver, service or auto-start task.
```

## Hero image

[store/hero-1920x1080.png](store/hero-1920x1080.png), 1920 × 1080, for the listing's 16:9
hero art: newspapers, a letter and a cassette recorder whose lines settle into a clean page of
text. It is an illustration generated with OpenAI GPT Image (`gpt-image-2.5-flare`); its prompt
and settings are in the `.json` beside it. It shows no real document, script or person, and
carries no text, so the Store's title and buttons can sit over its right side.

## Screenshots

Taken for version 1.2.0: take them again from the installed 1.4.0 package, since they do not
show its wider views, larger type, projects in the sidebar, Jobs link or Settings tabs. From the installed Store package, in one theme throughout (the light one, as
the 1.0.0 captures). Store rules: `.png`, at least 1366 × 768,
under 50 MB; ten at most, five to eight recommended. The first one or two are what most people
see, in search results and at the top of the listing, so they carry what sets Kalamu apart:
checking AI text against its source. Keep what matters in the top two-thirds, where the Store
draws no overlay. No logo or marketing line in the image: each caption goes in the screenshot's
Description field (under 200 characters), which is also what a screen reader announces.

**How to take them.** Capture the window alone (Win+Shift+S, window mode) at 1920 × 1080 with
Windows display scaling at 100 %, or at 2560 × 1440 at 125 %, and keep one size for the whole
set. Wait for toasts to go. Name the jobs as a researcher would ("Le Réveil du Dahomey, 1937",
"Interviews, Ouagadougou 2024"), and keep the sidebar open on the same short list of jobs
throughout, so the set reads as one person's work. Rename the folders first: no `C:\Users\<name>`
path, no real key, no personal data in a file name.

**Every legible word is listing content.** The Store reads the text in a screenshot as the
listing's own words: 1.4.0 was refused under policy 11.5 (offensive content, "metadata") for two
screenshots of *The Monitor* (Omaha, 1915), whose masthead names the readers it served in the
wording of its time. Historical sources carry such wording, so before a document goes into a
screenshot, read what it shows (masthead, headlines, the text pane, analysis answers, names in
the dictionary) and leave out anything on race, religion, violence or politics that a reviewer
could take out of context, as well as caricatures in its advertisements. Prefer everyday
material: a student newspaper, local news, a scientific report.

**Material whose rights allow it.** Everything in `samples/`: the recording and the invented
page (made for Kalamu, CC0), the 1755 manuscript and the 1911 newspaper (public domain, sources
in its README); more public-domain newspapers from the Library of Congress's
[Chronicling America](https://chroniclingamerica.loc.gov/) (no known restrictions), or older
issues whose rights holder agrees; a handwritten letter in the public domain on Wikimedia
Commons; your own recording, or one made with the speaker's consent for this use. Gallica and
most archives restrict commercial reuse: a Store listing is safest without them.

**A demo project** for the shots that need one (2, 4 and 5), from public-domain material only:
a project *The Daily Nebraskan, Lincoln 1911* with collections *February–March*, *April–May*,
*Correspondence* and *Interviews*; the nine issues of `kalamu-screenshot-material.zip` (spring
1911, four pages each, from Chronicling America), `dodington-1755.jpg` and `interview.wav`; tags
such as *agriculture*, *athletics* and *theatre* on a few documents. Save the example *Letters: read, check, then people
and places* as a workflow *Newspapers*, its first step on the newspaper preset, and run it on the
issues from the project's selection, so the dictionary
has the same people under several forms of their names (*William Jennings Bryan*, *W. J.
Bryan*, *Mr. Bryan*, who spoke at convocation on 28 March; *Prof. E. H. Barbour*, *Professor
Barbour*) for a model to propose merging, and a pair to keep apart (*Grace Bryan*, his daughter,
whose wedding the 24 May issue announces). *Chancellor Avery* and *Dean Burnett* recur across
issues as well.

| # | What to show | Caption |
| --- | --- | --- |
| 1 | **Comparison in the review.** A newspaper column scanned on the left; on the right its text from set-up A, with two or three words highlighted where B reads differently, and one set of readings open over the text. The strip of pages below, a few marked reviewed. | Check every page beside its scan. Where two models read differently, the words are highlighted and you choose the right reading. |
| 2 | **A project.** *The Daily Nebraskan* project's documents: collections in the side list, tags on a few rows, the results column showing what each was read with and how much is reviewed, one row selected with the selection bar's actions. | Keep a project's documents together in collections and tags, each with every result read from it, whatever job made it. |
| 3 | **Handwriting.** `dodington-1755.jpg`, read with Claude or Mistral (Gemini refuses it), beside its transcription, lines kept as on the page, one word corrected (blue). | Read handwritten letters and registers line by line, kept as on the page so each line is easy to check against the original. |
| 4 | **A workflow run.** The *Newspapers* run with its first step done and stopped for review, the next step waiting, and *Continue* in view; or the workflow's drawing of its steps in Workflows. | Chain steps into a workflow: read the pages, check them, then analyse the corrected text. Nothing goes on until you say so. |
| 5 | **The dictionary.** The People field: rows with their spellings and the documents naming them, look-alikes pointed out, and a model's proposal open (rows, label, reason, certainty) with Accept and Turn down. | Gather the people and places your analyses found across a project, and merge their spellings under your own label, or let a model propose merges you decide on. |
| 6 | **Transcript.** The player above the transcript, speakers given real names, timestamps, the line being spoken highlighted. | Transcribe interviews with speakers and timestamps, give the speakers their names, and click any line to hear it. |
| 7 | **Analysis.** An article beside its answers: summary, people, places and tone with its reason, two set-ups' sigla on the names they disagree on, a name pointed at and found in the text. | Ask your own questions of each text: summaries, people, places and tone, every answer shown beside the document it comes from. |
| 8 | **New job, ready to start.** Extract text chosen, a folder of issues added, two set-ups (Gemini and Mistral OCR), the diagram of the task, and the estimate on the right with a cost per model. | Choose your files, a task and a model, and see the estimated cost before anything is sent to the provider. |
| 9 | **Table of contents.** A *Daily Nebraskan* issue's page on the left, the entries on the right with titles, pages, authors and summaries, the entries on the page shown marked; the strip of pages below. | Turn a whole newspaper issue or archival file into a detailed table of contents: each article with its pages, authors and summary. |
| 10 | **Offline.** Settings › Offline with a Whisper model and French for Tesseract downloaded, or New job with Tesseract (on this computer) chosen and the estimate at $0.00. | Transcribe recordings with Whisper and read printed pages with Tesseract, offline: free, no account or key, and nothing leaves your PC. |

Search, the glossary, Prompts, the job's page and the Jobs page are left out of the ten: the
description and features name them. The captures are not kept in the repository; name them
after the rows above (`01-comparison.png` … `10-offline.png`).

Leave out an image rather than show a feature that does not run as shown: certification checks
that the screenshots match the app.

# Mac App Store (App Store Connect)

Paste-ready text for the Mac App Store listing, first submitted with version 1.4.1. The steps,
the age rating and the App Privacy answers are in [mac-app-store.md](mac-app-store.md).
App Store Connect's limits, checked when this text was written: name and subtitle 30
characters, promotional text 170, description 4,000, keywords 100 (comma-separated, without
spaces after the commas, and no other company's trademark, Guideline 2.3.7), review notes
4,000. The text here fits: name 6, subtitle 25, promotional
text 167, description 3426, keywords 100,
review notes 3069.

Differences from the Microsoft Store text: macOS words (the Keychain, "your Mac"); no Buy Me a
Coffee line, since the store build hides that link (Guideline 3.1.1); the description in short
headed sections, since the App Store shows only its first lines before *more*; and the same
facts otherwise. **Languages**: English (U.S.) first; French can be added as a localisation
with its own screenshots.

## English (U.S.)

### Name

```text
Kalamu
```

If the name is taken on the store, `Kalamu for Researchers` (22 characters).

### Subtitle

```text
Read, transcribe, analyse
```

### Promotional text

Can be changed at any time without a new version.

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider you choose, or offline, then check every result beside its source.
```

### Description

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider of your choice, then check every result against its source. Kalamu works with Google Gemini, OpenAI, Anthropic Claude and Mistral under your own account and API key: their usage charges apply and are billed by them, and the app shows an estimated cost before anything is sent. Recordings can also be transcribed offline with Whisper, and clean print read offline with Tesseract, free, after a one-time download.

EXTRACT AND CORRECT TEXT
Read PDFs and images, printed or handwritten: newspapers, reports, letters, registers, manuscripts. Keep the lines as on the page, or join them into running text. Kalamu checks the files first and suggests fixes for double pages, low resolution and quiet recordings. Correct the noisy text digital libraries supply, with every change shown against the original.

TRANSCRIBE
Recordings and videos with speakers and timestamps; give the speakers their real names. Offline, Whisper transcribes on your Mac with no account or key, and nothing leaves the computer. With Gemini, public YouTube videos are transcribed from their link.

TABLES OF CONTENTS AND ANALYSES
Turn a newspaper issue, an archival file or a book into a detailed table of contents, one entry per article with its pages, then read each article on its own. Analyse texts with fields you define, as you would brief a research assistant: a summary, the people, places and organisations named, the tone with a reason, themes from your own list.

CHECK EVERY RESULT
The page beside its text, the recording beside its transcript, line by line. Corrections are kept apart from the AI output, which is never overwritten, and you mark what you have reviewed. Compare two models on the same pages, their differences highlighted, with error rates against your corrections. Search a document, or a whole project, corrections included.

PROJECTS AND WORKFLOWS
Keep documents in collections and tags, each with every result read from it. A project dictionary gathers the people, places and terms your analyses found; merge spellings under your own label. Save a job's settings as a pipeline, or chain steps into a workflow that stops for your review.

SAFE FOR LARGE JOBS
Work is saved piece by piece: an interrupted job resumes where it stopped, and nothing is paid twice. The cheaper mode sends large jobs to the provider's batch service at about half the price.

OPEN FORMATS
Text, Markdown, Word, JSON, CSV, subtitles, searchable PDF and ALTO XML, each with a provenance file recording the model, prompt and settings, for your methods section.

YOUR DATA
Before the first job sent to each provider, Kalamu says what will be sent and asks you. Your files go straight from your Mac to the provider you choose; offline, nothing leaves it. API keys are kept in the macOS Keychain. The developer receives no files, results, keys or telemetry, and the app sells no subscription or credits. Results are generated by AI: check a sample against the source before relying on them.

For macOS 13 or later on Apple Silicon. Interface in English and French; documents in any language the chosen model reads. Free and open source (MIT).

Google Gemini and YouTube are trademarks of Google LLC; OpenAI of OpenAI OpCo, LLC; Claude of Anthropic, PBC; Mistral AI of Mistral AI. Kalamu is independent and not affiliated with or endorsed by any of them.
```

### Keywords

```text
OCR,HTR,handwriting,transcription,speech to text,interview,archive,historian,digital humanities,ALTO
```

### What's New

Not asked for the first version on the store. From the next one, adapt the Microsoft Store's
*What's new* above.

### URLs and copyright

| Field | Value |
| --- | --- |
| Support URL | `https://github.com/fmadore/Kalamu/issues` |
| Marketing URL | `https://github.com/fmadore/Kalamu` |
| Privacy Policy URL | `https://fmadore.github.io/Kalamu/privacy` |
| Copyright | `2026 Frédérick Madore` |
| Primary category | Productivity |
| Secondary category | Education |

## App Review Information

**Sign-in required**: no. **Contact**: the maintainer's name, telephone and email (App Store
Connect asks; they are not public). Fill in the bracketed part of the notes; a test key may be
given here, never in the listing. Notes:

```text
Kalamu extracts text from documents, transcribes recordings and analyses texts with an AI provider the user sets up with their own API key (Google Gemini, OpenAI, Anthropic Claude or Mistral), or offline with Whisper (transcription) and Tesseract (printed text). No sign-in: the app has no account of its own and sells nothing; provider usage is billed to the user's own account.

Sample files: https://github.com/fmadore/Kalamu/releases/download/v1.4.1/kalamu-samples.zip (interview.wav, a recording; bulletin-page.jpg, an invented newspaper page; dodington-1755.jpg, a 1755 manuscript; daily-nebraskan-1911.pdf, a 1911 newspaper page; bulletin.txt).

Offline route, no key needed:
1. Settings > Offline > Transcription with Whisper: download Whisper Base (about 60 MB, once).
2. New job > Transcribe audio, add interview.wav, Provider: Whisper (on this Mac), Model: Whisper Base. Kalamu asks for permission to save results in the file's folder: choose that folder in the panel. Start.
3. When done, Review: the transcript beside a player; click a line to hear it, edit a word (the correction is kept apart), press Cmd F and type "Mopti".
4. Settings > Offline > Text extraction with Tesseract: download French (4 MB). New job > Extract text, bulletin-page.jpg, Tesseract, French, Start. The formats include ALTO XML and a searchable PDF.

Cloud features need the user's own key, entered in Settings and kept in the Keychain. [Either: A test Gemini key with a spending cap: <key>. Paste it in Settings under Google Gemini, then New job > Extract text on bulletin-page.jpg, or Analyse texts on bulletin.txt. Or: No test key is provided.] Before the first job sent to a provider, a dialog names the provider, says what is sent (the pages, recordings or text and any personal information in them), that it goes under the user's account and the provider's terms, and links the privacy policy; Send records consent for that provider, Cancel sends nothing (Guideline 5.1.2(i)).

Downloads (2.4.5(iv), 2.5.2): Whisper models (from Hugging Face) and Tesseract language data (from GitHub) are model weights and data files, not code; the features using them are in this build and are only offered for download in Settings, each verified by SHA-256. The model catalogue refreshed at start is a JSON list of models and prices from the app's GitHub repository, not code. There is no self-updater.

Sandbox: the app reads only files and folders the user picks or drops, and keeps security-scoped bookmarks so jobs resume after a restart. FFmpeg and ffprobe inside the bundle convert audio, signed to inherit the sandbox. Outgoing connections only: the chosen provider, Hugging Face and GitHub for downloads the user asks for, YouTube's oEmbed and privacy-enhanced player for a public video the user pastes.

Every result is text generated by the model the user chooses; it is always shown beside its source to be checked. Report a problem (the flag in each review and Settings > About) opens GitHub's issue forms in the browser. The developer receives no data, files, keys or telemetry.
```

## Screenshots

At least one, up to ten, 16:10, without transparency, `.png` or `.jpeg`, all one size: **2880 ×
1800** (a Retina Mac at its default scaling), or 2560 × 1600, 1440 × 900 or 1280 × 800. Take them
on a Mac, of the Mac App Store build, so the window's title bar, traffic lights and menu bar are
macOS's: screenshots of the Windows app would not show the app as it runs (Guideline 2.3.3).
Use the same ten scenes and captions as the Microsoft Store above, with "your Mac" for "your PC"
in the tenth. Capture the window alone with ⌘⇧4 then Space, which adds a shadow with
transparency: hold ⌥ while clicking to leave it out, or export the image without alpha.
