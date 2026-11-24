---
layout: guide
title: When something goes wrong
nav: Troubleshooting
order: 9
lede: The common problems, and what to do about each.
description: Troubleshooting Kalamu, with answers to common questions.
---

[Français](fr/help.md)

## A key does not work

In **Settings › API keys**, press **Test**. If the provider refuses the key: check that you
copied all of it, that the key has not been deleted on the provider's site, and that the
account has billing set up (most providers refuse API calls without it, even for cheap
models).

## A job stopped or some pages failed

Open the job: each failed page explains why in plain language. The usual causes are a dropped
connection, a busy provider (Kalamu waits and retries automatically), or a page the provider
refused. Run the job again and only the failed pages are retried.

An analysis that returns an empty answer for a long text is not saved: Kalamu retries with
higher reasoning effort where supported (*Medium*, then *High*) and keeps the first complete
answer, recording the level used. If every attempt still comes back empty, the page lists the
levels tried: choose another model rather than running it again.

A job that was paused, or cut short because the app was closed, is at the foot of the sidebar
under *To resume*.

## The text is cut off, or lines are missing

- A page marked as cut off reached the model's output limit: read fewer pages per call.
- Lines missing in the middle of a page are a model's mistake. Read the page with a second
  model and [compare](review.md#compare-models), or use a stronger model for those pages.
- For a spread photographed as one image, accept the fix New job proposes, so each page is
  read on its own.

## An offline model is very slow

That is expected on a computer without a usable graphics processor, above all for the vision
and language models. Use a smaller Whisper size, Tesseract for clean print, or a cloud model.
See [Working offline](offline.md#how-fast).

## The cost is higher than I expected

Look at the job's actual usage, and at the model's price in the list. The estimate counts what
a model writes and the thinking it is billed for before writing, at the level chosen under
*Thinking* (some models, such as Gemini 4, think far more than others), and prices as pages
full of small print those read with the *Newspaper or magazine* prompt and those of a PDF
whose text layer is small print. A region read again is priced as the part of the page sent.
Long handwritten pages, a newspaper scanned as images and read with another prompt, and
analyses with long answers can still cost more than the estimate's averages. For the next batch:
a smaller model, or the [cheaper mode](costs.md#three-ways-to-pay-less).

## Where are my results?

On your computer, in Kalamu's data folder, until you delete the job. Deleting a job deletes
its results; your original files are never touched. To keep or share results, export them.

## Questions

**Is Kalamu really free?** Yes: free and open source, with every feature. You pay only the AI
provider you choose, if you choose one.

**Does it work in French?** The interface is in English and French. Documents can be in any language the model reads.

**Can I trust the result?** As much as you checked it. Kalamu makes checking quick and keeps a
record of it; it cannot make a model reliable.

**Something else?** Open an [issue](https://github.com/fmadore/Kalamu/issues) on GitHub, or
look up a word in the app's **Glossary**.
