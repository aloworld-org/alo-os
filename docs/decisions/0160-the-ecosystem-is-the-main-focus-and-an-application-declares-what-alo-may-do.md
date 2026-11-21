# ADR 0160 — The ecosystem is the main focus, and an application declares what alo may do in a signed file inside it

**Status:** accepted, 2026-11-21, for the owner's five answers below, given that
day in a question dialog and relayed by the coordinator. Everything this ADR
says beyond those answers is this lane's reading of them, and is marked as such
under *What is not decided here*.
**Date:** 2026-11-21

## What it changes for the person

Today alo can drive any installed application only through what the
application shows on its accessibility tree, and through one adapter compiled
into alo OS (GNOME Text Editor). After this decision is built, **an application
made by anybody, in any language, can tell alo what it can do** — "export this
drawing as PDF", not only "press the button called Export" — and a person
asking alo gets that road. alo still refuses anything that would let it run
code nobody chose, and every change still waits for the person's approval.

For a developer it means: they do not change alo OS, and they do not have to
write Rust. They ship a signed description file inside their application's own
package, register their signing key with alo once, and alo checks the file.

## The owner's answers

Each is the question asked and the option the owner chose, in the owner's
words.

1. Asked whether to unpark the ecosystem, which was parked on 2026-11-12 so
   that features already built would be finished and verified first:
   **"Make the ecosystem the main focus."**
2. Asked which languages outside developers may build in: **"Any language."**
3. Asked how an application declares what it can do: **"A signed file inside
   the app."** That is, the existing adapter contract
   (`docs/contracts/app-adapters.md`), carried as a file in the application's
   own package, signed, which alo checks and refuses when it is unsafe: no
   script and no "run anything".
4. Asked which application is the first pilot: **"An open-source app
   adapter"**, for example LibreOffice or GIMP, which already have automation of
   their own.
5. Asked who may publish and on what terms: **"Registered makers, free apps
   first."** Each maker registers a signing key, alo reviews each listing,
   applications from Flathub stay, and applications are free at first. Paid
   applications come later, after legal advice.

And on making open-source applications alo-native, the owner's direction of the
same day: **adapters first, with no change to the application; contributions
upstream where an application lacks automation; and forks only where alo needs
deep changes, as with VSCodium.**

## What each answer means against what is already written

**1. The focus.** The 2026-11-12 park was recorded in this lane's notes, never
in this repository, so nothing here is reversed; what changes is the order of
work. The first step of the ecosystem is still finishing something that exists:
the one shipped adapter's verbs are carried out for the terminal
(`alo-app`) and **not offered to alo** —
`crates/alo-turn/src/machine.rs`, *the_verbs_a_whole_machine_offers*: "An
adapter's own verbs — `text_editor.open_document` — are not on it". Until that
is true for one adapter, an ecosystem of adapters is an ecosystem nobody can
reach.

**2. Any language.** What alo checks is the **declaration and the wire** — the
verbs, their typed arguments, the mechanism and the grants — never the code of
the application. So the application's language is not alo's business. This does
not change *One language: Rust* in `CLAUDE.md`, which binds the code in this
repository: the loader, the checker and the conformance suite a developer runs
are alo OS's own and stay Rust.

**3. A signed file inside the application.** This is the road the contract
already pointed at — `crates/alo-adapters/src/adapter.rs`: "Reading one from a
signed file at run time is the v1 *adapter allowlist* and *SDK* lines, and
needs words that arrive at run time" — and the trust rule it already states —
`app-adapters.md`, *Distribution and trust*: "adapters are signed … An unsigned
adapter can be permitted explicitly for development and never by default." It
is **additive** to the contract (*Contracts outlive code*): an adapter compiled
into alo OS still loads, and a file adapter is loaded by the same `load` and
refused by every refusal it carries, including ADR 0001 §6's "The model never
authors code that executes." A file that declares a script argument, an
`exec`, or mechanism `synthetic` is refused with its sentence, exactly as a
constant that declared one is refused today.

**4. The pilot is an open-source application's adapter**, written by alo, with
no change to the application. Which application is chosen by reading each
candidate's real automation surface, and the adapter reaches only enumerated
calls with typed arguments: an application whose only useful road is a script
or a macro string (GIMP's Script-Fu batch interface, LibreOffice's command-line
dispatch of `macro:` addresses) is not a pilot on that road, because law 2
forbids it whatever the application calls it.

**5. Registered makers.** This generalises ADR 0158, which pinned one maker's
key and said "Any maker but storytold. A second maker is a second key file and
a second fingerprint, reviewed as this one is." There will now be others, on
those terms: a maker's key is pinned in this repository and never fetched, each
listing is reviewed, and a bundle or an adapter file not signed by its maker's
pinned key is refused. Flathub stays a source as it is today. It does **not**
create an alo-run Flatpak repository: ADR 0092 and the Non-goal *software is
installed as sandboxed Flatpaks from repositories other people run* still
disagree about that, and this decision does not need one. ADR 0005's *no format
of our own* stands: applications are still Flatpaks.

**Free first** means the market sells nothing. A maker may be a company and an
application may be proprietary (the market already lists and labels both);
what waits for legal advice is alo taking payment for one.

**Open-source applications** follow the order the owner gave: an adapter over
the application's existing automation, then a contribution upstream where that
automation is missing, and a fork only where alo needs deep changes. A fork is
an exception and is decided per application, as VSCodium's was.

## What is not decided here

Each of these is this lane's reading or an open question, and is put to the
owner or settled in its own change before the part it governs is built.

- **The file's format and where it lives in the package.** A path such as
  `/app/share/alo/adapter.toml` fits Flatpak, and is not decided. The contract
  gains a section for it before the first line of the loader is written,
  because a third party builds against its first version.
- **Words that arrive at run time.** A verb's words are `alo_strings::Word`s
  known when alo OS is built. A file's words are not, and how they reach a
  person in their own language, and what happens when a maker ships only
  English, is unanswered.
- **How a maker registers a key**: through a pull request that pins it, as
  ADR 0158 did for storytold, or something else; and what "alo reviews each
  listing" checks, written down so a maker can know before submitting.
- **Law 9 for a maker's verbs.** Every new adapter changes what the model is
  offered. Whether the many-tries check is run by the maker or by alo at
  review is not decided.
- **Which open-source application is the pilot.** Researched in its own report;
  chosen by the owner.
- **Paid applications**, and the money question ADR 0092 already records as
  undecided.
- **Whether alo's own applications are bound to Rust.** `alo-workplace`'s mail
  and meet clients are TypeScript, and meet's transcriber is Python; answer 2
  is about outside developers.

## The order of work

Each step is a pull request with its own tests, and none starts before its
lines are in `docs/features.md`:

1. alo is offered the shipped Text Editor adapter's verbs when a person asks.
2. An adapter declared in a signed file inside an application's package loads
   at run time, refused exactly as a compiled one is.
3. A developer checks their file without building alo OS (the conformance
   suite's first part), and then against their running application.
4. The pilot: an open-source application's adapter, with no change to it.
5. A second registered maker, and a developer guide with a sample written from
   the public documents alone.
