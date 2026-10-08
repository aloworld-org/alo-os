# ADR 0097 — One version a person says, and one build identifier

**Status:** **ACCEPTED, 2026-10-08**, by the owner, in three statements as the
document was put to them in pieces: *"We can go with this one alo OS 2026.10"*,
then *"Okay I agree with that let's do that"* on a release being a decision and
most builds not being releases, then *"1: yes"* to the whole of it. It was
refusable in one line at each step and nothing had touched a file before the
last of them.

Settled between the Mac lane and the desktop lane at the owner's instruction —
*"Chat with desktop and let's do our version in a professional and
understandable manner not like this and then when done let's update the version
so as we are professionals"* — and put to the owner rather than decided by us.
**Date:** 2026-10-08
**Proposed by:** the Mac lane (the shape, the `os-release` measurement, the
spec reading, and withdrawing the milestone rename) and the desktop lane (the
punctuation collision, *a release is a decision*, the signing rule, words
rather than a number, refusing to delete the check, the zero-padded month, which
month a release takes its name from, what a candidate's machine says about
itself, the walk that prints the wrong field, the landing order, *why now*, the
measurement owed on tooling nobody here owns, and the pin recording one commit
twice)
**Supersedes:** the **image** half of `ROADMAP.md`'s *Two numbers that look
alike*, for releases from this one onward. The milestone half stands untouched,
and nothing already published is renamed.
**Context:** [ADR 0033](0033-the-image-is-published-to-a-registry-and-pinned-by-digest.md)
(published and pinned by digest), [ADR 0036](0036-only-the-owner-signs-a-release.md)
(only the owner signs), [ADR 0046](0046-the-installer-is-the-owners-to-publish.md)
(the installer is the owner's to publish),
[ADR 0096](0096-a-workflow-may-publish-a-candidate-and-a-candidate-announces-itself.md)
(a candidate announces itself), `ROADMAP.md`, `image/usr/lib/os-release`,
`image/pinned.toml`, `crates/alo-citing/tests/every_version_this_repository_writes.rs`

## The question in one line

**What does alo OS call itself, so that one number answers *which alo OS is
this* and nothing else can be mistaken for it?**

## What is wrong today, measured rather than felt

Three measurements, in the order they were taken on 2026-10-08.

**One. The owner asked what ships next, and the honest answer took four
paragraphs.** Not because the answer is complicated but because the question
has two of them: the next image is the seventh of a series, and the next
milestone is `v0.5`, which is a phase and not a download. A product whose own
builder needs four paragraphs for *what version is next* has a naming fault.

**Two. We wrote a CI check over our own English to keep the two apart.**
`crates/alo-citing/tests/every_version_this_repository_writes.rs` holds every
markdown file in this repository to rules about how to spell a version, because
`v0.5` and the image series are one character apart. That check is good and it
caught a real fault three times in one ADR. **It is also the diagnosis:** when a
scheme needs a robot policing prose to stay legible, the scheme is the fault and
the prose is the symptom.

**Three — and this is the one that makes it concrete rather than stylistic. The
machine already gives three different answers about itself.**
`image/usr/lib/os-release`, as shipped in the current release:

```
NAME="alo OS"
ID=alo
ID_LIKE=fedora
VERSION="0.0.6"
VERSION_ID=42
PRETTY_NAME="alo OS 0.0.6"
CPE_NAME="cpe:/o:aloworld:alo_os:0.0.6"
PLATFORM_ID="platform:f42"
```

- `VERSION` and `PRETTY_NAME` say ours.
- **`VERSION_ID` says `42`** — Fedora's, and that is the field a script reads
  when it asks the machine its version.
- **`CPE_NAME` says ours** — and that is the field a vulnerability scanner
  reads.
- `IMAGE_ID` and `IMAGE_VERSION` are **absent**, and they are the two fields
  the specification wrote for a system like this one.

So a scanner and a script, reading one file, are told different things.

**And the divergence is deliberate, which is sharper than finding it
unnoticed.** `crates/alo-image/tests/the_machine_says_it_is_alo_os.rs` holds
**eight** tests over this file. `the_release_it_says_it_is_matches_the_recipe` ties
`VERSION`, `PRETTY_NAME` and `CPE_NAME` to the release the recipe builds, and
`what_it_is_built_on_is_stated_plainly` asserts `VERSION_ID=42` and
`PLATFORM_ID=platform:f42` **on purpose**, for the reason the file documents:
package tooling reasons about `$releasever`, and engines are configured rather
than patched. Nobody missed this. It is checked, in both directions, and
correct as far as it goes.

**What is actually missing is narrower and more fixable.** Not an oversight
about `VERSION_ID`, which should stay. Two things:

- **Nothing in the file carries our build identifier at all.** `IMAGE_ID` and
  `IMAGE_VERSION` — the pair the specification wrote for a system shipped as
  whole images — are absent, so a tool that wants *which alo OS build is this*
  has no field to read and must either take `VERSION`, which is a product
  version, or `VERSION_ID`, which is Fedora's.
- **The one walk that prints a version prints the Fedora field.** Measured at
  `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs:763`:

```
ExecStart=-/usr/bin/sh -c '. /usr/lib/os-release; echo "os-release: $NAME $VERSION_ID"'
```

  So a walk on a real machine reports `alo OS 42` today, and would go on
  reporting `alo OS 42` after this scheme landed. **The one check that would
  catch a version string going wrong reads the one field that never carries
  ours.** Found by the desktop lane in their own crate; verified here against
  the file, together with a sweep for every other reader in the workspace,
  which found none that reads a version.

## Why now, and not nearer 1.0

`os-release` is a **published contract surface**, and this repository's rule is
that contracts change additively. Adding `IMAGE_ID` and `IMAGE_VERSION` is
additive. Changing what `VERSION`, `PRETTY_NAME` and `CPE_NAME` *contain* is not
a shape break, but it would break anybody parsing `VERSION` as three
dot-separated numbers.

**Measured: nothing in this workspace parses it, and the only reader reads a
different field.** Outside it, nobody holds one of our images but the owner's
own machines. So the change costs nothing today and will not be free once a
third party builds an adapter against the field. That is the difference between
a tidy-up and a thing with a deadline, and it is the argument for doing it in
this release rather than in a later one.

## What shipping systems actually do

Ubuntu `24.04 LTS`. Debian `12 "bookworm"`. Fedora `41`. macOS `26` with build
`25A354`. Windows `11`, version `24H2`, build `26100`.

One invariant across all of them: **exactly one number a person says out loud,
and a separate build identifier that could never be mistaken for it.** And not
one of them shows an internal milestone number to the people who download it.
Ubuntu does not tell you it is in milestone v0.5.

## The decision

**1. The product version is the year and the month: `2026.10`.** It is what the
download page leads with, what the About screen shows first, and what a person
says out loud. A second one inside the same month is `2026.10.1`.

Date-based rather than sequential, for a reason specific to what this product
sells: **a sovereignty product's worst failure is a machine nobody updated**,
and a year-month tells the person standing at it how old their software is
without looking anything up. A sequential number would end the confusion
equally and answer nothing.

**The month is always two digits: `2026.01`, never `2026.1`.** The shape is
written here with its reason, because a shape without one gets tidied by
somebody removing a trailing zero. January written `2026.1` reads as `2026.10`
with a digit lost, and it sorts *before* `2026.2`. Ubuntu writes `24.04` for
exactly this reason.

**Five releases in one month, which is the owner's own question and the case
most likely to arise:** `2026.10`, then `2026.10.1`, `2026.10.2`, `2026.10.3`,
`2026.10.4`. **The first carries no `.0`** — Ubuntu's first is `24.04`, not
`24.04.0` — and the shape in rule 9 allows both forms for that reason.

It is worth saying what that case *means* rather than only how to write it. Six
images went out in nineteen days, about nine a month. Nine builds a month is
almost certainly nine builds and **two** releases, not nine — and the old scheme
made nine releases easy by incrementing a counter. So *am I about to make the
fifth release this month?* is usually answered *no, this is the fifth build and
the second release*. Where five genuinely are releases, because something worth
installing was fixed five times, `2026.10.4` is simply correct and there is
nothing to solve. The numbering carries it; rule 4 is what stops it being
reached for by default.

**The month is the one the release is made in, not the one the build was made
in.** A release decided in November from a build made on 30 October is
`2026.11`. The product version answers *how old is the software I am running*,
which is about when it reached people — and nothing is lost, because the build
identifier carries the build's own date. So `alo OS 2026.11` holding build
`2026-10-30+4799555` is both legible and true. This sentence is here because
two people would otherwise answer it differently, which is the original fault
arriving in a new place.

**2. The build identifier is a date and a short commit: `2026-10-08+4799555`.**
This is what a bug report quotes, what `image/pinned.toml` records beside the
digest, and what support asks for.

**3. Dots are a version; hyphens are a date.** `2026.10` and `2026.10.1` are
versions. `2026-10-08+4799555` is a build. This rule exists because the first
draft of this ADR proposed `2026.10.08+4799555` as the build identifier, which
reads exactly like a product version with a point release — *two numbers that
look alike*, rebuilt with new numbers, in the document written to end them. The
convention already exists in our own candidate tags, which read
`candidate-2026-10-07-57445ab`, and the specification agrees: `BUILD_ID`'s
examples in `os-release(5)` are hyphenated dates.

**4. A release is a decision, and most builds are not releases.** This is the
rule that makes a dated product version honest, and without it the scheme is
worse than what it replaces. Six images went out in nineteen days, each one
called a release; any product version over that cadence labels a moving target,
dated or sequential.

So: **a candidate carries a build identifier and no product version. A release
carries both.** ADR 0096 already built this machinery and it is verified against
the live API — a candidate announces itself and `/releases/latest` excludes it.
What changes is that the product version becomes the name of something fixed.

**5. What is signed is named by build identifier and digest, never by product
version.** *The owner signed `2026.10`* becomes ambiguous the moment two builds
share that version, where today version and build are one string and it cannot.
ADR 0036 does not move — the signature is over the digest and a product version
is metadata — and a request for a signature already quotes the digest, so this
is a tightening rather than a change. A release promoted from a candidate that
was already signed is the same bytes, so the signature travels and nothing is
re-signed.

**6. Earliness is said in words, with the specific gaps, and never smuggled
into the number.** The download page reads *alo OS 2026.10 — a preview*, with
what does not work under it. Two reasons: a number that signals immaturity has
to stop signalling it one day, which makes `1.0` a marketing event rather than
a fact about software; and the current series communicates a *feeling* of
earliness while telling a stranger nothing they can act on. We already do the
honest version of this well — the download page says Windows does not know who
made the installer, and ADR 0095 says a machine answers nothing out of the box
until a person brings weights.

**7. The machine answers with the specification's own fields.**
`os-release(5)`, read in the VM on 2026-10-08, settles which:

```
To summarize: if the image updates are built and shipped as comprehensive
units, IMAGE_ID+IMAGE_VERSION is the best fit.
```

That is exactly what alo OS is. So:

| field | becomes | why |
|---|---|---|
| `VERSION`, `PRETTY_NAME` | the product version | what a person reads |
| `CPE_NAME` | the product version | what a scanner reads |
| `IMAGE_ID` | `alo` | the specification's field for an image shipped whole |
| `IMAGE_VERSION` | the build identifier | the specification's field for telling two of them apart |
| `VERSION_ID`, `PLATFORM_ID` | **unchanged, Fedora's** | package tooling reasons about `$releasever` and the platform id, and engines are configured rather than patched — the reason the file already documents |

**And a candidate's machine says so in the same words the installer did.** This
is where rule 4 stops being a convention and becomes observable: a machine
installed from a candidate still has to answer what it is, and it has no product
version to answer with. The fields cannot carry the last release's — the
installer would say *this is a test build of alo OS and not a release* under
ADR 0096's fifth control, and the machine it installed would then answer
`alo OS 2026.10`, a version it never had. **A machine giving a confident false
answer about itself is the exact fault this ADR was opened on.** So:

```
PRETTY_NAME="alo OS test build 2026-10-08 (not a release)"
VERSION="test build 2026-10-08+4799555"
IMAGE_VERSION="2026-10-08+4799555"
CPE_NAME        omitted — there is no product version to put in it
```

A release differs from the candidate it was promoted from by having a product
version in `VERSION`, `PRETTY_NAME` and `CPE_NAME`, and by keeping **the same**
`IMAGE_VERSION`. That is the signature travelling, visible in the file.

**This is what makes rule 4 testable.** A built image either carries a product
version or announces itself as a test build — never neither, never both — and
that is a test in `crates/alo-image`, which is the Mac lane's to write.

**`BUILD_ID` is deliberately not used**, and the desktop lane's proposal of it
is the one correction that went the other way. The specification defines it as
*"a string uniquely identifying the system image originally used as the
installation base ... `VERSION_ID` would change during incremental system
updates, but `BUILD_ID` would not."* Ours must change every build, which is the
opposite.

**8. Milestones keep their numbers, and gain a rule instead of a rename.**
`v0.01`, `v0.5` and `v1` stay exactly as they are. What replaces the rename is
one rule: **a milestone never appears in anything a customer reads** — not the
download page, not the About screen, not release notes. It is a planning number
and it stays inside the plan.

This reverses the Mac lane's own proposal, on a measurement taken before
writing: the milestone spellings appear in **275** files for `v0.01`, **444**
for `v0.5` and **164** for `v1`, plus **161** `Roadmap:` trailers in the last
four hundred commits — and historical records must keep the old spelling by the
same principle that nothing published is renamed, so a rename leaves both forms
live forever. **The collision is solved by rule 1 alone:** what the owner hit
was `v0.5` against the image series, one character apart, and `v0.5` against
`2026.10` is not confusable at all. Moving the release scheme fixes it at zero
churn.

**The spelling is nevertheless to change, later and as its own change.** The
owner asked on 2026-10-08 whether the `v0.01` form is still needed, and the
honest answer is that the milestone *concept* is load-bearing — an exit gate is
what stops a page of ticked boxes reading as done — while the *spelling* does no
work. `[v0.5]` on a feature line says nothing about what it gates, and
`ROADMAP.md`'s own headings already name all three: *it boots and the agent
acts*, *a person can work on it all day*, *an organisation can buy it*. So
`[boots]`, `[all-day]` and `[organisations]`.

**Measured before recommending, because an earlier rename in this document was
withdrawn on a measurement:** 270 tier lines in `docs/features.md`, plus
`crates/alo-reconciling` — whose `tier.rs` holds the literal strings `- [v0.01]`,
`- [v0.5]` and `- [v1]` and whose tests cross-check the tiers in
`docs/features.md` against `ROADMAP.md` — plus the filename
`every_v0_01_promise_is_reconciled.rs`. **One crate and two documents**, which is
bounded work, and nothing like the 883 scattered prose mentions that killed the
first proposal.

**Deferred rather than refused, and the reason is sequencing rather than cost.**
Accepting rule 1 removed the *ambiguity*; what is left is legibility, so this is
a should and not a must. The owner was offered it first or second and chose
second — *"your recommendation"* — so it lands after the screens-and-desks work
rather than tangling a 270-line mechanical rename through the same window as
feature branches in three lanes. Historical ADRs keep their old spellings when
it happens, by the same principle that nothing published is renamed.

**9. The check is repointed, not deleted.** The first draft of this ADR
proposed removing the citing test's prose rules on the grounds that there would
be nothing left to confuse. That is retiring an instrument because the design
has become unbreakable, which is the shape `docs/misreadings/` exists for.

And it is measurably wrong. `numbers_in` matches **`0.0.N` and nothing else** —
read, not assumed — so the current check would say nothing whatever about
`2026.10.08` used as a product version, about `v2026.10`, about a build
identifier written with dots, or about a milestone written as a number out of
habit. The new rules are also **cheaper** to check than the old ones, because
they are two shapes rather than a set of style rules:

| kind | shape |
|---|---|
| product version | `\d{4}\.\d{2}(\.\d+)?` |
| build identifier | `\d{4}-\d{2}-\d{2}\+[0-9a-f]{7,}` |

The desktop lane takes that rewrite, since `alo-citing` is not the Mac lane's
crate.

**One reading, named and dismissed rather than left for somebody to raise.**
Much of Europe writes dates with dots, so a reader from outside this repository
could take `2026.10.08` for a date. We never write one — that is the whole of
rule 3 — and the shapes above enforce it, so every dotted number in our prose is
a version and every hyphenated one is a date.

## One measurement owed before the first image carries the new fields

`IMAGE_ID` and `IMAGE_VERSION` are read by tooling this project does not own.
alo OS is a bootc/ostree image, and `bootc status`, rpm-ostree and `dnf` all
read `os-release`. **Whether any of them keys off those two fields is not
something to reason about** — *engines are configured, never patched* means a
field an engine reads is configuration, and configuration that changes an
engine's behaviour is measured rather than assumed harmless.

So before the first image ships carrying them: **write a disk, read the fields
back, and confirm `bootc status`, an update and a `dnf` resolution behave as
they do now.** The Mac lane holds the walks for that. If something does key off
`IMAGE_ID`, a walk is where that should be found rather than a customer's
machine — and if nothing does, this ADR can state a measurement instead of a
hope. Raised by the desktop lane.

## The pin records one fact once

`image/pinned.toml` holds, per image:

```
version  = "0.0.6"
digest   = "sha256:fcc732bb…"
revision = "77db91c64e590be259b48a162091425213e28728"
```

A build identifier of `2026-10-08+4799555` would put **the same commit in the
file twice, in two forms** — its short half beside `revision`'s long one — in
the file that says which image is signed. **A field that duplicates another in a
different form is a field that can disagree with it**, and that is the worst
place in this repository for two fields to drift.

So the pin stores each fact once and **renders** the identifier rather than
keeping it:

| field | holds |
|---|---|
| `built` | the build's date, `2026-10-08` |
| `revision` | the full commit, unchanged |
| `digest` | unchanged |
| `version` | the product version — **and only where there is one**, so a candidate's entry has no `version` field at all |

The build identifier is then `{built}+{revision[..7]}`, derived at the point of
use.

**Seven characters is deliberate, and the full revision is beside it.** Git's
own short-hash length grows with a repository's object count, because seven
stops being unique eventually — and this one goes into release notes and bug
reports, where a person reads it. It is assessed as not a practical concern: a
collision needs an improbable number of objects, `revision` sits in the same
file in full, and **the digest is the authority rather than the identifier.**
Written down so that a reader in three years finds a decision rather than a
magic number. Raised by the desktop lane. Cross-checking two stored forms would catch a disagreement; storing one
fact makes the disagreement unrepresentable, which is the better of the two.

Renaming the field from `version` to `build` is itself a small instance of this
ADR's whole subject: a field called `version` holding a build identifier is the
same category error, one layer down. Raised by the desktop lane.

## The order this has to land in

**The citing test's two shapes land before or with the first version string in
the new form.** The check in force today enforces the old scheme, so the first
file that writes a dated product version fails the gate until the rewrite is
there — and accepting this ADR does not change a test. This document is
publishable in the meantime only because its prose was checked against the rules
as they stand, which is what was done.

That is `docs/misreadings/a-replaced-rule-and-the-callers-left-behind.md` read
forwards: not callers left behind by a changed rule, but **a rule left behind by
changed callers.** One sentence here costs nothing and saves a confused morning.

## What this does not change

- **Nothing published is renamed.** Six images shipped, their digests pinned
  and the owner's signatures over them; `image/pinned.toml` keeps its record of
  all of them, and `image/release-notes.md` keeps the tag and digest it was
  published with. Debian did not rename its old releases when its own scheme
  changed either.
- **ADR 0033, ADR 0036 and ADR 0046** in full. A machine builds and pushes;
  only the owner signs; publishing is theirs.
- **ADR 0096's candidate machinery**, which rule 4 relies on rather than
  alters.
- **`VERSION_ID` and `PLATFORM_ID`**, for the reason `os-release`'s own header
  already gives.
- **The milestone numbers**, per rule 8.

## What is not enforced, said plainly

A table rather than a promise, because this document's whole subject is a claim
that outran what checked it.

| rule | what enforces it |
|---|---|
| 1, 2, 3 — the two shapes | the repointed citing test, once written |
| 4 — a release is a decision | **partly checkable, and the test is owed.** A built image must either carry a product version or announce itself as a test build, which `crates/alo-image` can assert; ADR 0096 already makes a candidate announce itself and keeps it out of `/releases/latest`. What nothing checks, and nothing can, is that a human called the right build a release |
| 5 — signed things named by build and digest | **nothing automatic.** ADR 0036's existing practice of quoting the digest |
| 6 — earliness in words | **nothing.** A page review |
| 7 — the machine's own fields | **eight tests already cover this file** and one of them ties `VERSION`, `PRETTY_NAME` and `CPE_NAME` to the recipe. They are **extended**, not written: an assertion over `IMAGE_ID` and `IMAGE_VERSION`, and the walk's serial line changed to print them. This row said *owed and not written* in the first draft, which was false — the claim was corrected by reading the crate rather than by anybody disputing it |
| 8 — no milestone in customer-facing text | **nothing.** The citing test could be extended to the download page and the release notes, and this ADR does not claim it has been |

**Four of the nine rest on people**, and two more rest on tests that are owed
rather than written. That is the honest state of it on the day
it was proposed, and the owed test in row 7 is the one worth building first,
because it is the row where the fault was actually found.

## What accepting it would change

- `image/usr/lib/os-release`: four fields, per rule 7.
- `image/pinned.toml`: the next entry records a build identifier beside its
  digest.
- `ROADMAP.md`: the image half of *Two numbers that look alike* is superseded
  and says so, with one sentence recording that the sixth image was the last of
  the old form.
- `crates/alo-citing`: the two shapes replace the prose rules — the desktop
  lane's work.
- `image/release-notes.md` and the download page: the product version, the
  word *preview*, and the specific gaps.
- `crates/alo-image/tests/the_machine_says_it_is_alo_os.rs`: **extended**, not
  written — an assertion over `IMAGE_ID` and `IMAGE_VERSION`, and the product
  version and build identifier enforced as separate fields. The Mac lane's.
- `crates/alo-image`: a test that a built image either carries a product version
  or announces itself as a test build — the assertion that makes rule 4 real
  rather than a convention. The Mac lane's.
- `image/pinned.toml`: `built` and `revision` per entry, `version` only where
  there is a product version, and the build identifier derived rather than
  stored.
- A walk that reads `IMAGE_ID` and `IMAGE_VERSION` back off a written disk and
  confirms `bootc status`, an update and a `dnf` resolution are unchanged. The
  Mac lane's, and it gates the first image carrying those fields.
- `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs:763`:
  the walk's serial line prints `$VERSION` and `$IMAGE_VERSION` — the two things
  this scheme claims — keeping `$VERSION_ID` beside them, because a walk that
  hides the base makes every later question harder. The desktop lane's, landing
  with the citing test so the two go together.

## Options rejected

**A sequential product version — `alo OS 7`.** Ends the collision equally and
tells a person nothing about how old their machine is, which is the fact that
matters most on this product. Rejected on that, not on aesthetics.

**Renaming the milestones to words.** The measurement in rule 8: 883 file
mentions, 161 commit trailers, both spellings live forever in historical
records, and the collision already solved without it.

**Keeping the current series and writing the rules down harder.** Already tried.
The rules are written down, a CI check enforces them over every markdown file in
the repository, and the fault still reached three paragraphs of one ADR and all
four of `os-release`'s version fields.

**Deleting the citing test.** Rule 9.
