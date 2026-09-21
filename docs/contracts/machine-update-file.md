# Contract — the update a person approved

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

`docs/features.md` v0.5 promises **atomic updates with rollback** and **updates
that never interrupt**. `alo-keeping-up` decides what an update is and the one
instruction the base may be given; the privileged broker carries a person's
approval across to a unit that may run it. This is what crosses, and what its
digest is taken of, so the side that asks and the side that carries out cannot
disagree. [ADR 0053](../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
is the decision, and `machine-proxy-file.md` is the same pattern for the proxy.

## Why there is a file at all

The broker's door takes no text (`agent-verbs.md`, *The privileged broker*): a
verb's argument is thirty-two bytes. An update is two builds, which is
sixty-four. So the two builds are written into a file the person owns and the
verb's argument is the SHA-256 of exactly those bytes — ADR 0049 §3, the road
the proxy already takes.

## `updates.apply`

| | |
|---|---|
| Path | `/run/alo-broker/wanted/update.json` (`alo_brokerd::THE_WANTED_UPDATE`) |
| Directory | `/run/alo-broker/wanted`, `0770` in the person's group — made by the broker at start-up |
| File | a plain file, the person's, no longer than 1 KiB |
| Writer | whatever puts the choice in front of the person: the turn redeeming an approval, or Settings carrying out a choice of their own |
| Argument | the SHA-256 of the file's bytes, as sixty-four lowercase hexadecimal characters |

One JSON object — `alo_brokerd::AnUpdate` as `serde_json` writes it — and a
newline:

```json
{"from":"sha256:1111111111111111111111111111111111111111111111111111111111111111","to":"sha256:2222222222222222222222222222222222222222222222222222222222222222"}
```

| Field | |
|---|---|
| `from` | the build the machine was running when the person was told, whole, `sha256:` and sixty-four lowercase hexadecimal characters |
| `to` | the build they approved changing to, in the same form |

**Written one way.** The fields in that order, no space anywhere, one trailing
newline. The broker reads the file back and refuses it unless it comes back
byte for byte (`alo_brokerd::AnUpdate::read`) — a file that can be spelt two
ways is a file whose digest stands for less than it appears to.

**Where the update comes from is not in it, and never will be.** The repository
is the one the booted build came from, read from the base's own status by the
unit (`alo_brokerd::TheMachineNow`). There is nowhere on this road for a
request to name a registry.

**There is no `when` in it either.** An update approved this way always waits
for a restart the person makes: `alo_keeping_up::Staging::approved` takes no
choice about when and the instruction it writes can never carry `--apply`.

The broker carries the verb out only when:

1. the handed-over file is a plain file owned by the person, opened without
   following a link, and no longer than 1 KiB;
2. its bytes digest to the approved identity;
3. it reads back as exactly those two builds, written exactly that way;
4. the unit `alo-applying-an-update.service` runs and succeeds.

In every other case nothing is staged and the broker's record says
`not-carried`. The handed-over file is removed whichever way the act went.

## `updates.roll-back`

**Nothing is handed over.** Going back has no argument the machine did not
already have — the build before, the build running, and the update that going
back would set aside are all in the base's status and the machine's record. So
the identity is the digest of bytes **both sides write for themselves**:

```json
{"from":"sha256:2222…","to":"sha256:1111…","sets_aside":null}
```

| Field | |
|---|---|
| `from` | the build being left, whole |
| `to` | the build returned to, whole |
| `sets_aside` | the build waiting for the next restart that going back sets aside, whole — or `null` when there is none |

`alo_brokerd::GoingBackApproved::offered` writes them from an
`alo_keeping_up::GoingBack`, in that field order, with one trailing newline, and
`identity()` is their SHA-256.

The unit decides `GoingBack` again from the machine in front of it and refuses
unless what it decided digests to the identity the person approved. So an
update that started waiting since the offer, or a build before that changed, is
a machine that is not the one the person was shown — and nothing is set.

## Between the broker and the unit

| | |
|---|---|
| Directory | `/run/alo-broker/approved`, `0700`, root's — made by the broker at start-up (`alo_brokerd::for_the_unit::THE_FOLDER`) |
| `updates.apply` | `/run/alo-broker/approved/update.json`, `0600` root's: **the same bytes**, after the broker checked them |
| `updates.roll-back` | `/run/alo-broker/approved/going-back.identity`, `0600` root's: the approved identity, sixty-four hexadecimal characters and a newline |

**Copied rather than pointed at, and this is the point of the second folder.**
Between the broker digesting what a person handed over and a privileged program
reading it, the person's folder can be written again by anything running as
that person. A unit reading from there would act on bytes nobody approved.

Both files are removed whichever way the act went: a file in `/run` naming a
build to install is an instruction waiting for somebody to start a unit by hand.

## The two units

| | |
|---|---|
| `alo-applying-an-update.service` | `ExecStart=/usr/libexec/alo-applying-an-update`, no arguments |
| `alo-going-back.service` | `ExecStart=/usr/libexec/alo-going-back`, no arguments |

Each is `Type=oneshot`, runs as root, carries its capabilities **named one per
line**, and carries neither `CAP_SYS_BOOT` nor anything an update has no
business with. Neither can be enabled, so nothing but the broker starts either.
The broker asks systemd to start one over the system bus (`StartUnit`, mode
`fail`), waits for it, and reads its result — three methods and no other
(`alo_brokerd::units`). The broker itself still holds **no capability**.

## What is not here

- **How long the asking side waits.** `alo_broker::asking::waiting_for` answers
  it: an hour for the two update verbs, three minutes for every other.
- **Whether a build is genuine.** The instruction always carries
  `--enforce-container-sigpolicy`; the machine's signature policy answers at
  the moment the base is told (ADR 0036).
- **What a person reads.** The broker answers a word from a closed list; the
  surface that asked says what it means, in their own language.
