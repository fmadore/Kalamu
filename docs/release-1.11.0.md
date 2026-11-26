Kalamu 1.11.0 can use the models of a server of your own: Ollama on your computer or in your
office, or a university's service that speaks as OpenAI does, each model tried before it is
offered. It repairs the six bugs a review of the repository found on 25 November 2026, the day
after 1.10.1, keeps safety copies of your jobs so that one damaged file cannot cost you your
corrections, and lets you find a model in the list by typing its name. For what 1.10.1 changed,
see the [1.10.1 release](https://github.com/fmadore/Kalamu/releases/tag/v1.10.1).

## What is new since 1.10.1

- **A server of your own**
  ([#298](https://github.com/fmadore/Kalamu/pull/298),
  [#283](https://github.com/fmadore/Kalamu/issues/283),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-26-a-server-of-the-researchers-own.md)).
  **Settings › Other servers** adds a server in three steps: a **Name** and an **Address** (a
  key only if the server asks for one), **Connect**, then tick the **Models on this server** and
  press **Try and add**. Kalamu tries each model with three small requests of its own, and offers
  it in New job only for what it showed it can do: reading pages if it read the trial's picture,
  analyses, corrections and translations if it filled the trial's form. A server on this
  computer sends nothing out and asks nothing; a server elsewhere on a network is asked about
  once, by its name and host, and again if its address changes, before a job is resumed or
  retried included. Transcription, economy mode and the search by meaning do not use such a
  server yet. The [guide](https://fmadore.github.io/Kalamu/guide/other-servers) explains each
  step, and the [privacy policy](https://fmadore.github.io/Kalamu/privacy) what goes to the
  server.
- **Safety copies of your jobs**
  ([#294](https://github.com/fmadore/Kalamu/pull/294),
  [#275](https://github.com/fmadore/Kalamu/issues/275),
  [decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-25-job-database-protected.md)).
  Kalamu copies its job database, which holds your results and corrections, once a day and
  before an update changes its layout, and keeps the last three daily copies in `backups` in its
  data folder (none is made
  when it would leave less than 1 GB free, which Settings then says). The whole file is checked
  after launch. A file that cannot be read is set aside, never deleted, and the newest copy is
  offered in its place, at launch and from **Settings › General › Safety copies**.
- **Find a model by typing**
  ([#297](https://github.com/fmadore/Kalamu/pull/297)). The model list opens with a search
  field: typing narrows it by a model's name, its id or its provider's name, whatever the case
  and accents, and the arrow keys and Enter choose among what is left. Each model now shows a
  short summary whole, in English and French, and the list no longer grows under the pointer;
  the chosen model's full description is still shown under the field.

## What is fixed since 1.10.1

All six were found by the review of 25 November
([#293](https://github.com/fmadore/Kalamu/pull/293),
[decision](https://github.com/fmadore/Kalamu/blob/main/docs/decisions/2026-11-25-fixes-of-the-review.md)).

- **Pages sent at the same time across a whole job**
  ([#269](https://github.com/fmadore/Kalamu/issues/269)). *Pages sent at the same time* held
  within one file only, so a folder of photographs was read one call at a time. The limit now
  holds for the job as a whole, and a folder is read several photographs at once.
- **A per-minute limit no longer stops the whole job**
  ([#270](https://github.com/fmadore/Kalamu/issues/270)). When a provider refuses a call for a
  while and says how long to wait, up to five minutes, Kalamu waits and goes on, even when the
  refusal speaks of a quota. An account that has run out still stops the job.
- **Closing the window asks first**
  ([#271](https://github.com/fmadore/Kalamu/issues/271)). Closing the window or quitting (⌘Q on
  a Mac) with a correction, a prompt or a workflow not yet saved asks the question Kalamu
  already asks when leaving a screen.
- **Arabic and other right-to-left text reads in its own direction**
  ([#272](https://github.com/fmadore/Kalamu/issues/272)) in the review, its editor, transcripts,
  analyses, search and dialogs.
- **The search by meaning keeps what you paid for**
  ([#273](https://github.com/fmadore/Kalamu/issues/273)). Its index was deleted on any failure
  to open it, which threw away paid embeddings. It is now replaced only when it truly cannot be
  read, and set aside rather than deleted.
- **A deleted project takes its copied-in originals with it**
  ([#274](https://github.com/fmadore/Kalamu/issues/274)). The scans and recordings a project
  file brought in are removed when the project is deleted, which the question before deleting
  says, and an older project file is brought up to date in Kalamu's data folder rather than
  copied whole into the system's temporary folder.

Also: when something goes wrong in a screen, the screen says so and the sidebar keeps working;
an error nobody expected is said once and written to the log without the text it quotes; and
the start-up failure has **Try again**
([#280](https://github.com/fmadore/Kalamu/issues/280)). The window no longer stretches past its
ends when a touchpad keeps scrolling ([#296](https://github.com/fmadore/Kalamu/pull/296)). A
downloaded model list is checked before it is used, and one that fails is set aside and said
once in Settings ([#279](https://github.com/fmadore/Kalamu/issues/279)). A recording whose
length cannot be read is no longer estimated at nothing: New job says *length unknown* and its
cost shows in Jobs as it runs, and a WAV file's length is read even without FFmpeg
([#300](https://github.com/fmadore/Kalamu/pull/300)).

## Behind the scenes

- **The first release under the release gate**
  ([#276](https://github.com/fmadore/Kalamu/issues/276)). 1.9.0 and 1.10.1 were published
  before their test pass. A final version is now built only from a release candidate's commit,
  and published only once the test pass is signed off.
- **The app driven in CI** ([#277](https://github.com/fmadore/Kalamu/issues/277), in part). A
  test build of Kalamu is started and driven on Windows x64 and ARM64 against a stand-in for the
  providers (a job to its exported files, a correction found after a restart, a project saved
  and opened, a paused job resumed, a batch collected after a restart), and started and quit on
  macOS, on every release candidate.
- **A weekly live check** ([#278](https://github.com/fmadore/Kalamu/issues/278)) runs one small
  job per task against each provider's real service, under a ceiling of a few cents, and keeps
  the providers' error answers as test fixtures.

## Download

- **Microsoft Store**: the Store copy installs without a warning and updates itself:
  [Kalamu in the Microsoft Store](https://apps.microsoft.com/detail/9MXT5555BZ1H).
- **Mac App Store**: once Apple has reviewed it.
- `Kalamu_1.11.0_x64-setup.exe`: Windows 11 on Intel and AMD PCs.
- `Kalamu_1.11.0_arm64-setup.exe`: Windows 11 on ARM64 (Snapdragon laptops).
- `Kalamu_1.11.0_aarch64.dmg`: macOS 14 or later on a Mac with Apple Silicon (M1 and later).
  Open it and drag Kalamu to Applications.
- `kalamu-samples.zip`: a short recording, scanned pages, a newspaper issue and a text to try
  the app with.
- `Kalamu_1.11.0.msixbundle` and the two `.msix` packages are the Store submission and its
  per-architecture packages, for testing; install Kalamu from the Store or the installers.

Every download bundles FFmpeg, llama.cpp, offline Whisper and offline Tesseract, and the Windows
downloads Whisper's Vulkan helper; their models and language data are downloaded from Settings
when wanted. Installing 1.11.0 over an earlier version keeps your jobs, settings, presets and
keys; the first safety copy is made on its first launch, once the database has been checked. The job database's layout is
unchanged: 1.10.0, 1.10.1 and 1.11.0 read one another's; a version before 1.10.0 refuses it. A
job run on a server of your own shows its model as unknown in an earlier version.

## Known limitations

Those of [1.10.1](https://github.com/fmadore/Kalamu/blob/main/docs/release-1.10.1.md#known-limitations)
still hold. What this version changes among them, and adds:

- **A server of your own has been tested against answers written from Ollama's and the other
  servers' documentation**, and on Ollama by hand during this version's test pass; no
  OpenAI-style server, a university's included, has yet been reached. The files Kalamu writes
  name the server as it stands when they are written, not as each page was read.
- **A per-minute limit is recognised from the providers' documented wording**, not yet from
  answers captured from each of them.
- **Two identical files in one job may each be paid for**: with pages sent across the whole
  job, the second can be sent before the first comes back from the cache.
- **Right-to-left text in a block of several paragraphs** (a compared text, the correction
  editor) may start on the left in WebKit on a Mac, its lines in the right order.
- **The safety copies' question before the window opens, and the restart into a copy,** have
  run in the tests only.

## Included software

Unchanged since 1.10.0: see
[its notes](https://github.com/fmadore/Kalamu/blob/main/docs/release-1.10.0.md#included-software),
and `licenses/THIRD-PARTY-NOTICES.txt` in every download for each component and its licence.

See the [roadmap](https://github.com/fmadore/Kalamu/blob/main/docs/roadmap.md) for what comes
next.
