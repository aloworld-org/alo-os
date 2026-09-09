# An organisation's rule, off a disk

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`)
- Contributor: Claude Code
- Task: Load an organisation's inference policy into the daemon
- Status: **the path from a file to a refusal is complete and tested.** One
  expression is not covered by an automated test, and this report says which.

## What was actually missing

ADR 0016 gives the bound to the organisation and the choice to the person, and
`alo-agentd` has carried it since `6cff37c`: it words a refusal, names that an
administrator set the rule, asks the rule **before** the keyring, and writes the
refused question into the record. All of that was reachable only from a literal
`TheBound` written in a test.

**Every real machine was `TheBound::Nobodys`**, because
`docs/contracts/machine-description.md` had no key for a policy. The mechanism
was finished and unreachable. This is the key, and nothing more than the key: no
enrolment, no identity, no reporting, no server. One optional section in a file
that was already read once at startup, from the disk it is on.

## The contract first

`[questions]`, with `may-go` — `"anywhere"`, `"in-the-building"`,
`"this-machine-only"`, `"in-a-region"` — and a `region` that goes only with the
last of those.

**It requires `format = 2`, and that is the one place the file's additive rule
does not reach.** Every other key an older service does not know leaves it
describing the same machine. A policy it ignored would leave it sending an
organisation's questions wherever the person chose, which is not the same machine
under any reading. So a description carrying a bound says `2`; a service reading
only `1` refuses it and does not start — *a managed machine too old to understand
its policy does not run unmanaged*. The other direction is refused too:
`format = 1` with a `[questions]` in it is a file claiming an older service could
have read it correctly when it could not.

A description **without** the section means the same thing under either number,
so `1` is still read (`ALSO_READ`) and `2` without a policy is fine. That matters
when an organisation **takes** a bound off: deleting the section is the whole
edit, and nothing is renumbered back.

## Absent is absent, and it is the common case

No section is `TheBound::Nobodys`: ADR 0016's *absent* rather than permissive.
Both permit the same things; what they do not share is somebody to name in a
refusal. Every description that exists today is this, unchanged, still `1`.

## Present and not holding is refused, never read as unrestricted

An unknown `may-go`; `in-a-region` with no region, or a region of nothing but
spaces; a region written beside a `may-go` that would ignore it; a `[questions]`
with no `may-go`. Each stops the service.

This is the failure the whole section is shaped around: **an organisation that
wrote a policy and got no policy, with nothing on the machine saying so.** A key
somebody believes is bounding their questions and is not is worse than no key.

The unknown-`may-go` case is answered before the region beside it, so
`may-go = "in-a-regoin"` with a region under it reports the typo rather than
sending somebody to the wrong line.

## Attribution is a fact about the file, not a reading of the value

The requirement was that a restrictive policy must never, on its own, earn an
administrator. It does not, and what decides instead is **who owns the
description**.

`crate::trusting` already permitted exactly two owners and refused every other —
root, because an organisation's configuration system writes into `/etc` as root
(ADR 0004), and the person alo-agentd runs as, because a personal machine is
theirs. It then threw that answer away. It now carries it out as
`WhoDescribedIt`, and `crate::describing` turns `[questions]` into
`AnOrganisations` or `ThePersons` on the strength of it.

So the identical four words produce the identical bound and a different origin.
`the_same_bound_the_person_wrote_names_no_administrator` is that test, and
`ThePersons` is now reachable for the first time — it existed before only because
the distinction had to be unrepresentable to get wrong.

## The person's own choices are untouched

A bound refuses a choice; it can never replace one. Their model and provider stay
in their own settings and nothing here writes them.
`a_description_with_no_policy_leaves_the_persons_own_choice_alone` drives the
daemon path with an unbounded description and a real keyring holding a synthetic
key, and watches the provider they chose being reached and the one they did not
being left alone.

## The end-to-end acceptance

`a_policy_on_a_disk_bounds_a_question_and_reaches_the_record`, and every step is
the production one:

- a description written the way the contract writes it, in a temporary directory,
  with a `[questions]` in it;
- `Described::at` reads it off the disk under the file's own ownership and
  permission rules;
- the bound it produces is handed to `Questions` exactly as `crate::starting`
  hands it — no literal anywhere in the test;
- the question goes through `put_to_a_model` against a **file-backed** record;
- and the record is then **read back off the disk** and its wording compared with
  what the agent was told.

Two things it proves are *not* reached. Both providers are real listeners and
neither is connected to. The keyring is real, running, and holds no key for this
provider — so a daemon that looked would answer with a sentence this crate can
name, and the sentence is the rule's instead.

The disk-level half is in `what_a_machine_says_about_itself.rs`: a bound read off
a real file, the older shape still read, a policy in the older shape refused, and
an unreadable bound stopping the machine.

## Two mistakes, both caught by tests failing

**The suite runs as root.** `a_file_of_our_own_is_read` asserted `ThePerson` and
failed, which is the environment telling the truth: under `cargo test` in this
WSL distribution the file the test writes is root-owned. Pinning the expectation
to one value would have been a test about this machine. It now derives the
expectation from who really owns the file, and `may_be_believed` asks both cases
directly where neither depends on the environment.

**A test about a newer format stopped being one.** `format = 2` used to be the
example of *a shape this service does not read*, and now it is one it does. Both
occurrences are taken from `THE_FORMAT` now, so the next shape cannot quietly
turn either into a test about a number that reads perfectly well.

## Verified, and how

Mutation, not just green:

- making a person-owned policy `AnOrganisations` fails
  `the_same_bound_the_person_wrote_names_no_administrator` — one test, precisely
  the attribution property;
- reading an unknown `may-go` as `Anywhere` instead of refusing fails
  `a_bound_this_alo_os_cannot_read_is_refused_rather_than_ignored` and
  `an_unknown_bound_is_answered_before_the_region_written_beside_it`.

The workspace suite is green: **134 test binaries, no failures**, including the
five kernel-boundary tests in `a_question_is_bounded_by_the_kernel`.

## What this does **not** cover

**The hand-off in `starting::until_stopped`.** One expression —
`described.questions().clone()` — is in the same untestable class as
`Questions::of_this_process` reading the process environment. This crate sets no
environment variables in tests by policy (`std::env::set_var` is `unsafe` in this
edition and races every other test in the binary), so every question test drives
`of_a_session`. Everything either side of that expression is tested; the
expression itself is read rather than exercised, and saying so is more use than
a test that asserted the line back to itself.

**`ThePersons` off a real disk.** Proved in the reader's own tests, where the
owner is a parameter. On a disk it depends on who runs the suite, so the
disk-level tests assert that the attribution *follows the file's owner* rather
than that it is any particular value — which is the property, and under a suite
running as root it exercises an organisation's.

**A root-owned description on a machine where the tests are not root.** Same old
limit as the third-owner rule: making a file owned by somebody else takes a
privilege the tests do not have everywhere they run.

**The image's reader.** `crates/alo-image` reads the shipped description for the
handful of things the image is answerable for and knows only `format = 1`. The
image installs an unmanaged machine, so that is correct today; a policy in a
built image would need that reader taught the section first, and the contract now
says so.

**Nothing on this PC's real configuration was touched.** Every test writes its
own description in a temporary directory. No service was started, stopped or
reconfigured, and no organisation's configuration exists on this machine to
alter.

## One shared mount, once, with nothing in flight

`/sys/fs/bpf` was gone — WSL had restarted three minutes earlier and it forgets
the mount across a restart. Both workstreams were checked and idle first: no
`cargo`, no `rustc`, no `alo-*` process anywhere in the distribution, and no loop
running in either checkout. It was then mounted once and verified (`stat -f`
reports `bpf_fs`), which is the coordinated handoff the supervisor's own refusal
message asks a person for. **The supervisor still refuses to mount it itself**,
and that is deliberate: a checkout that remounts whenever it wants to publish is
one that changes another worker's ground mid-test.

## Proposed integration updates

**CHANGELOG.md** — an organisation that manages a machine can now state where its
staff's questions may be answered, in `/etc/alo/agentd.toml`, and a question that
breaks the rule is refused before any credential is read or any connection made,
in words that name the rule and say an administrator set it.

**ROADMAP.md** — ADR 0016's v0.01 requirement is met for the daemon.

**docs/autonomy/QUEUE.md** — the machine description carries `[questions]` at
`format = 2`; `1` is still read; the daemon's policy path is reachable from a
real file for the first time.

**docs/autonomy/STATE.md** — `alo-agentd` loads an organisation's bound from its
description, attributes it by the file's owner, and refuses a description whose
policy it cannot read rather than running unmanaged.
