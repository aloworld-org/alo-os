# ADR 0088 — A machine's grants belong to a person, and the move there happens once

**Status:** **accepted, 2026-10-04**, by the development PC under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision. The shape below was settled against two other lanes, whose objections
are credited where they changed it.

**Why this is a record and not a task.** The design was first written into
`docs/autonomy/accounts-and-session-entry-plan.md` task 18, where it ran to a
hundred and seventy-six lines. `CLAUDE.md` says *settled decisions live in
`docs/decisions/`* so that somebody reads the ADR before proposing an
alternative — and a decision buried in a task is one the next person
re-litigates because they never found it. **It changes a published contract and
binds two other lanes, which is an ADR's whole job.**

## The question

`docs/features.md` promises *Multi-user on one machine, with per-person grants
and no shared agent memory*. `alo_remembering::THE_GRANTS` is
`/var/lib/alo/grants.toml` — **one literal path with no person in it**. The file
is owner-protected, and `evidence-a-person-can-work-on-it-all-day.md` states the
consequence exactly: *one file whose owner is checked is not one file per
person — the second account to sign in meets the first one's grants.*

## The obvious implementation is the bug

The first shape proposed was *a person with no file of their own reads the
machine's*, as a migration-free fallback. **It reproduces the fault it was
meant to fix**, and the owner check cannot bound it:

```rust
// crates/alo-remembering/src/believing.rs:136
if owner != 0 && owner != us {
```

A **root-owned file is the believed case**, not the refused one — and
`crates/alo-changing/src/changing.rs` names `THE_GRANTS` as *the file the daemon
re-reads*, the daemon being root. So every account would inherit the same
grants, permanently, with the check passing for exactly the right reason.

**ADR 0001 §3 settles it beyond bounding:**

> A grant comes from a **deliberate act**: a folder chosen in a picker, or the
> document offered at invocation. … There is no grant to `/`, and **there is no
> grant that outlives the reason it was made.**

**An inherited grant is nobody's deliberate act**, and it outlives the reason it
was made the moment it answers for a second person. A fallback is therefore
forbidden rather than merely unbounded. *The applications lane found the
unboundedness; the reading of ADR 0001 that makes it forbidden was this lane's,
and the tense in the task's own words — a machine that **had** one person — was
theirs.*

## The decision

**One file per person, named by uid, reached by a single move.**

- **`/var/lib/alo/grants-<uid>.toml`.** A person with no file of their own has
  **no grants**, which is the only answer ADR 0001 permits.
- **The move runs once**, for the one person a machine had, and the old path
  **stops being read** rather than standing as a fallback.
- **A machine with more than one account at that moment refuses** rather than
  guessing whose the single file was, and writes `/var/lib/alo/grants-not-moved`.
- **The bytes are never parsed** — read believed, written whole.

### Why the uid, on two independent grounds

1. `believing.rs` believes a file belonging to *root or to the login reading it*,
   asked of the open file as a **uid**. Keyed by uid, **the path and the owner
   check compare the same number.**
2. `crates/alo-shell/src/settings_places.rs` records why grants were
   machine-scoped at all: *a login with no folder … has no places for the
   person's own sections at all. **The grants and pairings are the machine's
   rather than the person's folder's, so they are still read.*** A person with
   no home directory still has grants. The other per-person path on this
   machine, `/var/lib/alo/undo/<name>/…`, keys by **name** and would have taken
   that away silently. *This argument came from the applications lane.*

### Why a flat name rather than a folder

The image already makes `/var/lib/alo` — `0700 alo alo` in
`image/usr/lib/tmpfiles.d/alo.conf` — and this crate **refuses a folder that is
not there rather than making one**. A `grants/` subdirectory would need a line
in `image/`, which is another lane's file, for a name that buys nothing.

### Why the bytes are passed through unparsed

`written.rs` writes **the lowest format that holds what is written**, so a
machine with no application's grant writes exactly what it wrote before format
2 — and a rolled-back update must still read it. A move that parsed and
re-serialised could raise the format silently and break that. Reading believed
and writing whole preserves the file byte for byte **by never touching its
content**.

### Why the refusal is written down and not only logged

The refusal itself is a `NotRemembered`, in English, read out of a service log —
which `alo-remembering`'s own `Cargo.toml` says is what its refusals are for,
and why the crate has no vocabulary and no `alo-strings` dependency. But the
move runs **before anybody has signed in**, so that log line is gone by the time
a person looks. The marker keeps the same fact where whatever shows a person
their grants can still find it.

**Its absence is the safe reading.** A machine that never had anything to move
has no marker, and so does a machine whose grants moved correctly — so no
failure to write one can invent a refusal.

## What this costs, in other lanes

- **`granted.nothing-granted` becomes false.** It asserts *Nothing is granted
  right now. **No agent and no application has been granted anything on this
  machine**, and there is nothing here to revoke* — a positive claim about the
  machine's history, whose own note enumerates the states it covers: *the state
  every machine starts in, and the state after the last grant is revoked or
  expires.* **A refused move is a fourth state, which that sentence denies.**
  So what is owed is a **correction to a published key under ADR 0068**, not a
  new word beside it, and it is **release-coupled**: it must land with or before
  the first release in which a refusal can fire. `alo-granted` is the
  applications lane's, which has taken it. *That lane found this.*
- **`settings_places.rs`'s header becomes half true** — pairings stay the
  machine's, grants become a person's by uid though not by folder. The same
  lane's, same coupling.
- **`alo-agentd.service` hardcodes uid 1000 in six places**, including
  `WantedBy=user@1000.service` in `[Install]`, so **a second person's session
  manager pulls up no agent daemon at all** and per-person grants are correct
  but inert for them. 52 files name that unit. `image/` and `alo-image` belong
  to the installer plan by charter line 150. **This is the clause that makes the
  promise work, and it is in neither of the lanes that can currently act.**

