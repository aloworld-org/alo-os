# v0.5 — access and language: everybody can use it, in the language they asked in

**Workstream:** four `ROADMAP.md` v0.5 lines that are one subject — *Access: screen
reader, magnifier, high contrast, keyboard-only operation of everything*;
*Accessibility: EN 301 549 conformance on the shell*; ★ *The agent answers in the
language it was asked in*; and the half of *Language* that is not yet `alo-strings`
(regional formats and timezones per language). They belong together because they
answer one question: **can this person use this machine, whoever they are and
whatever language they think in.** `docs/features.md` sets the bar: *the AT-SPI
tree the agent uses is the one a screen reader uses; EN 301 549 conformance is the
same work, not extra work.*
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**Crates this plan owns:** new `crates/alo-access` (the accessibility settings and
what each one changes), new `crates/alo-conforming` (EN 301 549's clauses that apply
to a shell, each held to the evidence that meets it), new `crates/alo-formats`
(regional formats and timezones per language), and **the answering-language clause
of `crates/alo-instructing`** — the words a turn shows a model that say which
language to answer in, and nothing else in that crate (its plan finished on
2026-09-15). **It reads and never edits** `alo-strings` and `alo-saying` (the 24
languages and every sentence), `alo-appearance` (high contrast and text size are
appearance values it decides from), `alo-keyboards` (the desktop plan's — sticky and
slow keys act on its input), `alo-turn` and `alo-driving` (lane A's turn and the
measured grades), `alo-models`, and `alo-shortcuts`. **Nothing in
`crates/alo-shell`**: exposing the shell's accessibility tree, the magnifier and
keyboard-only focus are drawn by the shell plan's later tasks, from what this plan
decides and holds.

**What this plan may not do:** tick anything *on the machine*; write a screen reader,
a speech synthesiser or a magnifier engine (Orca, a speech engine and AT-SPI are
rented, configured and never patched, ADR 0011); publish a conformance report — that
is v1, and this plan only builds what the report will rest on; or run, download or
grade any model larger than the small one the catalogue already recommends (the
owner's rule, 2026-09-15: *test on the small model; once it works, larger models are
not run*). Before writing the next task, `git pull` and read the plan as published.

## Tasks

### 1. What each accessibility setting changes

**Status:** ready. **Depends on:** nothing.

