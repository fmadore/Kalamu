---
layout: guide
title: Costs and privacy
order: 8
lede: What a job costs, how to pay less, and exactly where your files go.
description: Estimated costs, the cheaper batch mode, caching and privacy in Kalamu.
---

[Français](fr/costs.md)

## What a job costs

Kalamu is free. With a cloud model you pay the provider, on your own account, for what you
send and what the model writes. Prices differ a great deal between models: a provider's
smallest model can cost a tenth or less of its largest.

Before a job starts, **Estimated cost** shows what it should come to, from the page counts and
the provider's published prices; a recording, from its length. When Kalamu cannot read how long
a recording lasts, the estimate reads *length unknown*, and the cost shows in **Jobs** as the job
runs. After it, the job shows what was actually used. The
provider's invoice is the reference; prices change, and Kalamu's price list carries its date.
Kalamu fetches a newer list from its repository at startup (**Check at startup** in **Settings**,
under **Models**), and checks it before using it: a list that fails Kalamu's checks is not
used, Settings says so under the list's date, and Kalamu keeps the list it has.

A [server of your own](other-servers.md) costs nothing unless its owner charges: its jobs read
*not priced* unless you gave its models a price in Settings.

A rule of thumb: try the smallest model first on a few typical pages. Many collections of
clean print do not need more.

## Three ways to pay less

- **Never pay twice.** Every page's result is saved as it arrives. A job that fails halfway, or
  that you stop, resumes without sending again what is done. Running a job again with the same
  files, model, prompt and settings costs nothing: Kalamu recognises the work and reuses it.
- **The cheaper mode.** For a large collection that is not urgent, choose **Cheaper** in New
  job: the job goes to the provider's batch service at half price and is answered within a
  day, often sooner. You can close the app in the meantime; the job is collected when you
  open it again. Two kinds of work are still sent at once, at the usual price, because Kalamu
  checks each answer against the text as it arrives: an analysis that quotes the text (such as
  screening against a research question, or an interview analysis) and a transcript translated
  line by line with its times. The estimate counts them at the usual price, and under **Speed
  and price** New job says which models this applies to; when nothing in the job can go in a
  batch, **Cheaper** is not offered.
- **Offline models** cost nothing at all. See [Working offline](offline.md).

## Where your files go

- **To the provider you chose, and nowhere else.** Files go directly from your computer to
  Google, OpenAI, Anthropic or Mistral, with your own API key. Kalamu has no server: its author
  never sees your files, your keys or your results.
- **Your keys** are kept in Windows Credential Manager or the macOS Keychain and sent only to
  their provider.
- **Your results** are kept on your computer, in Kalamu's data folder, until you delete them.
- **Offline**, nothing leaves the computer.
- **A server of your own** receives your files at the address you gave: at `localhost` they
  stay on your computer; on the network they go to a machine someone runs, under that person's
  or institution's rules, and Kalamu asks first. See
  [Other servers](other-servers.md#what-is-sent-and-where).

What each provider does with what it receives (how long it keeps it, whether it may be used to
improve its models) is set by that provider's terms for API use, which are generally stricter
than for its consumer chat app, and by any agreement your institution has. If your sources are
sensitive (interviews with living people, restricted archives), read those terms, ask your
institution, and consider working offline.

The [privacy policy](../privacy.md) says in full what Kalamu sends, to whom, and what it keeps.

## Before you send sources you do not own

Check that you are allowed to: an archive's reading-room rules, a consent form, a publisher's
licence. Kalamu cannot know; it sends what you give it to the provider you choose.
