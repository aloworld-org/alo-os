# A film is a film, and not a file nothing recognises

**Date:** 2026-09-17
**Workstream:** `alo-opening` — the crate the documents-and-paper plan left
finished, and the finding the devices walk turned up in it
**Task:** none: this closes the finding reported in
[the walk through a working day's devices](the-walk-through-a-working-days-devices.md),
so that ADR 0051's amendment has the road it names
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**.
**Egress:** none.
**Status:** done.

## What was wrong

[ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)'s
amendment says a format this machine does not play is reported through
`alo-opening`'s cannot-open road — *this is a film, and nothing on this machine
opens it* — and **not** through a second refusal written for media. That is the
right decision and it named a road that did not exist: `alo_opening::Kind` had
eighteen kinds in it, all documents, images and a zip archive.

So a person sent a film met this:

> This machine does not recognise what this file is, so it has not opened it.

Which is what a **corrupt** file reads like. The machine knew perfectly well what
they had.

The evidence was sitting in the crate's own tests, which is the part worth
pausing on: `a_file_nothing_recognises_is_explained` used the first bytes of an
**MP4 file** as its example of something unrecognisable, and so did
`alo-converting`'s. The gap had been written down twice as a fixture.

## What it is now

Nine kinds, read from the containers people are actually sent, each with a name,
a translator's note, a rule over the file's own bytes and an extension in the
claims table:

| Kind | Read from | Extensions that claim it |
|---|---|---|
| `MatroskaVideo` | EBML's signature, and the name in its header | `.mkv` |
| `WebmVideo` | the same, where the header says `webm` | `.webm` |
| `Mp4Video` | `ftyp` at offset four | `.mp4`, `.m4v` |
| `Mp4Audio` | the same, under the brand that means sound with no picture | `.m4a`, `.m4b` |
| `AviVideo` | `RIFF` with `AVI ` at eight | `.avi` |
| `OggMedia` | `OggS` | `.ogg`, `.oga`, `.opus` |
| `Mp3Audio` | `ID3`, or a frame header with all four reserved values checked | `.mp3` |
| `WaveAudio` | `RIFF` with `WAVE` at eight | `.wav` |
| `FlacAudio` | `fLaC` | `.flac` |

**No second refusal shape.** `Cannot::NothingHereOpens(Kind)` and
`Would::AnotherMachineOrFormat` already existed and already read correctly for a
film, so a person meets the same sentence whether what they were sent is a
document or a film:

> This is an MP4 video, and nothing on this machine opens it or converts it into
> something that does. What would open it is a machine with a program for this
> kind of file, or a copy saved in a different format by whoever sent it.

**A kind is the wrapping, not the codec**, and that is deliberate. One Matroska
file holds AV1 that this machine plays and the next holds something it may not,
and telling them apart means reading the tracks rather than the first four bytes.
`alo-opening` answers *what is this file* for every file a person opens;
`alo-playing` will answer *can this machine play it*, against ADR 0051's list,
and `Kind::is_played` says which kinds it is asked about. ADR 0051 now records
that the road exists.

## Three decisions inside it worth naming

- **Matroska's name is read by looking, not by parsing EBML.** The search is
  bounded to the head of a file that already begins with the EBML signature, and
  `webm` is the only name that decides anything; anything else in that container
  is a Matroska file, which is what it is. A full EBML walk would be a parser in
  a crate whose whole point is reading the first few bytes.
- **An MP3 without tags is checked properly.** Eleven bits set is not enough to go
  on: each field after them has a value that means *reserved*, and all four are
  checked, which is what a player does before it decodes anything. Without that, a
  file of bytes that happen to start `0xFF` would be called a sound recording.
- **`.mov` claims nothing.** A QuickTime file is the same container an `.mp4` is,
  so its bytes read as `Mp4Video`; mapping the extension would have meant either
  calling it *an MP4 video* in a way that reads wrong, or inventing a kind for a
  name. A name this crate has no claim for claims nothing, and nothing disagrees.

## What else moved, and why

- **`alo-applications`** (mine): every kind needs a spelling for
  `what-opens-what.toml` and a media type, both held by tests that walk
  `Kind::EVERY`. Nine spellings, sixteen media types, and the rule that a
  spelling is lower case and hyphens **now allows digits** — the formats people
  are sent have numbers in their names, and `mpeg-four-video` is a key nobody
  would guess. `docs/contracts/person-settings.md` lists the kinds and was
  updated with them.
- **`alo-opening`'s own rule that no sentence carries a number** now lists the
  three names that do — MP4, MP3, M4A — and still fails everything else. A number
  in a sentence is almost always machinery leaking; a format's own name is not.
- **`alo-converting`** (the documents-and-paper plan's, finished): one test case
  used an MP4 as its unrecognisable file and now reads as *not a kind this
  converts*, which is more true. Swapped for bytes of no kind. Nothing else in
  that crate changed, and its four converter tests fail here for the reason they
  always do on this architecture (ADR 0039 §5, no engine).
- **The devices walk** moved with it: step 9 is a different sentence, so the
  table is republished as
  [a follow-up](the-walk-through-a-working-days-devices-after-media-kinds.md) and
  the test points at that, which is the convention that test carries.

## And one thing the gate found, which was mine

Publishing this ran `alo-in-use`'s on-a-machine test for the first time since
`alo-sound` gave this machine a media server, and it failed:
`pw-dump failed: can't connect: Host is down`.

The cause is the finding I reported on 2026-09-17 and did not fix, because it was
another plan's crate at the time: **`alo-in-use` clears the environment before
running the media server's tool and does not pass `XDG_RUNTIME_DIR` through.** On
an ordinary login the tool's fallback — `/run/user/` and the user's number — is
the same place, which is why nobody noticed; on a machine whose server listens
anywhere else, the indicator reports *this machine's media server would not
answer* while the server is answering everything else.

It is fixed here, in the crate the capture-and-the-room plan owns, which is this
lane's: the variable is passed through **when it is set**, and the file says what
went wrong and how it was found. An indicator that cannot see the machine's
media server is an indicator that says nothing is watching, which is the one
answer it must never give by accident.

## What this does not do

- **It does not play anything.** Nothing here opens a film; it names one. The
  closed list of what alo OS plays is task 1 of the devices plan and waits on the
  owner's counsel answer about decoders, which ADR 0051 records.
- **It does not look inside a container.** Codecs, tracks, subtitles and chapters
  are `alo-playing`'s.
- **It adds no image or document kinds.** HEIC, AVIF, SVG and EPUB are all things
  people are sent and none of them is here; each is a variant, a word and a rule,
  and none of them was in the way of ADR 0051.
