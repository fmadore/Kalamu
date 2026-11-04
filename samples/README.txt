Kalamu sample files

Seven files to try Kalamu with, in particular for the store review teams. Three are historical
documents in the public domain; four were made for Kalamu and may be used freely (CC0).

Works with no account and no key
--------------------------------
interview.wav         A 20-second spoken passage in English, read by Windows' own speech
                      synthesis. In Settings > Offline, download the Whisper Base model (about 60 MB,
                      once), then New job > Transcribe audio > Whisper (on this computer) > Start.
interview-two-speakers.wav
                      A 90-second interview in French between two people, invented and read
                      by two of Windows' French voices (Julie asks, Paul answers): a retired
                      typesetter remembers a weekly paper in Ouagadougou. Offline, Whisper
                      transcribes it without speakers; with Gemini or another cloud model
                      that labels speakers, give them names in the review. Choose French.
bulletin-page.jpg     Also offline: in Settings > Offline, under Text extraction with Tesseract,
                      download French (4 MB, once), then New job > Extract text > Tesseract
                      (on this computer) > French > Start. Formats include ALTO XML.

Need a Gemini, OpenAI, Claude or Mistral API key, saved in Settings
-------------------------------------------------------------------
dodington-1755.jpg    A handwritten French report on the rocks where the East India Company
                      ship Dodington was wrecked off South Africa on 17 July 1755, with a key
                      to the map it accompanied. New job > Extract text: the hand and the
                      spelling of the time are what handwritten text recognition is for.
                      Choose Claude or Mistral: Gemini refuses this page without giving a
                      reason (OTHER).
                      Bibliothèque nationale de France (btv1b53168804x), via Wikimedia
                      Commons. Public domain.

the-monitor-1915.pdf  The Monitor (Omaha, Nebraska), 31 July 1915, page 1: a weekly newspaper
                      of Omaha's African American community, with the machine-read text
                      layer libraries supply. New job > Correct existing text, to correct
                      that text; or Extract text, to read the columns again. Library of
                      Congress, Chronicling America. Public domain (published in the United
                      States in 1915).

victoria-daily-times-1915.pdf
                      The Victoria Daily Times (Victoria, British Columbia), 22 July 1915: a
                      whole 16-page issue, scans only. New job > Detailed table of contents:
                      one entry per article or item, with its pages, its authors and a
                      summary. Its machine-read text layer was removed, so the pages are
                      read from the scans. Internet Archive (victoriadailytimes19150722), via
                      Wikimedia Commons. Public domain.

bulletin-page.jpg     An invented newspaper page in French, made for Kalamu. New job >
                      Extract text.
bulletin.txt          The same page as text. New job > Analyse texts (for example People,
                      places and organisations).

Kalamu: https://github.com/fmadore/Kalamu
