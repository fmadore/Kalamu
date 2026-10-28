# Kalamu licence terms

For the Microsoft Store listing (Partner Center, Store listings, "Additional license terms"),
where they replace Microsoft's Standard Application License Terms, and for the GitHub
releases.

---

Kalamu is free software by Frédérick Madore, released under the MIT licence:

> Permission is hereby granted, free of charge, to any person obtaining a copy of this software
> and associated documentation files (the "Software"), to deal in the Software without
> restriction, including without limitation the rights to use, copy, modify, merge, publish,
> distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
> Software is furnished to do so, subject to the following conditions: The above copyright
> notice and this permission notice shall be included in all copies or substantial portions of
> the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED.

Its source code is at https://github.com/fmadore/Scriptorium.

**Third-party software.** Kalamu includes software by others under their own licences,
listed in the app (Settings, About) and in the licences folder installed with it. The author of
Kalamu does not own these components.

- **FFmpeg.** This software uses code of [FFmpeg](https://ffmpeg.org/), licensed under the
  [GNU Lesser General Public License version 2.1](https://www.gnu.org/licenses/old-licenses/lgpl-2.1.html)
  or later. FFmpeg is a trademark of Fabrice Bellard. Its source code, as used in each
  release, can be downloaded from that release's page at
  https://github.com/fmadore/Scriptorium/releases. It runs as separate programs, which you
  may replace with your own build.
- **Opus**, built into FFmpeg: BSD 3-Clause licence, © Xiph.Org Foundation and contributors.
- **whisper.cpp** (MIT) and **whisper-rs** (Unlicense), for offline transcription. Whisper
  models are downloaded on request from Hugging Face; they are MIT-licensed by OpenAI.
- **PDF.js** (Apache License 2.0), © Mozilla Foundation and contributors, for page previews.
- Other open-source libraries under permissive licences (MIT, Apache 2.0, BSD and similar),
  listed with their licence texts in the third-party notices installed with the app.

**Reverse engineering.** Nothing in these terms restricts you from modifying Kalamu or
the components above for your own use, or from reverse engineering it to debug such
modifications.

**Online services.** When you choose a cloud provider (Google, OpenAI, Anthropic or Mistral),
your files are sent directly from your computer to that provider under your own account and its
terms. Kalamu sends nothing to its author.
