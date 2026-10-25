---
version: 1
slug: "src-routes-page-svelte"
primary_target: "src/routes/+page.svelte"
related_targets: ["src/app.css","src/lib/components"]
---

# Scriptorium app shell and screens

Scope: the whole desktop app (shell, New job, Jobs, Review, Prompts, Settings). Visitor mode:
Operate. Audience: non-technical researchers checking AI transcriptions against their sources.

## Direction contract

THESIS: The AI text is one witness; everything else (the other models' readings, the
researcher's corrections) is set as an apparatus beneath it, so disagreement and correction
are visible at a glance. Refuses the AI-product arrangement of glowing cards and chat
styling, and the parchment costume the name invites.

OWN-WORLD: White page ground; bookcloth blue #1D2B4F owns the shell (rail and header); text
black #111111; rubric red #B3261E only for sigla, marginal line numbers and corrections; pale
blue #DCE3F2 for selection and highlighted lemmata. EB Garamond for documents, titles,
running heads in small caps and oldstyle figures; Source Sans 3 for every control. Hairline
rules divide; no shadowed cards.

STORY: The researcher sees the source beside the reading text, finds every disagreement
listed under it by line, corrects in place, marks the page reviewed, and trusts that nothing
the model wrote or they changed is lost.

FIRST VIEWPORT: Blue rail (wordmark in Garamond, four nav items, background job at its
foot); blue header with back link, Garamond document title, pager; a scores strip; the source
pane on a grey ground at left; the edition pane at right with a small-caps running head,
numbered lines (every fifth and every line with a variant, in red), the apparatus below a
black hairline, and a sigla key carrying each set-up's controls.

FORM: Critical edition with apparatus criticus, first on the ordered list (IMPECCABLE'S
PICK, chosen by the user), seed:e298d742af434f7d. Signature interaction: hovering or
focusing an apparatus entry highlights its lemma in the text and its line number, and the
reverse.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
