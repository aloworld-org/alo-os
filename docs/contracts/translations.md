# Contract — a translation

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file a translation of alo OS arrives in. It is written by whoever
translates the system — for the 24 official EU languages to begin with, and any
language somebody contributes after that — and it is read once, when a process
starts.

It is the fourth of the files this repository describes.
`docs/contracts/daemon-protocol.md` is what a client says to the agent service;
`docs/contracts/record-file.md` is what that service writes down;
`docs/contracts/machine-description.md` is what a machine is told about itself.
This one and the description are the two a **person** types, and this is the
only one a person outside the organisation that owns the machine types.

## Where they are

```
/usr/share/alo/translations
```

Every file ending `.toml` directly in that directory is a translation. Anything
else in it — a note, a folder, a file with another suffix — is not one and is
left alone, so an image may put a README beside them.

**They arrive in the image.** alo OS is a bootable container image (ADR 0011):
the operating system *is* the image, `/usr` is part of it, and a translation
ships and is replaced the way the rest of the system does. That is not an
implementation detail, because of what a translation now contains — see *Who
may write one*, below.

## The shape

TOML, for the reason `docs/contracts/machine-description.md` is TOML: the
record and the protocol are JSON because a program writes them and a program
reads them, and this is typed by a person who needs somewhere to leave a note
about why a sentence is worded the way it is.

```toml
format = 1
language = "de"

[says]
"files.gone" = "Es ist nicht mehr da"
"files.too-big" = "{path} ist {bytes} Bytes groß, und ein Verb liest höchstens {most}"
"files.found.one" = "1 Datei"
"files.found.other" = "{how_many} Dateien"
```

| Field | Meaning |
|---|---|
| `format` | Which shape this file is in. Required. `1` today. |
| `language` | Which language it is written for, as a BCP 47 tag. Required. |
| `says` | The sentences, one per line, keyed by the name of the string. Optional — a language somebody has started and not written any of yet is a file with no `says` in it. |

**`format` is answered before anything else in the file.** A file written for a
later alo OS is refused *as one*, rather than as whichever of its keys this
version happened not to know. It is the rule the record file and the machine
description both state, and it is why a key this version does not know is a
typo rather than a guess.

**The language is in the file and not in the name.** A file named `de.toml` and
a file named `deutsch.toml` are the same translation if both say `language =
"de"`; the name is only ever used to tell somebody which file was wrong. A tag
is normalised when it is read, so `PT-br` and `pt-BR` are one language and not
two.

**Two files for one language: the first by name is read and the second is
reported.** Files are read in the order their names sort, so which one that is
is a fact about the image rather than about the disk. Put the two together into
one file.

## The sentences

**A key names a string the running system says.** Every one of them is declared
in the code, with its English and a note to the translator beside it; the part
before the first dot says which part of the system it comes from.

**A gap is `{name}` and it may be moved but not removed.** `{path}`, `{bytes}`,
`{how_many}` are where a file name or a number goes. A sentence that drops one
would reach somebody as *your file is too big* with no size in it, in their own
language, with nothing anywhere saying so — so a line that drops a gap the
English has, or invents one it does not, is **left out**, and the rest of the
file is shown. Where the gap goes in the sentence is the translator's; a
language that puts the number first puts the number first.

**A sentence that counts something is written once per form.** `files.found.one`
and `files.found.other` are two lines of one string, and which forms a language
has is that language's — Polish has `one`, `few` and `many` and no `other` for a
whole number; Latvian has a `zero`. A form the language never uses for a whole
number is left out, and so is a countable string in a language whose plural
rules alo OS has not read.

**A partial translation is normal.** Missing lines are not an error: a language
arrives a few hundred strings at a time, and what is not translated is shown in
English and is marked as English wherever it appears.

## What happens when something is wrong

**Nothing here stops a machine.** A translation that is missing, unreadable,
half written or from a later alo OS leaves the machine speaking English, and
what went wrong is written into the service log. A machine that would not start
over a translation could not say why: the sentence explaining it would be in the
file that did not load.

**A line is left out; a language is never thrown away.** A key nothing says any
more, a dropped gap, an invented gap, a form the language does not use — each of
those costs that line and nothing else. This is deliberately more forgiving than
the check a contribution is held to, and for a reason: a string renamed in a
release would otherwise turn a person's language off, on every machine at once,
in the release that renamed it.

**A key a particular process does not say is an ordinary line to leave out.**
One machine has one vocabulary, and a process may say less than the machine
does — the agent service says three strings the shell does not. A translation
covers the machine, and a process leaves out what it does not say.

## Who may write one

**A translation is not decoration.** Since the sentence a person approves became
a string like any other, `delete {path}` is a line in one of these files —
so whoever can write them can change what somebody is agreeing to, on a machine
behaving exactly as it was built to.

What answers that is the image, not a permission check. These files are part of
a release that was built and signed, in a directory that is not writable on a
running machine, and alo OS deliberately does **not** check their owner or their
mode: a check on a path under `/usr` would be theatre, and would teach whoever
packages a machine that this is a directory files may be dropped into.

**A translation a person adds to their own machine is a different question and
does not have an answer yet.** `docs/features.md` puts community translation at
v1. It will be a different directory, because it is one somebody can write, and
nothing in alo OS reads such a directory today.

## What is **not** in it

**Which language a person reads.** That is their setting, not a property of the
translations. Every translation on the machine is loaded and the person's choice
decides which is preferred; a language they asked for falls back to its broader
form before it falls back to English, so somebody who asked for `pt-BR` and met
a string only `pt` has is shown `pt`.

**How a number, a date or a size is written.** That belongs to the region rather
than to the language — somebody reading Swedish in Finland writes a time the
Finnish way — and the text that fills a gap is made before it gets here.

**What a language is called.** A language is named in its own language, and the
24 are named in the code. A language somebody contributes shows its tag until
its own name is added beside it.

## What changes additively

New fields may be added and `format` stays `1` for as long as an older alo OS
reading the file without them would show the same sentences. Anything else — a
field removed, a meaning changed, the gap syntax altered — is a new `format`,
and a machine refuses a number it does not read.

**Strings themselves are not versioned by this number.** Keys are added and
removed as the system changes, and both are ordinary: a key that has gone is a
line left out with a sentence saying so, and a key that is new is one that is
not translated yet.
