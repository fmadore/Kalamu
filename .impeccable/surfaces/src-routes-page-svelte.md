---
version: 2
slug: "src-routes-page-svelte"
primary_target: "src/routes/+page.svelte"
related_targets: ["src/app.css","src/lib/components"]
---

# Kalamu app shell and screens

Scope: the whole desktop app (shell, New job, Job detail, Review, Prompts, Settings). Visitor
mode: Operate. Audience: non-technical researchers checking AI transcriptions against their
sources on Windows laptops.

## Direction contract

THESIS: Grey surfaces do the work, so the three marks that matter read at a glance: black for
the one action that moves the work forward, highlighter yellow for "look here" (the models
disagree), blue for the researcher's own hand (corrections, kept apart from AI output that is
never overwritten). The source is the authority and gets the best seat.

OWN-WORLD: Chrome grey #F2F3F5 sidebar, white work surface, hairlines #E3E4E8. Graphite ink
#111214 is the only button fill (#ECEDEF in dark), one primary per work area with its
shortcut. Highlighter #FFE066 only for disagreements and found terms, ink text on it. Hand blue
#2456D6 only for corrections, as an underline. Scans on a dark lightbox #1E1F22 (#0F1012 in
dark) in both themes. Instrument Sans for the interface, Source Serif 4 only for the text being
checked, IBM Plex Mono for anything a machine produced. Lucide icons at an absolute 1.5px
stroke.

STORY: The researcher opens a page, sees the scan lit on the dark stage beside its text, finds
each disagreement already highlighted, opens it to see who read what, takes a reading or types
their own as a blue-underlined correction, and marks the page reviewed with Ctrl Enter while the
page strip fills in green.

FIRST VIEWPORT: Grey sidebar (mark and name, black New job with Ctrl N, jobs grouped Running /
Unfinished / Finished with mono counts, Prompts and Settings at the foot); a white masthead
with the job name, a slash and the file name in mono, a pager and the black Mark reviewed
button; the scan on the dark stage with a floating zoom pill; the reading text in Source Serif
with highlighted disagreements and faint mono line numbers; a page strip along the bottom with
a review tally.

FORM: Working tool: shell plus masthead plus one work area; review as a two-pane lightbox and
marked text; New job as numbered steps beside a sticky summary. Signature interaction: a
highlighted disagreement is a button that opens an inline readings card under its line, where
any reading can be taken as a correction.

FINISH: unreviewed and undocumented is unfinished; DESIGN.md and .impeccable/design.json record
the shipped system ("Graphite & highlighter"), with open questions left for the user.
