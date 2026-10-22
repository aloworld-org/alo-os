# ADR 0122 — The workplace ships as separate applications, and *projects* is the layer above them

**Status:** **accepted, 2026-10-17, by the owner**, in these words: *"so we
shall split and we shall have alo-mails, alo-meets etc."* Proposed the same
day, after the owner asked whether the workplace should be added *"as different
applications not as a workplace"*.

**Date:** 2026-10-17
**Context:** `aloworld-org/alo-workplace` (`products/mail`, `products/meet`,
`products/sites`, `platform/`, `apps/desktop`);
[ADR 0001](0001-the-capability-model.md) §2 (no ambient authority) and §6
(applications are reached through adapters);
[ADR 0002](0002-the-shell-is-native.md) (*the workspace is one application
running on that shell, like any other*); `docs/features.md`'s starred promise
*Projects instead of applications*; `crates/alo-software/catalogue.toml` (the
decided list of what a person can install).

## The decision in one line

**Mail, meet and sites ship as separate applications** — each installed,
updated, granted and opened on its own — and **the project view is a layer
above them**, not the thing that packages them together.

## What was already true, and is the reason this is small

The workplace repository is **not** one program. It is already:

```
products/   mail  meet  sites
platform/   alo-ai  alo-auth-mail  alo-identity  alo-pdf  alo-sieve  alo-store
apps/       desktop
```

Three products over one shared platform. The only thing that is one is
`apps/desktop`, the shell that hosts them. So this decision changes **how they
are delivered**, and not how they are written.

It is also **Rust**, which settles a question asked the same day: our own
applications are native already, and the choice between a web wrapper and a
desktop-native application does not arise for them. It arises only for
`alo-workplace`'s web surface, which `alo-engine` serves.

## Why separate

- **A grant is per application** (ADR 0001 §2). Mail wanting a person's
  contacts and meet wanting their camera are two questions with two answers.
  One binary asks for the union of everything its parts ever need, and keeps it
  for as long as it is installed.
- **They release on their own clocks.** Mail can reach a person while meet is
  still being built. One workplace is one all-or-nothing gate, and the launch
  list's sixth item — *install applications and work with them* — is answered
  better by three applications somebody can install than by one they cannot.
- **A crash is contained.** A meeting that falls over should not take somebody's
  mail with it.
- **It is what this system is.** `alo-shell` draws a canvas with windows on it.
  A single surface holding six products inside itself is a second window
  manager running inside the first, and the person's own
  *put aside*, *more apps* and window controls stop reaching the thing they are
  actually using.

## What this does **not** change

**`docs/features.md`'s promise stands, unweakened:** *Projects instead of
applications — open the client's website and its mail, meetings, tasks, design,
code and the agent's memory of it are one place, not six windows.*

That promise is about **how a person navigates** — by client, by project —
rather than about how many binaries there are. A project view opens *this
client's mail* and *this client's meeting* as windows on the canvas, and the
agent's one project memory is `platform/alo-ai`, which they already share.

**Packaging them together to keep that promise would be solving a navigation
problem with a build decision**, and paying for it in grants, crash isolation
and release cadence for as long as the product exists. The promise is kept
above, where it was made.

## The name is *workplace*, and *workspace* is two other things

Settled the same day, by counting. **`workspace` already means two things in
this repository**, and either is more likely than the product when a reader
meets the word:

- Cargo's crate set — around 1,378 uses: `[workspace]`, `--workspace`,
  `cargo test --workspace`;
- **the desktop canvas** — `docs/design/palette.toml`'s *"the workspace canvas
  — the desktop a person's windows sit on"*, `alo_appearance::Role::BgCanvas`,
  and the design document `the-canvas-as-a-workspace.md`.

`workplace` means one thing in all 99 of its uses: this product. So *the
workplace runs on the workspace* is a sentence with two different subjects, and
**the product is the workplace** everywhere from here. `CLAUDE.md` is corrected
in the same change, and its *one language* rule keeps the old wording visible
beside the new one so a reader can see what changed rather than only what is.

