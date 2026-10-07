# ADR 0096 — A workflow may publish a candidate, and a candidate announces itself

**Status:** **ACCEPTED, 2026-10-07**, by the owner, stated directly: *"go
ahead"*, in answer to this document's recommendation put to them in full with
the note that nothing would land until they answered.

Settled between the Mac lane and the third PC at the owner's earlier
instruction — *"chat with desktop and both choose the best one to go with then
we update the documents if needed"* — and put to the owner rather than decided
by us, because ADR 0036 and ADR 0046 place publishing in their hands and two
agents agreeing is not the owner deciding. The recommendation is one paragraph,
below, and was refusable in one line.
**Date:** 2026-10-07
**Proposed by:** the third PC (the need, and three of the six controls) and the
Mac lane (the research, and the reframing of what a draft protects). Neither
lane may decide it: ADR 0036 and ADR 0046 put publishing in the owner's hands.
**Supersedes:** one clause of
[ADR 0046](0046-the-installer-is-signed-by-a-certificate-a-person-holds.md) —
its rejected option *Publishing a Release that is not a draft* — and the
`says_why_it_is_not_yet_the_road` rule in `crates/alo-image/src/workflow.rs`,
which went false on 2026-10-07.
**Context:** [ADR 0036](0036-the-image-is-signed-by-a-key-a-person-holds.md)
(the image's signature), ADR 0046 (the installer's), 
[ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
(which is what made the image fit a runner at all),
`crates/alo-image/src/workflow.rs`,
`crates/alo-image/tests/the_installer_candidate_is_not_a_release.rs`.

## The recommendation, so it can be refused in one line

**Let the candidate workflow publish its build as a pre-release on the public
releases page, and require a candidate to announce itself in six ways — one of
them its own first line on screen.** Signing stays yours and a candidate never
has it.

Refuse it and the third PC plans the metal runs around full releases instead:
every attempt at task 4's walk then costs an image version, your signature, a
tag and a publish, and the walk may take five or ten attempts.

## The question

A candidate build cannot currently be put in front of a person. `release.yml`
runs on a tag, which is right — a tag is the owner's word that a version is a
version. `installer-candidate.yml` uploads an artefact, and an artefact needs a
GitHub session, which is not how anybody downloads an operating system.

That gap has a measured cost. On the testing NUC, the installer from **release
0.0.5** died in the Windows loader with `0xC0000135` for want of
`VCRUNTIME140`, before `main`, printing nothing. The fix — `crt-static` — is on `main` and **has never been tried**,
because the only machine that can try it has nothing to run. A fix nobody can
try is a fix nobody has tried.

## What has already changed, and this document is partly catching up to it

**An unsigned release is already on the public download page.** Measured
2026-10-07: **release 0.0.6** is published, **not a draft, not a pre-release,
marked Latest**, with `alo-installer.zip` and `SHA256SUMS` downloadable by
anyone. Its
notes say *"This download is not signed, and Windows will say so"*, describe the
SmartScreen box, and offer `Get-FileHash` against `SHA256SUMS` — arguing that
check *"does more for you than a signature would."*

The owner decided that deliberately, having been asked plainly first. **Nothing
in `docs/decisions/` records it**, and ADR 0046's clause reads as though it
could not have happened:

> **Publishing a Release that is not a draft.** A Release that exists is a
> Release somebody can download; the draft is what keeps the unsigned build off
> the download page.

The draft did not keep it off the page. The owner looked and acted, which is
what the draft is for. So this document has two jobs and they are one decision.

## Job one: what a draft actually protects

**That publication is a person's act rather than a workflow's.** Not a
signature, and not a look — a look is passive and what happened was an act.

That is narrower than ADR 0046's clause implies, and the narrowing is the whole
decision. Follow it through:

| | who publishes |
|---|---|
| ADR 0046's model | CI builds → draft → **the owner** signs and publishes |
| what this permits | CI builds → **CI** publishes, for a candidate only |

So the real change is not *may a candidate be a pre-release*. It is **may a
workflow publish at all**, and *a candidate may be a pre-release* is the
consequence. The six controls below are therefore not decoration on a decision
already made: **they are the price of letting a workflow publish.**

## Job two: what makes a candidate not a release

**"Unsigned" no longer carries the information, and that is the crux** — found
by the third PC. ADR 0046's clause leaned on unsigned meaning *not for the
download page*, and after 2026-10-07 an unsigned artefact is exactly what is on
the download page. The property has stopped distinguishing anything, so
something must replace it.

What replaces it is not another property of the bytes. It is that **the
candidate says what it is, in six places, one of which is not a web page at
all.**

### The six controls

Each is held by a test with a named constant, not by a habit.

1. **Marked `prerelease`, and never `make_latest`.** GitHub defines
   `/releases/latest` as the most recent **non-prerelease, non-draft** release,
   and drafts and pre-releases **cannot be set as latest**. This control is
   therefore **enforced by the API rather than by us** — it holds whether or
   not our test runs, whether or not somebody edits the workflow, and whether
   or not a future reader understands why. That is a stronger kind of guard
   than anything we assert, and it is the only one of the six with that
   property.
2. **The tag is never `v<pinned version>`** and carries a candidate marker and
   a date. The date makes staleness visible without clicking.
3. **It never signs.** Unchanged from ADR 0036 and ADR 0046. Not weakened by
   this document in any direction.
4. **It is never written into `image/pinned.toml`.** A stronger invariant than
   any distribution has available: there is exactly one pin in this repository,
   so a candidate that is never pinned cannot be resolved to by anything.
5. **The programme announces itself, with the date it was built.** Its first
   line on screen reads, in the person's own language:

   > This is a test build of alo OS from 7 October 2026. It is not a release.

   Compiled in by the candidate workflow the way
   `ALO_INSTALLER_ENVIRONMENT_SHA256` already is, and tested **both ways** — a
   candidate must carry it and a release must not.
6. **Exactly one candidate exists at a time.** Publishing a new one deletes the
   previous, at publish, with no scheduler. Seven days is the stated intent.

### Why control five is the one worth arguing for

Every other control lives on the release page. **The person it protects is not
looking at the release page** — they downloaded a candidate, kept it, and
double-click it next month. Expiry does not reach them: deleting the
pre-release removes the page, not the file on their disk. A filename marker
does not: nobody reads a filename, and a zip gets extracted. The notes marker
does not: that is the page they left.

**And the date in it is what makes expiry reach that person at all.** Somebody
who kept a candidate and runs it in December reads *from 7 October 2026* with
no page to consult and no network. Control six cannot help them; control five
carrying a date can. *That improvement is the third PC's, applying the Mac
lane's own argument for five to the case the Mac lane had said five could not
fix.*

**It informs and deliberately does not refuse.** A test build that stopped
working after N days could bite mid-walk, and the failure would look exactly
like the `0xC0000135` loader failure this whole road exists to eliminate — a
program that prints nothing and does not start. It would also be the installer
deciding for the person, which the fifth law forbids: *every protection is a
default they can change, not a wall.* Telling somebody plainly what they are
holding is a protection that restricts nothing. Refusing to run is a wall.

Control five is the only one that survives the artefact outliving its context,
**which is the characteristic failure of a candidate.** And ADR 0046's own
first sentence says what is at stake: *the program that repartitions somebody's
only computer*. A person who mistakes a candidate for a release does not have a
bad afternoon; they have a repartitioned disk. One sentence and a compile-time
variable is cheap against that.

### Why control six is a property and not a duration

*A candidate that lives forever becomes a release by default.* Every
distribution deletes its dailies; without this we would have reinvented a stale
download page. One-at-a-time was the third PC's answer and it is better than
the duration the Mac lane proposed, because it gives a property rather than a
deadline — **the candidate on that page is the newest one, always** — and needs
nothing to run on time.

## What the professionals do, measured rather than recalled

Verified 2026-10-07 by reading the sources, not from memory.

- **Test builds are public and unauthenticated.** Debian publishes
  `cdimage.debian.org/cdimage/daily-builds/`. Ubuntu's dailies are described in
  Ubuntu's own documentation as *preview images … not fully tested and
  unsuitable for production use*. No sign-in anywhere. **Gating a test build
  behind a session is the thing nobody does**, which is what makes the artefact
  road wrong rather than merely inconvenient.
- **The distinction is structural, never procedural**: carried by location
  (`cdimage` against `releases`, `current` against `pending`), by name
  (`41_Beta`, `_rc3`, `daily`), and by which key signed it.
- **Signatures mean provenance, not endorsement.** Fedora signs Rawhide too,
  with a different key that changes when a release branches. **Nobody uses
  "unsigned" to mean "not a release"** — which is independent confirmation that
  ADR 0046's clause was leaning on the wrong property even before 2026-10-07.
- **The counter-model exists and does not fit us.** Microsoft's Insider builds
  and Apple's betas *do* gate behind enrolment — and sign everything, including
  test builds. So there are two coherent models: **gate the access, or sign the
  manifest.** We cannot gate, because that defeats testing the way a person
  installs; we cannot sign, because ADR 0036 and 0046 reserve it. The
  distribution model is the one available to us, and the six controls are what
  we substitute for the key we will not hold.

### Where we get the shape of the idea and not its strength, said plainly

Debian's daily builds ship `SHA256SUMS` **and `SHA256SUMS.sign`** — a detached
signature over the checksum file, which is what makes a daily provably
Debian's. **We cannot do the second half.** A signing key in CI is ADR 0036's
rejected option under a different name.

So our published checksum says *these are the bytes this workflow produced*. It
is real against a corrupt download and **worth nothing against anybody who can
write to that page.** This document says so rather than citing Debian and
letting a reader infer a parity we do not have — and it is an argument *for*
control six, because an unsigned candidate that lived forever would be the
weakest artefact this project ever published.

*Correction owed to the third PC, who caught the Mac lane about to cite the
comparison without the caveat.*

## And a rule that is already false, replaced on its own terms

`crates/alo-image/src/workflow.rs` holds `image.yml` to
`says_why_it_is_not_yet_the_road`: the workflow existed but was not the road,
because the build did not fit a hosted runner while the image carried 4.87 GiB
of weights.

**That went false on 2026-10-07, before any of this, and for an unrelated
reason.** ADR 0095 took the weights out, the third PC dispatched the workflow,
and it built and pushed image 0.0.6 — the first image this project ever built
on CI rather than on somebody's machine. CI **is** the road. The rule now
reads as a reason not to switch on something that is already on, and a reader
who trusted it would be misled about how the shipped image was made.

It is replaced by what is true: the workflow builds and pushes, it **never
signs**, and it **never pins**. Those two are what ADR 0036 actually reserved;
*not yet the road* was a fact about a runner's disk and never a rule.

*Found by the third PC while dispatching it. Recorded here rather than in a
commit message because a rule that went false deserves the same visibility as
the decision that made it.*

## What is not enforced, said plainly

Three of the six are mechanisms and three are intentions, and a reader deserves
to know which is which.

| | enforced by |
|---|---|
| 1. not `latest` | **GitHub's API.** Holds without us. |
| 6. one candidate at a time | **The publish step.** No scheduler needed. |
| 2, 3, 4 | **Tests.** Hold while the tests exist. |
| 5 | **A test, both ways** — and the sentence itself, once it is on screen. |
| **the seven days** | **nothing.** |

**Nothing enforces the seven days.** If nobody dispatches a build for three
weeks, the single candidate sits on that page for three weeks. It is an
intention about how often a walk happens, not a property of the system, and the
reason there is no scheduled deleter is that its failure mode is deleting
something somebody is mid-walk on.

What makes the seven days *less* load-bearing is control six: the candidate on
the page is always the newest, so a long-lived candidate is stale rather than
wrong, and control five tells whoever runs it how old it is. *Stated at the
third PC's insistence: an ADR claiming an expiry nothing enforces is the kind
of sentence this repository has misreadings entries about.*

## What this does not decide

- **Signing.** Untouched. ADR 0036's five steps and ADR 0046's two remain the
  owner's, and a candidate never has a signature in any form.
- **Whether release 0.0.6 should have been published unsigned.** The owner decided
  that; this document records it and draws the consequence. It does not endorse
  or revisit it.
- **Whether releases should be signed in future.** ADR 0046 stands. If a
  certificate arrives, nothing here is in the way.
- **The installer's own behaviour**, beyond control five's one line.
- **Whether a candidate should ever refuse to run.** Considered and declined,
  with the reasoning above rather than by omission.

## Consequences

- `.github/workflows/installer-candidate.yml` publishes a pre-release. **Mac
  lane.**
- `crates/alo-image/tests/the_installer_candidate_is_not_a_release.rs` is
  **rewritten rather than loosened**: `it_never_publishes` becomes the five
  controls that are checkable from the workflow and the repository. **Mac
  lane.**
- `crates/alo-image/src/workflow.rs` gains readers for the new controls and
  loses `says_why_it_is_not_yet_the_road`. **Mac lane.**
- Control five — the programme's first line — is **the third PC's**, in
  `crates/alo-installer`, and is the one change here that adds a sentence a
  person reads, so `crates/alo-saying/the-vocabulary.txt` is regenerated by
  that lane and by nobody else.
- The third PC can then walk task 4 on the NUC without spending a release per
  attempt, which is the whole point.

## What would make this wrong

A candidate reaching `image/pinned.toml`, or `/releases/latest`, or a machine
whose owner believed it was a release. Five of the six controls are tests and
can be deleted by somebody who finds them inconvenient; the first cannot,
because GitHub enforces it. **If this decision is ever revisited, the question
to ask is not whether the controls are still written down — it is whether a
person could still download that file, run it, and not be told.**
