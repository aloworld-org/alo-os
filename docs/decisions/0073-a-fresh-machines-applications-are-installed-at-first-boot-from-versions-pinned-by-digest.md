# ADR 0073 — A fresh machine's applications are installed at first boot, from versions pinned by digest

**Status:** proposed, 2026-09-28.

Written by the development PC's lane, which owns `image/`. `docs/autonomy/v0-5-evidence.md`
says of five separate promises that **the installing** is still owed — the file
manager, the archives, the text editor, the image viewer, the terminal and the
browser — and gives one reason for all of them:

> Nothing in `image/` installs `shipped.toml`'s list: the installer plan is to name
> the fresh machine's applications to the image from that file, and that has not
> been done.

Measured before writing this: `crates/alo-software/shipped.toml` names seven
applications, `image/` mentions flatpak nowhere, no task in
`docs/autonomy/v0-5-the-installer-plan.md` covers it, and no decision record
does either. The applications plan hands the file over — *"the installer plan
reads that same file rather than a copy in `image/`"* — to a task that was never
written.

So this asks rather than changes, because two of the three questions below are
not a lane's to answer.

## The decision in one line

**A fresh machine installs its applications at first boot, not at image build,
and each one is pinned by the digest of the release `shipped.toml` names** — so
that a version in that file means the same thing a digest in `image/pinned.toml`
means, and a machine that cannot reach Flathub says so rather than arriving
without a file manager and no explanation.

## What forced the question: `shipped.toml` pins by tag

`image/pinned.toml` pins the operating system by content:

    digest = "sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1"

and the sentence beside it says why — *the digest is what an installer already
released will pull, so a stolen key cannot redirect it.*

`shipped.toml` pins by name and number:

    identifier = "org.mozilla.firefox"
    source = "flathub"
    version = "156.0"

Those are not the same kind of pin. `156.0` is a tag on a remote that can move
it; the digest cannot be moved by anybody. **The repository therefore holds its
own operating system to a standard it does not hold the browser to**, and nothing
said so until somebody tried to install the browser.

That is the whole of what makes this a decision rather than a Containerfile line.

## Build time or first boot

**First boot**, and the reasons are not about convenience.

- **The image is pinned by digest and a flatpak installed into it would not be.**
  Everything in `image/` is content-addressed today — the base, the office
  engine, the model runtime, the weights. Installing seven applications at build
  time from a remote that serves tags would put the one unpinnable thing inside
  the one artefact whose whole claim is that it is pinned.
- **Size.** Firefox alone is over half a gigabyte unpacked. The image is
  8.3 GiB installed and the installer refuses a disk under 24 GB; seven
  applications would push both numbers for every machine, including the ones
  whose owner wanted none of them.
- **A person may remove them.** Task 1 of the software plan already says each of
  these goes through the ordinary two steps, *so a person updates and removes any
  of them, the browser included.* An application baked into an ostree image is
  not one a person can remove; it comes back at the next deployment.
- **And it is what the crate already does.** `alo-software`'s `installing` and
  `install` are written, tested, arrive with no grants and refuse a place that
  does not check signatures. A build-time path would be a second road to the same
  place, and the second one would be the one nobody tested.

**What first boot costs, said plainly:** a machine with no network arrives with no
applications. That is the next section.

## What a machine that cannot reach Flathub says

It says so, by name, and it is still a machine.

- The desktop comes up, the agent runs, the model answers — none of those wait on
  an application.
- Each application `shipped.toml` names is reported as **not installed, and why**,
  through `alo-software`'s existing refusals rather than a new sentence. A place
  that cannot be reached is already `NotDone`, and a person reads it.
- It is **not** a failed install. A machine that has never had a file manager and
  says so is in a different state from one whose install broke, and the two must
  not share a word.

The thing this forbids is the state the ledger would otherwise have produced: a
machine with no file manager, no error, and a promise in `docs/features.md` saying
it has one.

## What is asked of the owner, because it is not ours

1. **Whether the digest is required, or preferred.** Pinning a flatpak by digest
   is possible — a Flathub release has one — but it means this repository carries
   seven more digests that a person must update when a version moves, and the
   update is manual because nothing here watches Flathub. The alternative is to
   keep the tag and say in `shipped.toml` that it is weaker than the image's pin.
   **Recommendation: the digest**, because the sentence beside `pinned.toml`
   applies word for word to an application as much as to an operating system.
2. **Whether a machine may be sold that arrived with nothing.** If first boot
   fails, the machine is usable and incomplete. The alternative is an image that
   carries them and a person who cannot remove them.
   **Recommendation: usable and incomplete**, said in words.

## What this does not decide

- **Which applications.** That is task 2 of the software plan, done on
  2026-09-15, and `shipped.toml` is its answer.
- **How an application is sandboxed or granted anything.** It arrives reachable by
  nothing, which `alo-software` already holds as a fact rather than a shape.
- **The terminal's own question.** The software plan notes the terminal is a
  person's own in a way the others are not; nothing here changes that.

## Rejected

- **Install at build time from a pinned digest.** It solves the pin and none of
  the rest: the size still lands on every machine, and a person still cannot
  remove what an ostree deployment restores.
- **Install nothing and let a person choose.** It is a fresh machine with no way
  to open a file, and `docs/features.md` promises the opposite — *a text editor
  and an image viewer, so a fresh machine is not helpless.*
- **A first-boot unit that retries for ever.** A machine that is still trying is
  a machine that never says what happened. It reports, once, and a person asks
  again when they have a network.
