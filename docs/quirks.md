# Quirks

Where reality and the specification disagree.

An operating system meets three kinds of reality that no document describes
correctly: hardware and firmware, applications being driven through their own
automation, and pinned upstream engines behaving unlike their manuals. When you
lose an afternoon to one of them, write it down here. The next person should
inherit the knowledge, not the debugging session.

## How to write an entry

One entry per quirk, newest first within each section. Every entry says: **what
it is, what version, what actually happens, what we do about it, and the date.**
A quirk with no version and no date is a rumour.

Keep the accommodation and the reason together. Six months from now the code
will look wrong to somebody, and this file is the only thing standing between
them and reintroducing the bug.

The rule this file serves: **strict in what we do, tolerant in what we accept.**
We behave correctly; we cope with hardware and applications that do not.

---

## Hardware and firmware

_(no entries yet)_

<!--
### <Machine or component> — <one-line summary>
**Version:** firmware / kernel / driver version the behaviour was seen on
**Behaviour:** what actually happens, as opposed to what is documented
**Our response:** what we do, and why this rather than something else
**Date:** YYYY-MM-DD, and who saw it
-->

## Application automation

Applications driven through adapters (`docs/contracts/app-adapters.md`) change
their automation surfaces between versions, sometimes silently. This is where
that gets recorded: which version, what changed, and what the adapter now does.

_(no entries yet)_

<!--
### <Application> <version> — <one-line summary>
**Mechanism:** api | accessibility | dbus | synthetic
**Behaviour:** what the API or the accessibility tree actually does
**Our response:** what the adapter does about it
**Date:** YYYY-MM-DD
-->

## Pinned engines

The kernel, Mesa, systemd, the model runtime and the fine-tuning stack are
configured, never patched. When one of them behaves unlike its documentation,
the accommodation lives in our configuration and the reason lives here.

An entry here that says "we patched it" is a bug in the process: a source patch
to an engine requires an ADR first.

_(no entries yet)_

<!--
### <Engine> <version> — <one-line summary>
**Behaviour:** what it does, versus what is documented
**Our response:** the configuration we apply, and why
**Upstream:** issue link if reported
**Date:** YYYY-MM-DD
-->

## Filesystems and paths

A grant is over a place, and a path is only a name for one. Where the two come
apart, a capability check can be correct and still be wrong — so this is where
that gets written down rather than discovered.

### A zip has nowhere to say which clock its timestamps came from
**Version:** the zip format as every reader implements it; seen 2026-09-02 in
`alo-files`, against Windows 11 26200's own reader
**Behaviour:** a zip keeps each file's time as a DOS date and time, which
carries no timezone and is **conventionally the local time of whoever wrote the
archive**. `std` cannot say what this machine's offset from UTC is, and every
crate that can does it either through a dependency whose local-offset lookup is
unsound in a threaded process or through code this repository forbids.
**Our response:** the moment written is **UTC**, consistently, and it is
documented where the archive is made rather than left to be discovered. A reader
on a machine two hours ahead of UTC shows a file archived at 20:04 as 18:04.
Seconds are also kept in two-second steps, which is the format and not us. The
alternative — a guessed offset, or a dependency to find the real one — would be
wrong more interestingly rather than less often.
**Date:** 2026-09-02

### Resolving a path does not defeat a hard link
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** `alo-files` resolves every path a verb names and asks the grants
about where it really leads, which stops a symbolic link out of a granted
folder. A **hard** link is not a link in that sense: it is a second real name
for the same file, so a hard link inside a granted folder to a file that also
lives outside it resolves to the granted name and passes the check.
**Our response:** nothing in the path layer, because there is nothing honest to
do there — the granted name genuinely is a real name for that file. Making a
hard link needs write access to the granted folder and read access to the
target, so it is not a way *in*; it is a way for somebody who can already write
to a granted folder to widen what an agent may read. It is documented here, in
the contract and in `alo-files`, and the answer if it ever matters is a policy
about link counts at the moment of opening, not a cleverer path comparison.
**Date:** 2026-09-02

### A path checked and then opened by name can change in between
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** the real path is resolved, the grants permit it, and then the
file is opened by that name. Anything with write access to a folder on the way
can swap a link in between the two.
**Our response:** the check is where it can be, and the fix is not another
check. Whatever opens the file holds on to *what it opened* rather than
resolving the same name twice. The acting half in `alo-files` does as much of
that as `std` allows — a file is opened once and its size is asked of the open
handle rather than of the name, and nothing resolves a path a second time — and
what `std` does not allow is the rest: opening relative to a directory handle
(`openat`) and renaming without replacing (`renameat2` with `RENAME_NOREPLACE`)
are Linux calls with no portable spelling, so a destination is checked and then
renamed onto, with a gap between the two. Narrowing it is item 6b in
`docs/autonomy/QUEUE.md`, written down rather than left to be rediscovered.
**Date:** 2026-09-02, extended 2026-09-02 by the acting half

