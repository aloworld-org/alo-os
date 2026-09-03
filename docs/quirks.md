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

### ureq 3.4.0 — `send_json` puts a pretty-printed body on the wire
**Behaviour:** the request body is `serde_json::to_writer_pretty`-shaped —
indented, with a newline after every field — where the obvious assumption is the
compact form. It is documented nowhere either way. Observed on a real socket by
`alo-asking`'s stub, which reads what actually arrives.
**Our response:** nothing is configured, because nothing is wrong: a provider
parses either, and the few hundred extra bytes on a question that is already
kilobytes of somebody's text are not worth a hand-built body. What changed is
the **test**: `alo-asking` asserts on the request body *parsed* rather than on
its text, so it says *these three fields and nothing else* — which is the
promise worth keeping (nothing of the person's leaves except the question) and
is also the assertion that does not break the next time ureq changes its
whitespace.
**Upstream:** not reported; it is not a defect.
**Date:** 2026-09-03

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

## Clocks and moments

A record is evidence about when something happened, so what a moment means when
it is written down, read back and compared is this section's subject.

### `SystemTime` walks back past 1970, and how far is the platform's
**Version:** Rust 1.97 `std::time::SystemTime`, seen 2026-09-03 in
`alo-keeping` against Windows 11 26200
**Behaviour:** a retention rule is naturally written as *keep anything after
`now - 30 days`*, and `SystemTime::checked_sub` is the obvious way to say it.
On Windows a `SystemTime` is counted from 1601, so subtracting thirty days from
a machine whose clock says it is the first minute of 1970 answers with a moment
in **1969** rather than `None`. On a platform where the representation is a Unix
`timespec` the same call can answer `None` instead. Both are correct for the
type; they are not the same boundary.
**Our response:** the window is measured **from the epoch forwards**, not from
`now` backwards. `Keeping::oldest_kept` asks how far `now` is past the epoch,
subtracts the window from *that*, and answers `None` when it does not reach —
so a boundary before 1970 is *nothing is removed*, identically on every
platform. It matters because the case it covers is a machine whose clock is
wrong, and a wrong clock must never be a way to empty a record. The test that
says so is `a_wrong_clock_never_removes_more`, and it was the failing test that
found this.
**Date:** 2026-09-03

### A record is replaced while it is open for appending, and Windows allows it
**Version:** Rust 1.97 `std::fs::rename`, Windows 11 26200; seen 2026-09-03 in
`alo-keeping`
**Behaviour:** shortening a record writes the replacement beside the old file
and renames it over. On Windows that is `MoveFileEx` with
`MOVEFILE_REPLACE_EXISTING`, and replacing a file another handle has open is
the classic way to get *access is denied*. It succeeds here, because `std`
opens files with `FILE_SHARE_DELETE` among the share flags — which is `std`'s
choice rather than a documented guarantee of the platform.
**Our response:** the rename happens with the writer's own append handle still
open, and the handle is **reopened immediately afterwards** — an old handle
goes on writing into a file that is no longer the record, which is a lost entry
rather than an error. Shortening is therefore a method on the writer taking
`&mut self`, so nothing can append during it and nothing else is expected to be
holding the record open. If a filesystem ever refuses the replace, the answer
is to close the handle before renaming and not to copy over the old file in
place: nothing is removed until the replacement is whole on the disk.
**Date:** 2026-09-03

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
**Closed by item 9g on 2026-09-03.** Kept because the shape of the mistake is
worth recognising again, and because the fix cost a public surface change.
**Version:** `alo-capability` at item 9e, observed 2026-09-02.
**Behaviour:** a translated sentence is only as translated as what goes into
its gaps, and two here came from somewhere that had not moved.
`capability.call.missing` — *{verb} needs {argument} — {purpose}* — filled
`{purpose}` from what the verb was declared with, which is the source string
rather than the reader's; the crate that declared the verb had the translation,
and the crate that refuses the call did not, because a `Verb` carried the
declaration and not a key. `capability.answer.lapsed` quotes the approval
sentence, which `alo_capability::Call` rendered at the moment the call was made
and kept as a string. So a German machine could read a German sentence with an
English clause inside it, which is exactly the failure `alo-appearance` closed
for colour names in item 9d.
**Our response while it stood:** the note on each of those two words said so, in
the words a translator needs, so nobody spent an afternoon looking for the
string that would fix it. **What closed it:** item 9g. A verb is declared from
`alo_strings::Word`s, a `Call` carries the key of its sentence and the values
that fill it, and `CallError::Missing` carries the key of the argument's
purpose — so both gaps are looked up with the same vocabulary as the sentence
around them. It was never worked around, because working around it would have
meant a second copy of a declaration, and one string rather than two that agree
is the rule the whole 9-series is built on.
**Date:** 2026-09-02, closed 2026-09-03

### A translated error cannot be a `std::error::Error`
**Version:** Rust 1.x, `std::error::Error: Debug + Display`, met again at item
9f on 2026-09-02 and at item 9h on 2026-09-03.
**Behaviour:** `std::error::Error` requires `Display`, and `Display` takes no
argument but a formatter — so a type that can only say what it is when it is
handed the reader's language cannot implement it. Everything downstream of that
trait goes with it: `?` into a `Box<dyn Error>`, `#[from]`, `anyhow`, and the
`{e}` a programmer writes without thinking. It is the same collision the
deserialiser entry above describes, met from the other side, and item 9f is
where it reached a type in a **public trait's** signature — `ModelRuntime`
returns `RuntimeError`, and third parties implement `ModelRuntime`.
**Our response:** the types a person reads give up `Display` and answer
`said(&Strings)`, and the ones a programmer reads keep it. The line between
them is *who is holding the machine when this appears*: `CatalogueError`
refuses the catalogue this repository ships, `VerbError` refuses a verb
declaration, `alo-shortcuts`' `DefaultsError` refuses a release's own defaults —
all read by whoever is fixing the thing that failed, so all still English and
still `std::error::Error`. What an adapter author gives up is `?` into a boxed
error, and what they get is a refusal their user can read; `RuntimeError`'s own
documentation says so where they will look. Two doctests in `alo-models` had to
drop their `?` for this reason and were re-checked afterwards, because a
`compile_fail` that starts failing on a missing conversion has stopped testing
what it was written for.

Item 9h met it in the place where it costs the most and is still worth paying:
`alo_egress::NotPermitted` is what `Indicator::beginning` returns, so an egress
refusal no longer arrives as an `Error` a caller can `?` into a box. The person
holding the machine when that appears is the owner watching the indicator, so
the refusal gives up `Display` like the rest — and three doctests, one of them
in `alo-record`, dropped their `?`. The `compile_fail` beside them was
re-checked outside the doctest harness and still fails on **E0624, associated
function `new` is private**, which is what it was written to test.
**Date:** 2026-09-03

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

### What a provider's status code means when a *question* fails
**Version:** the OpenAI-compatible convention as documented by its publishers as
of 2026-09-03. **Not observed against any live provider**; `alo-asking`'s tests
drive a stub on a real socket, and checking this against a provider somebody
pays for is owed alongside the rest of the hardware verification.
**Behaviour:** the convention says what the *endpoint* is and says almost
nothing about which status a provider answers with when it will not answer a
question. In particular **404 means the model, not the address**: a provider
that does not offer the model somebody asked for answers 404 on an endpoint that
exists, which is indistinguishable at the protocol level from an address that is
wrong. 400 is used for a request the provider would not accept and 429 for one
it would have accepted later, and neither is a thing the person who asked can
do anything about.
**Our response:** `alo-asking`'s `hosted.rs` maps each status to the sentence a
person is actually told, and the mapping is written down there beside the
reasoning: 404 and 405 become *the model this question needed was not there*,
400 and 422 become *something answered, and not with an answer*, 401 and 403
become *the key was not accepted*, and everything else becomes *it answered
{status}, which is a problem at that end rather than yours*. What is
deliberately **not** done is guessing between "the model is gone" and "the
address is wrong" — both send a person to look at something, and only one of
them is worth their afternoon, so the sentence names what was needed rather than
what to fix.
**Date:** 2026-09-03

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
