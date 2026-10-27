# 2026-10-27 · YouTube videos transcribed from their link

## Context

Researchers asked to transcribe YouTube videos (sermons, conferences, interviews) without
downloading them first (issue #2). The Gemini API reads public YouTube videos from their address:
the request carries the link, Google fetches the video, and nothing is downloaded or stored on
the PC, which also keeps within YouTube's terms. OpenAI, Claude and Mistral have nothing
similar. This entry draws on Gemini's video-understanding documentation as of 27 October and on
a forum report from late August about clipping YouTube links.

## Decisions

- **A video is a source with no file.** `InputKind::YouTube` and `SourceFile::youtube` (in
  `core`) turn a watch, `youtu.be`, Shorts, live or embed link into a source whose path is the
  canonical watch address and whose fingerprint is the SHA-256 of the video id. The same video
  pasted under another form of link therefore finds the same cached results. The engine's
  `inspect_links` checks pasted links; it does not fetch the video, so whether it is public and
  how long it is are known only when Gemini reads it.
- **The link travels as a prepared input** with the pseudo-MIME type `video/x-youtube`
  (`YOUTUBE_LINK`), its bytes being the address, as offline engines already receive raw PCM
  under a type of their own. A model reads YouTube links when its catalogue entry lists that
  type, so the per-model input filter of the same day decides which models are offered and sent
  a video, without the engine knowing any provider. Gemini 3.8 Flash, Gemini 4 Argon and
  Gemini 3.1 Pro list it.
- **Gemini sends `fileData.fileUri` at low media resolution.** The words are in the sound track;
  low resolution reads about 100 tokens a second of video instead of 300.
- **A video is one piece.** Gemini's `videoMetadata` clipping would cut a long video into
  twenty-minute pieces like a recording, keeping resume and retries per piece. But since about
  19 August clipping a YouTube link reportedly clips only the frames: the whole sound track is
  still sent and billed, so each piece would hear the whole video. Until that changes, a video
  is sent whole, which suits videos up to about two hours: the 65,535-token answer limit runs
  out before three hours of speech. A transcript cut off at that limit fails with a message
  saying the video is too long, rather than ending silently part-way.
- **Refusals in plain words.** A private or unlisted video comes back as "permission denied";
  read as a rejected key, it would stop the whole job. Gemini's errors for a YouTube link are
  reworded (private or unlisted, removed, too long, free tier's daily 8 hours used up), and a
  403 that does not mention the key is the video's problem, not the account's.
- **Provenance records the address and when it was read** (`source.url`, `source.read_at`),
  since there is no file to keep and the video may change or be removed.
- **Videos are named after their title.** When a link is added, the app asks YouTube's public
  oEmbed endpoint (no key) for the title, in `src-tauri`, the one place that already holds an
  HTTP client outside the providers. The title names the row, the job and the files
  (`<title> [<id>]`, the id keeping two videos of the same title apart; `youtube-<id>` when the
  title cannot be read, offline for instance) and is recorded in provenance. oEmbed also refuses
  private (401) and removed (404) videos, so those are caught before the job; unlisted videos
  answer like public ones, so a recognised link is shown in grey, not as checked.
- **Results go to a chosen folder**; a job of videos written "next to the sources" is refused
  (`job.link_needs_folder`). The folder last chosen is remembered in the settings and offered,
  and while none is chosen the summary says so above Start.
- **The review embeds the video, on the stage, behind a poster.** The video sits on the dark
  stage, as a scan does. Until the researcher plays, a poster with the video's title and the
  app's own play button stands in, so opening a review contacts no one; the first play, or a
  click on a line, loads YouTube's privacy-enhanced player (`youtube-nocookie.com`, the one
  domain the content security policy lets into a frame) at that moment, in the interface's
  language. No YouTube script runs in the app: the review drives the player with its message
  API (`enablejsapi`), seeking on a click, highlighting the line being spoken and keeping it in
  view, as the audio player now does too. A player error (101 or 150: the owner forbids
  embedding) or a player that never answers (offline) turns the stage into a card that opens
  the video on YouTube, and lines then open it there at their moment. The pane is sized from
  its own height, so in a half-screen window the stage sits beside the lines. Lines are read from the `[hh:mm:ss]` markers Gemini writes into its text, now also
  for recordings transcribed by Gemini, whose review showed no timed lines before.
- **No cost before the run.** The length is unknown until Gemini reads the video, so New job
  says so instead of estimating. Costs are recorded from the tokens Gemini reports at the
  model's price; while YouTube links are in preview Google may charge less.
- **No "preview" label in the interface**: the tile reads like the other tasks, and the tips
  say that Google may change the limits of this preview feature.

## Consequences

- The four tiles of New job sit two by two: written sources (extract, analyse), then spoken
  ones (a recording, a YouTube video). The YouTube tile is a transcription with links in place
  of files and only the models that read them.
- A pipeline saved for a transcription applies to videos too, keeping the videos.
- Tested with a mock server (request shape, error wording, truncation) and an engine test (one
  call for the same video under two links, provenance, timed lines in the review). Not yet run
  against the live API: the Gemini account has no credit.
- The embedded player was checked in the browser preview, not yet in the packaged app: YouTube
  refuses embeds whose page sends no referrer (error 153), so the `tauri.localhost` origin has
  to be checked in a build. The frame sends `strict-origin-when-cross-origin`.
- A design critique (Impeccable, 27 October) shaped the titles, the folder, the poster and
  stage, the blocked state and the half-screen layout.
- Left for later: pieces for long videos once clipping covers the sound track, and a cost
  estimate from `countTokens`.
