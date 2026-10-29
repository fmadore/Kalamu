# Partner Center listing text

Paste-ready text for Kalamu's Microsoft Store listing, written for the **first
submission**, version 1.0.0, not yet built. The submission steps are in
[microsoft-store.md](microsoft-store.md). Check every block against the app as packaged before
pasting: a listing that promises what a reviewer cannot find fails certification.

Every paragraph inside a block is one unwrapped line, because Partner Center keeps line breaks;
blank lines separate paragraphs. Keep it that way when editing, and do not let a formatter
re-wrap the blocks.

Microsoft's [listing field documentation](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)
allows 10,000 characters for Description, 1,500 for What's new, and 20 features of 200
characters each. Short description accepts 1,000 characters, but only the first 270 show in
some views; the one below fits. Keywords allow up to 7 entries of at most 40 characters, with
no more than 21 words across all of them.

**Languages.** English (United States) only, since the interface is in English. Add French and
German listings when the interface is translated, with screenshots in those languages.

**Before pasting**, replace the provider model names if the catalogue has changed, and check
that the features listed still match the app.

## English (United States)

### Description — full text

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with the AI provider of your choice, then check every result against its source. Kalamu works with Google Gemini, OpenAI, Anthropic Claude and Mistral under your own account and API key, so their usage charges apply and are billed by them; the app shows an estimated cost before anything is sent. Recordings can also be transcribed offline with Whisper, free, after a one-time model download.

Extract text from PDFs and images, printed or handwritten: newspapers, reports, letters, registers and manuscripts. Keep the lines as on the page, to check against the scan, or join them into running text for reading and searching. Before a job starts, Kalamu checks the files and suggests fixes: double pages to split, scans whose resolution is too low, recordings that are too quiet.

Transcribe recordings and videos with speakers and timestamps, and give the speakers their real names. Long recordings are cut at pauses and sent piece by piece. Offline, Whisper transcribes on your PC with no account, key or internet connection, and nothing leaves the computer; speaker labels need a cloud model. With Gemini, public YouTube videos are transcribed from their link, without downloading them.

Make a detailed table of contents of a long document: a newspaper or magazine issue, an archival file scanned in one piece, a book or report. One model reads each page, another, cheaper one combines the pages into one entry per article or item, with its pages and the fields you choose. Correct the noisy text that digital libraries supply, page by page, with every change shown against the original.

Analyse texts with fields you define, as you would brief a research assistant: a summary, the people, places and organisations a document names, its tone with a reason, themes from your own list. Ready-made analyses cover common needs, including the question, argument, methods and findings of a scholarly publication for a literature review. Answers come back in the shape you asked for and can be exported as a spreadsheet with a column per field.

Check every result beside its source: the page next to its text, or the recording next to its transcript, line by line. Corrections are kept apart from the AI output, which is never overwritten, and you mark what you have reviewed. Compare two or more models on the same documents: their differences are highlighted in the text, with error rates against the pages you have corrected.

Large jobs are safe and affordable. Work is saved piece by piece: a job interrupted by closing the app, a crash or a lost connection resumes where it stopped, and nothing is sent or paid for twice. Long jobs keep running from the notification area while the window is closed. For large collections, choose the cheaper mode: the provider's batch service answers within a day at about half the price, and Kalamu collects the results when it next opens.

Export to text, Markdown, Word, JSON, CSV, subtitles (SRT and WebVTT) and, with Mistral OCR, searchable PDF. Each output comes with a provenance.json file recording the model, prompt and settings, so your methods can be described and repeated. Save a job's settings as a pipeline to start the next batch in one click, and export it with its full prompts to cite in a methods section or share with colleagues.

Your files go straight from your PC to the provider you choose; offline, nothing leaves the PC. API keys are kept in Windows Credential Manager. The developer receives no files, results, keys or telemetry, and the app sells no subscription, credits or API access. AI output can be fluent and wrong: check a sample against the source before relying on it. An optional Buy Me a Coffee link supports development and unlocks nothing.

Requires Windows 11 with the Microsoft Edge WebView2 Runtime, with native x64 and ARM64 packages. The interface is in English; documents and recordings can be in any language the chosen model reads. Free and open source under the MIT licence.

Google Gemini and YouTube are trademarks of Google LLC; OpenAI is a trademark of OpenAI OpCo, LLC; Claude is a trademark of Anthropic, PBC; Mistral AI is a trademark of Mistral AI; Microsoft Word is a trademark of the Microsoft group of companies. Kalamu is an independent project, not affiliated with or endorsed by any of them.
```

### What's new in this version — 1.0.0

```text
First release in the Microsoft Store.
```

### Features — one feature per line

```text
Extract text from scanned and handwritten PDFs and images with Gemini, OpenAI, Claude or Mistral
Transcribe recordings and videos with timestamps, in the cloud with speakers or offline
Offline transcription with Whisper: download a model once, then no account, key or internet
Transcribe public YouTube videos from their link with Gemini, without downloading them
Analyse texts with fields you define: summaries, people, places, organisations, tone, themes
Make detailed tables of contents of newspaper issues, archival files and books
Correct existing OCR text page by page, every change shown against the original
Ready-made analyses, including the question, argument, methods and findings of a publication
Check every result beside its source, page by page or line by line in the recording
Corrections kept apart from the AI output, which is never overwritten
Compare models on the same documents, with differences highlighted and error rates
Estimated cost shown before anything is sent, from page counts and published prices
Cheaper mode: send a large job to the provider's batch service at about half the price
Interrupted jobs resume where they stopped; nothing is sent or paid for twice
Long jobs keep running from the notification area while the window is closed
Save a job's settings as a pipeline; export it with its prompts for a methods section
Export to text, Markdown, Word, JSON, CSV, SRT, WebVTT, and searchable PDF with Mistral OCR
A provenance file beside every output, recording the model, prompt and settings
File checks for double pages, low resolution and quiet recordings, with suggested fixes
API keys kept in Windows Credential Manager; files go straight to the provider you choose
Light and dark themes; native Windows x64 and ARM64; free and open source
```

### Short description

```text
Read scanned and handwritten documents, transcribe interviews and analyse texts with Gemini, OpenAI, Claude or Mistral under your own key, or transcribe offline with Whisper. Check every result beside its source; corrections never overwrite the AI's text.
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
```

Recommended memory: 8 GB; Whisper's larger models need more, and their speed depends on the
processor.

## Notes for certification

Fill in the bracketed parts at submission. These notes are seen only by the certification
team; a test key may be given here, never in the listing.

```text
Product: Kalamu. Version: 1.0.0.0, native x64 and ARM64. Windows 11 with Edge WebView2 Runtime. Kalamu extracts text from documents, transcribes recordings and analyses texts with an AI provider the user configures with their own API key (Google Gemini, OpenAI, Anthropic Claude or Mistral), or transcribes offline with Whisper. The app supplies no cloud account or credits; provider usage is billed to the user's own account, as the listing states.

