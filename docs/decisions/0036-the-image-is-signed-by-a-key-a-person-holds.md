# ADR 0036 — The image is signed by a key a person holds, and published by that person

**Status:** accepted by the owner, 2026-09-15 — option **A**. The owner generated
the key pair with `cosign generate-key-pair` on a machine they control, holds the
private half and its password, and the public half is committed at
`image/signing/alo-os.pub`. Task 1 of `docs/autonomy/v0-5-the-installer-plan.md`
now waits only on the first publish it describes.
**Date:** 2026-09-14, accepted 2026-09-15
**Context:** [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(*our own image, our own registry, our own signing key*),
[ADR 0023](0023-installed-from-the-machine-it-replaces.md) §3 (the boot
environment pulls the image *verifying signatures before writing*),
[ADR 0033](0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
§3 (the registry is `ghcr.io/aloworld-org/alo-os`; until a workflow pushes, the
machine that built the image pushes it, and the digest is pinned in one file a
test reads), and the installer plan's task 1, which asks for the image *signed
with `cosign` and the public key in the repository*.

## The question in one line

**Every machine alo OS is ever installed on will trust whatever one key signs.
Who holds that key, where does it live, and who is allowed to publish?**

## What was true before this decision

Nothing in this repository says. ADR 0011 says the key is *ours*; ADR 0023 says
signatures are verified before a disk is written; the plan says the verifying
half is in the repository. None of them says who generates the private half,
where it is kept, what signs with it, or whether the build loop may publish.

And the task was sent to a worker on 2026-09-14, which found, on the machine
that builds the image:

- podman, and a local build of the image two days old — older than the recipe
  as it now stands, so not the image anybody could pin;
- **no `cosign`, no login to `ghcr.io`, and no GitHub command-line tool** —
  nothing a worker could push with, and nothing it should sign with.

A worker could install `cosign`, generate a key pair, choose its password and
leave both halves on a development disk. The public half would then go into
every installer and every image as the root of trust for the whole product,
and nobody would have decided that it should be. That is the one step this
record exists to stop being taken by default.

## The options

### A. A key pair a person generates and holds; a person signs

The owner generates the pair on their own machine. The public half is committed
at `image/signing/alo-os.pub`; the private half is never in the repository, never
on a build loop's disk and never in a CI secret. It is `cosign`'s own
password-encrypted key file first, and a hardware token (`cosign`'s PIV support)
when there is one to put it on.

Publishing is done by that person, by digest:

1. build the image at a clean, published commit of `main`;
2. push it to `ghcr.io/aloworld-org/alo-os:<version>`, where `<version>` is the
   recipe's `org.opencontainers.image.version`, and keep the digest the push
   reports;
3. sign **that digest**, never the tag, with the private half;
4. verify the signature with `image/signing/alo-os.pub` from a clean state;
5. only then hand the version and the digest to the repository, where they are
   pinned and held to the recipe.

A push whose signature does not verify at step 4 is not a publish, and nothing
is pinned against it.

- **For:** it is ADR 0011 literally. The dependency is on a person and a file,
  not on a company. Nothing automated can publish alo OS, which on a product
  that repartitions somebody's only computer is a feature. Verification needs
  the public key and the registry, and nothing else.
- **Against:** it is manual, and it is one person. A publish waits for them. A
  lost private key means a new key, a new public half and installers released
  after it; a machine installed before still trusts the old one until an update
  says otherwise.

### B. The private key is a GitHub Actions secret; the workflow signs

- **For:** automatic, and the plan's eventual *by a workflow* road becomes one
  step.
- **Against:** anybody who can change a workflow that runs on `main` can sign
  alo OS, and so can the company that stores the secret. And the workflow cannot
  build the image yet — it carries 4.5 GB of weights — so this buys nothing
  today and moves the root of trust off a person's desk for good.

### C. Keyless signing through Sigstore

The workflow (or a person) signs with a short-lived certificate tied to a GitHub
identity, recorded in Sigstore's public transparency log.

- **For:** no private key to lose or steal, and a public log of every signature
  ever made.
- **Against:** there is no public key to put in the repository, which is what
  the plan asks for; trust is anchored in an identity GitHub issues and a
  certificate authority run outside this project, which is ADR 0011's *the
  dependency is on packages, not on a company* reversed; and a boot environment
  that checks the log online makes a request nobody's screen showed, against law
  1, unless it is given an offline bundle — at which point it is a key again.

## Recommendation

**A.** The key is generated and held by the owner; the public half is committed
at `image/signing/alo-os.pub`; signing is by digest and done by a person; **no
agent and no build loop ever holds the private key or publishes**, which is the
same rule `CLAUDE.md` already makes about whose name a commit carries.
Signatures are made without uploading to a public transparency log, so a pull
verifies against the committed key and the registry alone; whether a
transparency log is worth its egress is a question for the update channel, with
its own record.

Two layers, and both are kept: **the digest pinned in the repository** is what
an installer pulls, so a stolen key cannot redirect an installer that has
already been released; **the signature** is what lets the boot environment
refuse bytes the registry changed, before anything is written.

## As the owner accepted it, 2026-09-15

Two decisions the owner made when accepting A, and they sharpen what
*publishes* means above rather than change it:

1. **A machine builds and pushes; only the owner signs.** The build of
   `image/Containerfile` at a published commit of `main`, and its push to
   `ghcr.io/aloworld-org/alo-os` under the owner's GitHub account, are done by a
   machine — the road *What it costs* already names for a workflow, taken before
   there is a workflow. **An unsigned push is a candidate, not a publish:**
   nothing pins it, the boot environment refuses it, and no installer can write
   it to a disk. A release exists only once the owner has signed its digest with
   the private half and the signature verifies against
   `image/signing/alo-os.pub`. The private half still never touches a machine an
   agent runs on, and no agent ever runs the signing command.
2. **A person installing alo OS never sees a key.** The public half travels
   inside the installer and the image, and verifying a signature is something
   the installer does, not a step a person is shown, asked about or able to
   skip. What a person does is download, click, and restart (ADR 0023). A key,
   a password or a signature is the publisher's concern and appears in no
   sentence an installer or the setup a person meets can say.

## What it costs, and what follows

- **Task 1 has a person's half and a repository's half, in that order.** The
  person's half is the five steps above, once. The repository's half — the
  pinned file `alo-image` reads, the check that the recipe's version and the pin
  agree, the registry and the one `bootc install` invocation in
  `docs/booting.md`, and the workflow file with the reason it is not yet the
  road — is a worker's, and it can only be finished once there is a real digest
  to pin. A pin written before a push is a digest nobody can pull.
- **The recipe names its release before anything is built**, so the image that
  is pushed is one that can say which release it is. That landed with this
  record: `image/Containerfile` states `org.opencontainers.image.version`, and
  `crates/alo-image` refuses a recipe that names none, two, or a word that moves.
- **The package on `ghcr.io` has to be public** for an installer to pull it
  without an account, and making it so is the owner's setting on GitHub.
- **When a workflow becomes the road**, under A it builds and pushes a candidate
  and never signs; a person signs the candidate's digest or does not.
- **Machines will need the public half too.** Updates come from the same
  registry (ADR 0023), so the image will carry the key in the container
  signature policy the base's tooling reads. That is the update channel's task,
  not this one, and it is named here so that the key's path is chosen once.
- **Rotation** is a new public half committed beside the old, images signed by
  both for as long as machines trusting only the old one exist, and a record of
  when the old one stops being used.

## Rejected, if A is accepted

- **A key an agent generates.** It would work, and it would make the root of
  trust for every installed machine something nobody decided.
- **B**, for the reasons above; revisited only when a workflow can build the
  image, and then only as *the workflow pushes, a person signs*.
- **C**, for the reasons above; a transparency log may return as an addition to
  a key, never as a replacement for one.

## The private half was exposed, and the owner accepted it, 2026-09-17

While release 0.0.2 was being prepared, the encrypted private half was pasted
into a conversation with the agent preparing it. The agent did not write it to
disk and it is not in this repository — the test below still refuses any private
key under `image/signing/` — but a transcript is stored, and the key's secrecy
from that moment rests on its passphrase alone.

The agent recommended rotating: nothing is installed in the field, 0.0.2 was not
yet signed, and rotating the root of trust costs almost nothing before machines
exist and a great deal afterwards. **The owner declined, and that is recorded
here rather than left as a silence**, because a decision nobody wrote down looks
identical to an oversight when somebody reads this in a year.

What follows from it, unchanged by the decision:

- **The passphrase is never typed where an agent can read it.** Ciphertext and
  passphrase together are the key; separately, neither is.
- **Signing stays the owner's.** The agent prepares the digest and hands over one
  command; the owner runs it; the agent verifies the signature against the
  committed public half before anything is pinned. That was option A's shape and
  it is now also the reason the exposure is survivable.
- **If a machine is ever installed in the field, this stops being cheap.** The
  moment there are machines trusting this key, rotation means every one of them
  refusing updates until it is re-keyed by hand. Revisit before the certified
  laptop is installed, not after.