## What this does not decide

- **How the daemon calls the move.** It belongs where `alo-agentd/src/main.rs`
  reads `THE_GRANTS`, needs the machine's logins, and `alo-agentd` has no
  dependency on `alo-accounts` today. `alo-remembering` keeps a list and decides
  nothing about it — the same reason there is no `Deserialize` for a grant
  there — so the logins are handed in rather than read.
- **No shared agent memory**, the promise's third clause. 29 of `alo-agentd`'s
  82 source files name a persistent path and every one is under `/var/lib/alo`
  or `/run/alo`, so the question is answerable from the paths — but it is its
  own reading and this record does not pretend to have taken it.

## Consequences

- `crates/alo-remembering/src/whose.rs` is the file side of this, and it is
  **correct code with no production caller** until the daemon calls it. Named
  rather than left to be discovered.
- A test that could not fail was found while building it:
  `a_format_1_file_is_still_format_1_afterwards` passed with the move replaced
  by a parse-and-reserialise, because a fixture written by `kept` round-trips to
  identical bytes. **A claim is only tested by a fixture whose writer would not
  produce it** — the replacement uses a file carrying a comment, which
  `written.rs` never emits.

## This is one end of a larger question, and the other end is open

The installer lane raised it and the measurement is theirs, confirmed here on
main at `17fa85c1`. Two long-lived processes on this machine answer *whose is
it* in opposite ways, and **neither answer survives a second person**:

```
alo-compositor.service   User=root   WantedBy=multi-user.target
alo-agentd.service       User=alo    WantedBy=user@1000.service
```

The compositor is the machine's — root, started at boot, taking its card from
the login seat through libseat, with no `seatd` on the image at all. The agent
is one person's, and that person is a number written into the unit six times.
**A grants file keyed by uid is the same question answered for a file**, and it
is the only one of the three this record decides.

So this is deliberately **not** the ADR that settles what on an alo machine
belongs to a person and what belongs to the machine. That one wants the seat
owner and the per-person daemon together rather than two records written from
opposite ends, it reaches `image/` and `alo-shell`, and the seat-owner half is
with the owner. **Naming it here is what stops this record being read as
though grants were an isolated case.**

## Measured while building the caller, 2026-10-05

**The decision above stands unchanged.** One file per person, keyed by uid, one
move, no fallback, bytes unparsed. What follows corrects three sentences that
supported it and names a blocker this record missed. Each was measured on main
while writing the caller it deliberately left open, and none of them reopens the
choice.

**1. The agent does not run as root.** This record says *the daemon being root*
while arguing that a root-owned file is the believed case. The unit says
`User=alo`, and `image/usr/lib/sysusers.d/alo.conf` says `u alo 1000`, so **the
agent runs as the person** — `image/usr/lib/tmpfiles.d/alo.conf`'s own comment
says so too: *alo-agentd — which runs as the person, holding nothing*. The
conclusion survives on the half that is true: `believing.rs` believes a
**root-owned** file for any reader, whoever the daemon is, so a fallback would
still have handed one person's grants to the next. The reasoning was right about
the file and wrong about the process.

**2. The refusing case cannot arise on this image, and the folder is why.** This
record says a machine with more than one account has no safe answer because
*nothing on disk says which* person the single file belonged to. The folder says
it: `/var/lib/alo` is **`0700 alo alo`**, so no login but that person — or root
— could ever have written the file being moved. Measured with real uids and real
modes rather than reasoned about. `Moved::CouldNotTellWhose` stays reachable and
stays right for a machine whose folder is not that, which is why the caller
passes a list rather than a number.

**3. The logins cannot come from `alo-accounts`.** This record guesses the
caller *needs the machine's logins* from that crate. **Nothing in this
repository writes `/etc/alo/accounts.toml`** — every call of
`alo_accounts::kept` and of `Accounts::created` is inside a test, and
`crates/alo-image` checks the image ships no store. So that store is empty on
every real machine, and a move told *no accounts* would refuse and leave the one
person who has grants unable to read them. The caller uses the machine
description's `[logins].person` instead: the file this process has already read,
already checked itself against, and the number `crates/alo-image` holds the
unit's six mentions to.

**4. A blocker this record missed, and it is in the same file as the one it
named.** This record chose a flat `grants-<uid>.toml` over a `grants/`
subdirectory because *a new directory would need a line in `image/`, which is
another lane's file, for a name that buys nothing.* The flat name needs a line
in that same file anyway:

```
/var/lib/alo at 0700 alo alo   uid 1001 reading grants-1001.toml: REFUSED
/var/lib/alo at 0711           uid 1001 reading grants-1001.toml: READ IT
                               uid 1001 reading grants-1000.toml: REFUSED
```

**At the mode the image ships, a second person cannot read a grants file of
their own** — the refusal is on traversing the folder, before the file's own
mode is consulted. At `0711` they read theirs and are still refused the other
person's, which is this decision's intent exactly. So the directory mode is a
third thing owed by `image/`, beside the agent unit's six uids, and it is **one
character**. `image/` is the installer plan's by charter line 150; this is a
measurement sent rather than a change made.

### What this means for the promise

`docs/features.md`'s *Multi-user on one machine, with per-person grants and no
shared agent memory* is **not finished by the caller landing**, and the caller
does not claim it. What works now is the machine that exists: one person, whose
grants move to their own file and are read from it by both daemons. A second
person on one machine needs three things in `image/` — the unit's uids, the
folder mode, and something that creates an account at all — and none of the
three is this lane's.