### Windows returns a path spelled differently from the one it was given
**Version:** Windows 11 26200, Rust 1.97 `std::fs::canonicalize`
**Behaviour:** canonicalising `C:\Users\x\Temp\Invoices` gives
`\\?\C:\Users\x\Temp\Invoices`. The two are the same folder and compare as
different paths, component by component, because the verbatim prefix is a
component.
**Our response:** none in the comparison, which is right to be exact — a grant
that matched loosely would match more than the person picked. **A grant is made
over a resolved path**: the folder a person picks is resolved when they pick it,
so both sides of every later comparison are spelled the way the machine spells
them. Written into the contract and asserted in `alo-files`' integration test,
which grants a resolved folder for exactly this reason.
**Date:** 2026-09-02

## Languages and counting

A sentence with a number in it is the one string that cannot be translated
line for line. Where what a plural form is called and what it actually covers
come apart, write it here — because the person who would notice is the person
reading that language, and there is nobody here who reads all 24.

### A plural form's name says nothing about which numbers it covers
**Version:** CLDR cardinal rules, `common/supplemental/plurals.xml` from
`unicode-org/cldr`, read 2026-09-02. Not a disagreement with CLDR — CLDR is
right — but with what the names lead a reader to assume.
**Behaviour:** three assumptions that all look safe and are all wrong. **Every
language has `other`:** Polish does not, for a whole number — its `one`, `few`
and `many` cover every integer between them, and CLDR's Polish `other` has
decimal samples only. A file offering a Polish translator `one` and `other` asks
them for one sentence nothing will ever show and leaves out the two that most
numbers take. **`one` means one:** Croatian's `one` covers 1, 21, 31 and 101;
French's covers 0 as well as 1; Latvian's `zero` covers 0, 10, 11 and 20 alike.
A translation that spells the number out — *jedna datoteka* — is then shown to
somebody with twenty-one files. **A form is picked by the number:** it is picked
by the number *and the language*, so English's forms cannot be used to look up a
Polish sentence.
**Our response:** `alo-strings`' `cldr.rs` holds the rules as code with each
CLDR condition quoted beside the arm it became, and three things are refusals
rather than conventions. A translation into a form its own language never uses
is refused, naming the forms it does use. A form may leave the number out only
where `names_one_number` says exactly one whole number takes it. A countable
string translated into a language whose rules are not in the table is refused
outright, in words addressed to whoever is contributing that language — nothing
falls back to English's two forms, because a sentence wrong for most numbers in
a language nobody here reads is worse than one that has not arrived.
**Date:** 2026-09-02

### Half the keys on a keyboard are not printed with a word anybody translates
**Version:** physical keyboard layouts, EU national variants, observed 2026-09-02.
**Behaviour:** *what a key is called* looks like one list of strings and is two.
`Q`, `7`, `,` and `F1` are printed identically on every keyboard sold in the
union, and translating them is not translation at all — it names a **position**,
which is the model `alo-shortcuts` exists to reject, since `Super+Q` on a French
keyboard is the key marked Q and not the one where an English keyboard has Q. The
other sixteen print a *word*, and it is a different word almost everywhere: a
German keyboard prints **Entf** for Delete, **Einfg** for Insert, **Pos1** for
Home, **Strg** for Ctrl and **Bild ↑** for Page Up; a French one prints **Maj**
for Shift. A shortcuts panel translated from one English list would either name
keys that are not on the keyboard in front of the person, or invite a translator
to render `Q` as `Й`.
**Our response:** the two kinds are different questions in the code.
`Key::mark` answers for the fifty-three that print a mark and is not a string at
all; `Key::said` answers for the sixteen that print a word, each declared in
`alo-shortcuts`' `words` with a note naming what a keyboard in another country
prints; and `Key::shown` is what a panel draws for either. Declaring all
sixty-nine was the alternative and is worse twice over: it hands a translator
forty-one rows reading `A`, `B`, `C`, and it makes `Strings::unanswered` — *what
a release note has to count* — report fifty-three strings nobody should ever
translate.
**Date:** 2026-09-02

### A machine cannot punctuate a list it assembled
**Version:** Greek orthography; CLDR list patterns, `common/main/*.xml`, not
implemented here.
**Behaviour:** a sentence that names two or more things has to join them, and
the joining is not punctuation a program can pick. Greek writes `;` where
English writes a question mark and `·` where English writes a semicolon, so a
list joined with `"; "` reads as a row of questions to the people it is for. The
conjunction before the last item is a word — *and*, *und*, *et* — that would
have to be its own string, placed by a machine that does not know the sentence.
**Our response:** no sentence in this repository joins a list. Where one thing
is named it goes in a gap, as `alo-shortcuts`' *{chord} is already {action}*
does; where two or more are, the sentence says so and the things are handed over
to be drawn as rows — `Clash::said` names the chord and `Clash::actions` hands
over what wants it, each said in the reader's own language. If a sentence ever
genuinely needs a list inside it, the list patterns are CLDR data like the plural
rules and are read rather than recalled.
**Date:** 2026-09-02