- **Acceptance:** `alo-access` holds a closed list of settings — screen reader on,
  magnifier on and its factor, high contrast, larger text, reduced motion, sticky
  keys, slow keys, bounce keys, keyboard-only focus always visible — each with **what
  it changes, named as a value another crate reads** (`alo-appearance`'s palette and
  text scale, the keyboard crate's key filter) and a test per setting that the value
  changes; **high contrast is a palette of its own that meets WCAG AAA contrast for
  text, measured in a test over every pair the shell draws**, and terracotta keeps its
  meaning with its mark and word (ADR 0010); every setting can be turned on **from the
  sign-in screen and during setup, before any account exists** — a person who needs a
  screen reader to set the machine up must not need one to find the setting; and the
  settings are kept by this crate (ADR 0038), with the sign-in's copy machine-wide.
- **Constraint:** nothing draws. No setting is an *accessibility mode* that switches
  off features; each changes one thing.

### 2. The screen reader and the tree it reads

**Status:** ready. **Depends on:** 1.

The agent reads applications through AT-SPI (`docs/contracts/app-adapters.md`'s
fallback). A screen reader reads the same tree. If the tree is good enough for one it
is good enough for the other; if it is not, both fail.

- **Acceptance:** `alo-access` decides, as data the shell will expose, the
  accessibility role, name and state of **every control the shell draws today** —
  sign-in, dock, status area, egress indicator, approval surface, record window,
  settings — read from the shell's own list of surfaces rather than retyped, with a
  test that fails when a surface exists with no entry; the approval surface is read
  **as the sentence the turn wrote, then its two answers, and nothing preselected**
  (ADR 0001), held by a test; the egress and in-use indicators are announced when they
  change, once; and the rented screen reader starts from the setting with the rented
  speech engine speaking the person's language, with a test per language naming the
  voice or naming that none exists.
- **Constraint:** no speech or reading logic of our own. Where the rented speech
  engine has no voice for a language, that is stated per language, not hidden.

### 3. Keyboard-only operation of everything

**Status:** blocked — on `v0-5-hands-on-the-desktop-plan.md` task 6, whose
`alo-keyboards` is the input sticky, slow and bounce keys act on. **Depends on:** 1.

- **Acceptance:** every action the shell offers has a keyboard road, held by a test
  that walks the shell's list of actions and fails on any reachable only by pointer;
  focus is always visible and never trapped — Escape leaves every surface; the order
  focus moves in is the order a sighted reader reads; sticky, slow and bounce keys act
  as EN 301 549 and the platform conventions define them, with a test each; and the
  agent overlay's one key is not the only road to the agent — a keyboard-only person
  reaches it by Tab as well.
- **Constraint:** nothing draws; the shell honours what this decides.

### 4. EN 301 549, clause by clause

**Status:** ready. **Depends on:** 1, 2, 3.

`docs/features.md` v1: *procurement asks for the report, not the intention.* v0.5
builds what the report will be generated from.

- **Acceptance:** `alo-conforming` lists **every clause of EN 301 549 (clauses 5 and
  11, and the parts of 9 that apply to a native shell's text) that applies to alo OS's
  shell**, each with its number, its requirement in one sentence, and one of: *met,
  by* a named test in this workspace; *not yet, because* a named task in a named plan;
  *not applicable, because* a reason — **no clause may be met by a sentence**, held by
  a test that every *met* names a test that exists and passes; and the clause list is
  checked against the standard's version it names, so a revision is a visible change.
- **Constraint:** no report is published and no conformance is claimed. The file is
  evidence for a person to write the v1 report from.

### 5. The agent answers in the language it was asked in

**Status:** **Done, 2026-09-16.** **Depends on:** nothing.

★ *Being able to say "wo ist die Rechnung von Northstar?" and get an answer is the
thing a cloud assistant does badly for smaller languages.*

- **Acceptance:** the words a turn shows a model (`alo-instructing`) say to answer in
  the language of the question — decided from the question itself, not from the
  shell's language, because a person with an English shell may ask in German — and
  the verbs, which are not words a person reads, stay as they are; the language of a
  question is detected **on the machine** without any network call, with a test; the
  sentences alo OS itself says around an answer (*answered on this machine*, refusals)
  are in the shell's language as always; and **the small model the catalogue
  recommends is measured asking in each of the 24 languages**, the result recorded per
  language as *answers in it*, *answers in another language* or *does not answer*,
  with those numbers in the report — **no larger model is run to improve them**
  (the owner's rule), and a language the small model answers poorly is stated as that,
  not hidden.
- **Constraint:** nothing is translated by a service. No answer is machine-translated
  after the model answers, because a translation the person did not ask for is a
  second author of the answer.

### 6. Regional formats and timezones per language

**Status:** ready. **Depends on:** nothing.

- **Acceptance:** `alo-formats` gives each of the 24 languages its default regional
  formats — dates, times, numbers, currency, first day of the week, paper size — **from
  rented CLDR data rather than a table of our own**, and a person changes any of them
  separately from the language (a Portuguese speaker in Belgium); the timezone comes
  from what the person chose, and following the network's time zone is off unless they
  turn it on, because it is a network lookup; every date alo OS shows goes through this
  crate, held by a test that reads the shipped source for a date formatted any other
  way; and the choice is kept by this crate (ADR 0038).
- **Constraint:** no CLDR data is edited. Maltese and Irish get the same completeness
  test as German.

### 7. Every sentence, and the walk with the screen off

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5, 6.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — turn on the screen reader at sign-in, sign in by
  keyboard alone, open the agent, ask a question in Greek, answer an approval, read the
  record — produces the exact sequence of spoken and shown text, recorded as a table and
  held by one test; no sentence names Orca, AT-SPI, a speech engine or CLDR.
- **Constraint:** nothing here re-decides what the sentences describe.
