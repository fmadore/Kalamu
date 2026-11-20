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

Open the job: each failed page says why, in words. The usual causes are a lost connection, the
provider being busy (Kalamu waits and tries again by itself), or a page the provider refuses.
Run the job again: only what failed is sent.

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

Look at the job's actual usage, and at the model's price in the list. Long handwritten pages
and analyses with long answers cost more than the estimate's averages. For the next batch:
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
