# 2026-11-22 · An answer left empty is asked again with more thinking

## Context

1.10.0-rc2 refuses an analysis's answer that leaves every field empty for a text of more than
500 words (`structured::refuse_empty`), as a broken answer: billed, charged to the piece and
asked again, up to the engine's four attempts. The Windows test pass of rc2 found GPT-6.2 Sol,
at its suggested *Low* thinking, answering the 10 May issue (5,557 words) with *People, places
and organisations* empty all four times (about $0.14), and again on *Retry failed pages*; the
same issue alone at *Medium* answered fully. The same request four times fails four times, and
the failure's "Try again" led to the same failure.

## Decision

- **An answer refused as wholly empty is asked again one thinking level up**: the next level the
  model's catalogue entry lists above the one sent (`Reasoning::above`), to `high` at most
  (`MOST_THOUGHT_ASKED_AGAIN`): *Low*, then *Medium*, then *High*, and *High* again for a fourth
  attempt. Levels past `high` cost far more for what they add and stay the researcher's choice.
  A model that does not reason, one the catalogue lacks, or one already at the bound is asked as
  before. Every other refusal (broken fields, a cited answer whose quotations are not found) and
  every other failure is retried as before, at the same level.
- **The answer kept records the level it was sent at**: the piece's request settings, which
  provenance shows, are taken from the attempt that answered (`reasoning.effort: medium`). The
  set-up, its cache key and the job keep the level chosen, so the answer is found again under
  the same key and a later run of the same set-up starts from the same level.
- **The failure says what to do next.** Not escalated: "…it may not have read it. Choose a higher
  thinking level, or another model". Escalated and still empty: "…even when asked again with
  more thinking (medium, then high): it may not have read it. Choose another model"
  (`structured::empty_reason`). The interface's sentence for an unusable answer
  (`provider.invalid_answer`, en and fr) now points to *Thinking* or another model instead of
  "Try again later".

## Consequences

- An escalated attempt costs what its level costs; the estimate still prices the level chosen,
  so a piece answered at *Medium* after an empty *Low* costs more than estimated.
- Economy mode is unchanged: a batch's answer left empty is refused as before
  (`batch::collect`), with the new sentence, and is not sent again at another level.
- Tests: `engine` (`an_answer_left_empty_is_asked_again_with_more_thinking_and_kept_with_its_level`,
  `an_answer_left_empty_at_every_level_fails_saying_which_were_tried`) and `catalog`
  (`Reasoning::above`).
