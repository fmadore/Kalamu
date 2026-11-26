---
layout: guide
title: Other servers
order: 7
lede: Your university may run AI models for its members, or a stronger computer in your office may run Ollama. Kalamu can use their models, once it has tried each one.
description: Adding Ollama or a university's OpenAI-style server to Kalamu, what its models are offered for, and where your files go.
---

[Français](fr/other-servers.md)

## Add Ollama on this computer

If [Ollama](https://ollama.com) is installed and has a model (see
[Before you start](#before-you-start)):

1. Open **Settings**, then **Other servers**, and choose **Add a server**. Give it a **Name**
   of your own (it is shown wherever a provider's name is) and the **Address**
   `http://localhost:11434`. Leave **This server asks for a key** unticked. Press **Connect**.
2. Kalamu says what it found, such as *Ollama 0.40.18, on this computer.*, and lists the
   **Models on this server**. Tick the ones to use.
3. Press **Try and add**. Kalamu tries each model, shows what it can do, and saves the server.
   Its models are now in **New job**.

The rest of this page explains each step, what the trial decides, and what to do when it does
not work.

## When it is worth it

- **Your institution offers a service** of its own, usually reached at an address that "speaks
  like OpenAI's", and you are allowed, or asked, to send your material there rather than to a
  company.
- **Another computer in your office runs Ollama**, with more memory or a better graphics card
  than yours.
- **You want a larger model than the ones Kalamu runs itself** under
  [Offline models](offline.md), and your computer has the memory for it.

When not to bother: the [offline models](offline.md) already in Kalamu need no set-up at all;
and the four cloud providers read difficult scans and handwriting better than most models a
single machine can run. Try a few of your own pages before you move a whole collection.

Nothing about other servers shows in New job until you add one.

## Before you start

**Ollama** is free and runs on Windows and macOS.

1. Download it from [ollama.com](https://ollama.com) and install it. It then runs in the
   background, its icon in the notification area on Windows and in the menu bar on a Mac.
2. Open a terminal (**Terminal** on Windows, from the Start menu; **Terminal** on a Mac, in
   Applications › Utilities) and download a model that reads images:

   ```
   ollama pull gemma3:4b
   ```

   `ollama pull` downloads the model named after it, once. A larger one, such as `gemma3:12b`,
   reads better and needs more memory. Names change: look for models marked *vision* in
   [Ollama's model library](https://ollama.com/search). A model that reads only text can still
   do analyses, corrections and translations, but not read pages.
3. `ollama list` shows the models you have downloaded.

**Ollama on another computer of your network** answers only its own computer until its owner
tells it otherwise. On the computer that runs Ollama, set `OLLAMA_HOST` to `0.0.0.0:11434`, as
[Ollama's documentation](https://docs.ollama.com/faq#how-can-i-expose-ollama-on-my-network)
explains for each system, then restart Ollama. In Kalamu, the address is then
`http://<that computer's name or address>:11434`.

**A university's or computing centre's service** gives you its address, and usually a key. Ask
whoever runs it which models read images.

## Add a server, step by step

### 1. Name and address

In **Settings › Other servers**, choose **Add a server**. The **Name** is yours to choose. The
**Address** is where the server answers: Ollama on this computer answers at
`http://localhost:11434`. Tick **This server asks for a key** only if it does, and paste the
**Key**. Press **Connect**.

### 2. What Kalamu found

Kalamu asks the server what it is and which models it has; there is no kind of server to
choose. It says so in one line, such as *Ollama 0.40.18, on the network at 192.168.1.20.* or
*An OpenAI-style server, on this computer.* For an address that starts with `http://` on the
network, it warns that your files travel unencrypted (see
[What is sent, and where](#what-is-sent-and-where)).

Under **Models on this server**, each model shows its size and what the server says of it
(*reads images, the server says*, or *does not read images, the server says*). Tick the models
to use. A model that *Runs at ollama.com, not on this server, so it cannot be added.* cannot be
ticked: Ollama would pass your pages on to a company's servers, unseen behind an address on your
own computer.

### 3. More choices, if you need them

You need touch none of this to start: Kalamu fills every value. Each model has its own
**More choices**:

- **Reads up to**: how many tokens (pieces of words) the model reads at once, its answer
  included. Kalamu starts from what the model was trained for, up to 32,768. More lets it read
  longer pages and texts, but takes more of the server's memory.
- **Longest answer**: the most tokens it may write in one answer.
- **Price per million tokens read** and **Price per million tokens written**: in US dollars,
  if your institution charges. Left empty, a job's cost reads as *not priced*.

Below the models, the server's own **More choices** holds **Requests at the same time**, from 1
to 8, one by default. More is faster only if the server answers several at once.

Press **Try and add**.

## The trial, and what it decides

Before a model is offered anywhere, Kalamu tries it with three small requests of its own: a
one-line question; a small picture, bundled with Kalamu, that holds a number; and a short form
of two fields to fill about one sentence. **None of your files is sent.**

Each model then shows two lines, each **Yes** or **No**:

| Line | Yes means |
| --- | --- |
| **Reads pages** | it read the number in the picture: offered for text extraction. |
| **Fills an analysis’s form** | it filled the form correctly: offered for analyses, corrections and translations. |

With both, it is offered for tables of contents too. A model that did neither is not added.

You cannot overrule the trial, and that protects you: a model that cannot see a picture would
otherwise answer every page of a forty-page scan with an invented or empty reading, and look
just as confident doing it.

- The first trial can take **a few minutes** while the server loads the model; **Stop** ends it,
  and the model is not added.
- Updated a model on the server? **Try again**, beside it in **Your servers**, repeats its trial.
- **"No" for Reads pages** usually means a model that reads only text; choose one that reads
  images. **"No" for the form** usually means a model too small to keep to a form; choose a
  larger one.

## Use it

In **New job**, the server's models sit under their own heading, **Other servers**, each named
with the name you gave the server, and only for the tasks its trial showed. They read one page
at a time. The cheaper mode does not apply to them. They are not offered for transcribing
recordings (Whisper, under [Offline models](offline.md), does that on your computer) nor for
the [search by meaning](projects.md#search-by-meaning-and-the-map).

## What is sent, and where

- **On this computer** (`localhost`), nothing leaves it, and Kalamu asks nothing.
- **On the network**, your files go to a machine someone runs, under that person's or
  institution's rules, not to a company under your account. Before the first job, Kalamu asks
  once, naming the server and its host (*Send your files to another server?*), and asks again
  if the address changes, before a job made earlier is resumed there.
- **An address that starts with `http://`** sends your files, and the key if there is one,
  unencrypted over the network. That is usual inside an institution's network, and a reason
  not to use such an address across the internet; ask whoever runs the server for an
  `https://` address.
- **Kalamu knows only the address.** What the program there keeps, and who can read it, is up
  to whoever runs it: ask them.
- **The key** is kept in Windows Credential Manager or the macOS Keychain, and sent only to
  that server.

An export's `provenance.json` names the server and its address. The
[privacy policy](../privacy.md) says the same in full.

## What it costs

Nothing, unless your institution charges. Without prices under **More choices**, the cost of a
job reads *not priced*.

## Never a reading in part

A page or text longer than the model reads on that server is refused, never cut short in
silence. The page fails with a message that begins *The page or text is longer than* the model
*reads on* the server. Two remedies: raise **Reads up to** for that model, if the machine has
the memory; or choose a model that reads more.

With Ollama, Kalamu tells the server to refuse rather than cut. An OpenAI-style server cannot
be told so: Kalamu compares the number of tokens the server says it read with the text sent,
and refuses an answer that read only part of it. A server that does not say how much it read
is trusted on its word.

## Change or remove a server

In **Your servers**, **Edit** opens the same three steps; a model already tried at the same
address keeps its trial. **Remove** asks first: *Its jobs and results stay. Saved pipelines and
workflows that use it will fail until it is added again.* A server cannot be changed or removed
while a job is running on it: pause the job, or let it finish.

## When it does not work

| Kalamu says | What to check |
| --- | --- |
| *Nothing answers at* … | The address, and that Ollama (or the server's program) is running on that computer. For another computer, that its owner set `OLLAMA_HOST` (see [Before you start](#before-you-start)). |
| *Something answers at* …*, but it is not a server Kalamu knows* | The address: check it with whoever runs the server. |
| *The server at* … *asks for a key.* | Tick **This server asks for a key** and paste it. |
| *The server at* … *refused the key.* | The key, with whoever runs the server. |
| *The page or text is longer than* … | Raise **Reads up to**, or choose a model that reads more (above). |
| … *is no longer on* … | The model was removed or renamed on the server: connect again with **Edit** and tick its new name. |

Anything else: [When something goes wrong](help.md).

## Still new

Other servers are new in Kalamu. They have been tried against answers recorded from Ollama,
not yet against every kind of server. If something does not work as this page says, please
[tell us](help.md#questions).