Sample files for testing: [link to kalamu-samples.zip on the GitHub release page], containing a short public-domain recording (interview.mp3), a scanned page (letter.pdf) and a text (article.txt).

Offline route, with no account or key:
1. Open Settings, find Offline transcription and download Whisper Base (about 60 MB; internet needed once). The download can be cancelled.
2. Choose New job, then Transcribe audio. Add interview.mp3 from the sample files. Under Provider, choose Whisper (on this PC), and under Model, Whisper Base. Choose Start.
3. The job runs on the CPU and shows its progress in Jobs. When it is done, choose Review: the transcript appears beside a player; click a line to hear it. Edit a word: the correction is kept apart from the original output.
4. Disconnect the network and repeat with another recording to confirm it runs offline. Results are written next to the sample file, as .txt and .docx with a provenance.json.

Cloud features: text extraction (letter.pdf), analyses (article.txt) and cloud transcription need the user's own key, entered in Settings and kept in Windows Credential Manager. [Either: A test Gemini API key with a small spending cap is provided for certification: <key>. Paste it in Settings under Google Gemini, then run New job > Extract text on letter.pdf, and New job > Analyse texts on article.txt with the Summary and keywords for research analysis. Or: No test key is provided; these features need a key from the provider, as the listing says.] An estimated cost is shown before each job starts. The cheaper mode sends a job to the provider's batch service, which may take up to a day to answer; it is optional and off by default.

YouTube: in Transcribe YouTube, the user pastes a link to a public video; Google's Gemini API reads the video from YouTube itself. Kalamu does not download, store or convert YouTube videos. The app asks YouTube's public oEmbed endpoint for the video title, and the review plays the video in YouTube's privacy-enhanced embedded player, loaded only when the user plays it.

Generative AI (Store Policy 11.16): every result is text generated by the AI model the user chooses. The listing says so, results are always shown beside their source for checking, and Report a problem in the review and in Settings opens a GitHub issue form in the default browser to report inappropriate output to the developer.

Voluntary support link (Store Policy 10.8.2): Buy me a coffee, at the foot of the sidebar and in the Support development section at the end of Settings, opens https://buymeacoffee.com/fmadore in the default browser. Buy Me a Coffee, a third-party service, takes any payment on its own site. The app has no in-app purchase, collects no payment information, and a donation unlocks, removes or changes nothing. The Product declaration "This product allows users to make purchases, but does not use the Microsoft Store commerce system" is ticked for this link.

The full-trust desktop process reads the files the user selects, converts audio with bundled FFmpeg executables (LGPL; source published with each release, licence terms in the listing), runs Whisper on the CPU, stores API keys in Windows Credential Manager and writes results where the user chooses. It connects only to the provider the user selects, to Hugging Face for model downloads the user requests, to GitHub for the public model catalogue and to YouTube as described above. There is no developer server, telemetry, driver, service or auto-start task.
```

## Hero image

[store/hero-1920x1080.png](store/hero-1920x1080.png), 1920 × 1080, for the listing's 16:9
hero art: newspapers, a letter and a cassette recorder whose lines settle into a clean page of
text. It is an illustration generated with OpenAI GPT Image (`gpt-image-2.5-flare`); its prompt
and settings are in the `.json` beside it. It shows no real document, script or person, and
carries no text, so the Store's title and buttons can sit over its right side.

## Screenshots

From the packaged app, in the dark theme, at the display scaling it is used at. Store rules:
`.png`, at least 1366 × 768, under 50 MB; ten at most, five to eight recommended. Keep what
matters in the top two-thirds, where the Store draws no overlay. No logo or marketing line in
the image: each caption below goes in the screenshot's Description field (under 200 characters),
which is also what a screen reader announces. Use sample material whose rights allow it, and
no real API key or personal data.

| Image | Caption |
| --- | --- |
| 1 — New job | Choose files, a model and a prompt, and see the estimated cost before anything is sent. |
| 2 — Review of a page | Check the text beside the scanned page; corrections are kept apart from the AI output. |
| 3 — Comparison | Compare models on the same pages, with their differences highlighted in the text. |
| 4 — Transcript | Review a transcript beside the recording, with speakers named and every line playable. |
| 5 — Analysis | Answers to your own fields, collated across documents and checked against the text. |
| 6 — Jobs | Long jobs run in the background, resume after interruptions and never pay twice. |
| 7 — Offline | Transcribe offline with Whisper: nothing leaves your PC. |