### A deserialiser is required to have a sentence and has nobody to ask for one
**Version:** `serde` 1, `#[serde(try_from = "…")]`, observed 2026-09-02.
**Behaviour:** every value in this repository that a settings file holds is
checked again on the way in, because a settings file is a thing a person edits —
a colour, a screen's name, a rotation, a schedule, a text size, a time of day, a
key combination. `serde` implements that with `try_from`, and it requires the
error to have a `Display`, because it turns it into a message with
`de::Error::custom`. That is exactly the thing our rule forbids: a `Display` on a
user-facing refusal is an English sentence one `to_string()` away from a screen.
And the deserialiser is the one caller that genuinely cannot obey the rule the
other way either — it is handed a value and a format, never the language the
person in front of the machine reads, and there is no argument to give it one.
**Our response:** what a refusal writes at that point is the **key** of the
string rather than the string. `alo-appearance`'s `NotRead` is that, shared by
its six deserialisers, and `alo-shortcuts`' `Chord` has a private one of its own
from item 9c. Whoever reports a settings file that did not read looks the key up
and shows the same words a settings panel shows for the same refusal — one
rendering, in the reader's own language, rather than an English line in a log
beside a translated line on a screen. The refusal itself is unchanged: the same
files are refused as before, and `said(&Strings)` is still the only road to
words. What is given up is `std::error::Error` on ten types that were never
errors a programmer handles.
**Date:** 2026-09-02

### Two gaps in a translated sentence arrive in the language the code was written in
**Version:** `alo-capability` at item 9e, observed 2026-09-02.
**Behaviour:** a translated sentence is only as translated as what goes into
its gaps, and two here come from somewhere that has not moved yet.
`capability.call.missing` — *{verb} needs {argument} — {purpose}* — fills
`{purpose}` from what the verb was declared with, which is the source string
rather than the reader's; the crate that declares the verb has the translation
(`alo-files`' `saying::purpose_of` answers with it), and the crate that refuses
the call does not, because a `Verb` carries the declaration and not a key.
`capability.answer.lapsed` quotes the approval sentence, which
`alo_capability::Call` renders at the moment the call is made and keeps as a
string. So a German machine can read a German sentence with an English clause
inside it, which is exactly the failure `alo-appearance` closed for colour
names in item 9d.
**Our response:** the note on each of those two words says so, in the words a
translator needs — *it arrives in the language the verb was declared in* — so
nobody spends an afternoon looking for the string that would fix it. The fix is
item 9g in `docs/autonomy/QUEUE.md`: a `Call` carrying a key and a filling
rather than a rendered sentence, which makes the approval, the record and the
screen one thing. It is written down here rather than worked around because
working around it would mean a second copy of a declaration, and one string
rather than two that agree is the rule the whole 9-series is built on.
**Date:** 2026-09-02

## Models

Open-weight models in the catalogue have their own personalities: refusing
formats they claim to emit, ignoring stated context limits, or answering in the
wrong language. Where a model in the catalogue misbehaves in a way that affects
the agents, record it here with the exact model and quantisation — "it was fine
for me" is usually a different quantisation.

_(no entries yet)_

## Providers and their APIs

A provider somebody adds themselves is a service nobody here operates, behind an
address nobody here chose. Where the convention every provider claims to follow
turns out to be followed differently, record it here — with what the evidence
actually is, because a provider's documentation is not a run against it.

### An OpenAI-compatible address is documented both with and without `/v1`
**Version:** documented behaviour as of 2026-09-02 — Mistral publishes an
address ending `/v1`, the pinned runtime's OpenAI-compatible surface is the bare
address with `/v1/…` beneath it. **Not yet observed against either live
service**; the tests in `alo-models` are against a stub on a real socket, and
checking this against a provider somebody pays for is owed alongside the rest of
the hardware verification.
**Behaviour:** there is no single spelling of "the address of the API". Half the
world writes `https://api.example.com/v1` in the settings field and half writes
`https://api.example.com`, and appending `/v1/models` to the first gives
`/v1/v1/models` and a 404.
**Our response:** `trying.rs` appends `/v1/models`, or just `/models` when the
address already ends `/v1`. It is one line and it is deliberately not cleverer
than that: a 404 here would be read by a person as *my address is wrong* when
their address was right, which sends them to change the one thing that was
correct. A provider that answers on neither is reported as one this system
cannot use, which is what it is.
**Date:** 2026-09-02
