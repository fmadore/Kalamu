---
layout: guide
title: Other servers
order: 7
lede: Your university may run AI models for its members, or a stronger computer in your office may run Ollama. Kalamu can use their models, once it has tried each one.
description: Adding Ollama or a university's OpenAI-style server to Kalamu, what its models are offered for, and where your files go.
---

[Français](fr/other-servers.md)

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

**A university's or computing centre's service** gives you its address, and usually a key. Ask
whoever runs it which models read images, then go to
[Add a server, step by step](#add-a-server-step-by-step).

**Ollama** is free and runs on Windows 10 or 11 and on macOS 14 or later. It is a program that
runs models on your own computer; Kalamu then talks to it. Setting it up takes four steps, and
the only typing is two short commands.

### 1. Install Ollama

Download it from [ollama.com](https://ollama.com) and install it like any other program. It
then runs in the background, its icon in the notification area on Windows and in the menu bar
on a Mac. If you quit it, open it again from the Start menu or Applications before using it
in Kalamu.

### 2. Choose a model for your computer

A model must fit in your computer's memory (RAM, or a Mac's unified memory) with room to
spare, or it runs very slowly or not at all. To read pages it must also read images: Ollama's
library marks such models *vision*. Find your memory under **Settings › System › About** on
Windows, or **Apple menu › About This Mac** on a Mac, then pick:

| Your computer's memory | A model to start with | Download |
| --- | --- | --- |
| 8 GB | `qwen3-vl:4b` or `gemma3:4b` | about 3.5 GB |
| 16 GB | `gemma4:12b` | about 8 GB |
| 32 GB or more | `gemma4:26b` | about 17 GB |

A larger model reads better and more slowly. A graphics card (NVIDIA or AMD) on Windows, or any
Mac with Apple Silicon, makes it much faster; without one, a page can take minutes. These names
were current in November 2026 and change often: if one is gone, look for a model marked
*vision* in [Ollama's model library](https://ollama.com/search?c=vision), of about the same
download size. A model that reads only text can still do analyses, corrections and
translations, but not read pages.

### 3. Download the model

Open a terminal: on Windows, **Terminal** from the Start menu; on a Mac, **Terminal** in
Applications › Utilities. Type `ollama pull` and the model's name, then press Enter, for
instance:

```
ollama pull gemma4:12b
```

The download runs once, shows its progress and can take a while: the model is several GB. When
it ends, the model stays on your computer.

### 4. Check that it works

In the same terminal, type:

```
ollama list
```

It lists the models you downloaded. If it says instead that it could not connect to Ollama,
Ollama is not running: open it again (step 1) and retry. You can close the terminal now.

### Ollama on another computer of your network

Ollama answers only its own computer until its owner tells it otherwise. On the computer that
runs Ollama, set `OLLAMA_HOST` to `0.0.0.0:11434`, then quit Ollama and open it again:

- **Windows**: in the Start menu, search for *Edit environment variables for your account*. Under
  the user variables, choose **New…**, name it `OLLAMA_HOST` and give it the value
  `0.0.0.0:11434`.
- **Mac**: in Terminal, type `launchctl setenv OLLAMA_HOST "0.0.0.0:11434"`.

[Ollama's documentation](https://docs.ollama.com/faq#how-can-i-expose-ollama-on-my-network)
has the details. If that computer's firewall asks whether Ollama may accept connections, allow
it, on private networks only. In Kalamu, the address is then
`http://<that computer's name or address>:11434`.

## Add Ollama on this computer

Once Ollama runs and has a model:

1. Open **Settings**, then **Other servers**, and choose **Add a server**. Give it a **Name**
   of your own (it is shown wherever a provider's name is) and the **Address**
   `http://localhost:11434`. Leave **This server asks for a key** unticked. Press **Connect**.
2. Kalamu says what it found, such as *Ollama 0.40.18, on this computer.*, and lists the
   **Models on this server**. Tick the ones to use.
3. Press **Try and add**. Kalamu tries each model, shows what it can do, and saves the server.
   Its models are now in **New job**.

The rest of this page explains each step, what the trial decides, and what to do when it does
not work.

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
address keeps its trial. **Remove** asks first, and says what goes with it: its jobs and
results stay, but a job paused or interrupted on it cannot be finished, and saved pipelines and
workflows that use it will fail. A server added again is a new server, whose models are chosen
again. A server's address cannot be changed, nor the server removed, while a job is running on
it: pause the job, or let it finish. After an address has changed, **Resume** on a job made
before asks again whether your files may go there.

## When it does not work

| Kalamu says | What to check |
| --- | --- |
| *Nothing answers at* … | The address, and that Ollama (or the server's program) is running on that computer: `ollama list` in a terminal says so. For another computer, that its owner set `OLLAMA_HOST` and allowed Ollama through the firewall. Both are in [Before you start](#before-you-start). |
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