## What follows

- **The products' own names are `alo-mails` and `alo-meets`.** Asked on
  2026-10-17, the owner answered in their own words: *"Mails our own, alo
  mails."* The owner's spelling is therefore the one that stands and the
  repository's `mail` and `meet` are the ones that give: plural, hyphenated,
  carrying the system's name. `sites` has not been named by the owner and
  keeps the repository's spelling until it is asked for, which is the same
  rule applied to a question that has not been put rather than a different
  one.
- **And *mails our own* settles something larger than a spelling.** It means
  alo's mail client is a thing this company builds, not a third-party client
  packaged with the machine. Any plan to ship an existing mail application as
  the machine's mail — Thunderbird was the obvious candidate and was under
  consideration on 2026-10-17 — is closed by that answer. It does not stop a
  person installing one: the market is everybody's catalogue and refuses
  nothing on the grounds of who wrote it. It decides what alo OS *ships*.
- **They reach a machine the way anything does.** Each becomes an application
  with an identifier, a source, a release and a licence — the four fields
  `alo_software::Catalogue` refuses an entry without. Whether alo's own
  applications sit in the same catalogue as everybody else's, or are shipped
  with the machine like the seven in `shipped.toml`, is a separate decision and
  is not taken here.
- **`CLAUDE.md` is wrong about this and is corrected in the same change.** Its
  standing rule reads *"The workspace above is TypeScript and lives in another
  repository."* The workplace is Rust. The rule it belongs to — *one language:
  Rust* — is unaffected and reads better without the false clause.

## Alternatives rejected

**One workplace application, as today.** Rejected for the four reasons above,
and because the thing it buys — a single place a person navigates — is bought
by the project view instead, which is where the promise was made.

**One application per *project* rather than per product.** A window per client
rather than per kind of work. Rejected because a person who wants to read all
their mail would have no way to, and because grants would then be per client
and per capability, which multiplies rather than clarifies.

## Amended, 2026-10-22 — `sites` is out of scope, and two remain

**The owner, asked what alo OS should finish before going to market:** *"we can
first remove the sites."*

So **mail and meet stand; `sites` does not.** The decision above is otherwise
unchanged: they ship as separate applications, each installed, updated, granted
and opened on its own, and the project view is the layer above them.

**Out of scope, not abandoned.** This is read as *not now* rather than *not
ever*, because that is the reading that loses nothing: a product taken out of a
plan can be put back, and one deleted from a decision takes its reasoning with
it. If the owner meant it is gone for good, say so and this paragraph becomes one
line.

**Why it was the right one to drop, and the ADR had already half-said it.** *What
follows* above records that the owner named `alo-mails` and `alo-meets`
themselves — *"Mails our own, alo mails"* — and that **`sites` had not been named
by them at all**. A product nobody has chosen a name for is the one furthest from
being built, and it was the only one of the three in that position.

**What this does not change.** `products/sites` stays in the workplace
repository; nothing here deletes anything there. What changes is that alo OS
makes no promise to deliver it, so it is not owed an identifier, a source, a
release or a licence, and `crates/alo-software` has nothing to add for it.

### What is still owed for the two that remain, and it is not small

Neither mail nor meet is reachable on a machine today. Measured 2026-10-22:
**neither is named in `crates/alo-software/shipped.toml`, nor in
`catalogue.toml`, nor anywhere in `image/`.** A person on alo OS cannot get mail
at all, five days after this decision was accepted.

The question this ADR left open is the blocker and is still open: *whether alo's
own applications sit in the same catalogue as everybody else's, or are shipped
with the machine like the seven in `shipped.toml`, is a separate decision and is
not taken here.* Under it sits a harder one that is nobody's in this repository:
**the catalogue refuses an entry without an identifier, a source, a release and a
licence**, and the other eight entries have all four because Flathub publishes
them. Nothing publishes `alo-mails`. Until something does, neither road can be
taken whichever the owner picks.
