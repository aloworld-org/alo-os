# Contract — what a machine keeps so that an undo can put something back

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the folder a machine keeps a person's files in as an agent found them
and as it left them, so that *undo what the agent did* has something to rewind
from. [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) chose the base's
own read-only `btrfs` snapshots for that and said where they live — *outside
every grant, under `/var/lib/alo`* — and named who takes them and who removes
them. It did not say how they are laid out, because until 2026-09-21 nothing
had to read them. Something does now, and this is the layout.

**Two processes meet in this folder and neither may guess at the other.**
`crates/alo-turn` takes a bracket, running **as the person**, at the moment a
changing turn begins and again when it ends. `crates/alo-letting-go` removes an
expired one, running **as root on a timer**, because removing a read-only
snapshot needs `CAP_SYS_ADMIN` and taking one does not (`docs/quirks.md`). That
asymmetry is the whole reason this is a written contract rather than a shared
constant.

**Since 2026-09-22 a third reads it**, and it is the same crate as the second
under another unit: `alo-forgetting.service`, which a person's own act starts,
removes **everything** one person's machine was keeping for them when they ask
(ADR 0045 point 5, `docs/contracts/asked-to-forget-folder.md`). Everything below
about what a reader may and may not do holds for it unchanged.

**Nothing an agent can reach is in it.** There is no verb over this folder and
there is not going to be one (ADR 0045's seventh term): an agent that can
forget an undo can erase the evidence of what it did.

## Where it is

```
/var/lib/alo-undo
```

**It was `/var/lib/alo/undo` until 2026-10-17**, inside a directory that is
`0700` and the **person's** — which meant a person could rename the whole folder
away, and a second person could not traverse into it at all. Both were measured.
[ADR 0120](../decisions/0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md)
is the decision, and it carries the readbacks. The folder is now root's own
directory under `/var/lib`, where root makes it holding no capability at all.

`0711 root:root`. Outside every grant — there is no grant to `/` and none to
this, and a person's agent cannot name a path under it because no verb takes a
path at all.

**It said `0700 root:root` until 2026-10-16**, and that was wrong rather than
stricter. A bracket is taken by the person's own session — this document says so
two sections down, about who writes `theirs.json` — and a session cannot
traverse into a folder with no search bit for it, whatever the snapshot itself
costs. Nothing had hit it because nothing takes a bracket yet.
[ADR 0045](../decisions/0045-what-undoing-rewinds-to.md) term 8 is the
amendment, with what was measured.

`0711` and not `0755`: every person may **traverse** to their own directory and
nobody but root may **list** the folder. Listing it would tell one person who
else uses this machine, which is nobody's business and which nothing needs.

## The shape

```
/var/lib/alo-undo/                 0711 root:root
  ada/                             0700 ada — one directory per person
    theirs.json                    where that person's settings folder is
    4f1c8a…/                       one directory per kept changing turn
      kept.json                    when it ran, and what it did in their words
      before/                      a read-only snapshot, as the turn found the home
      after/                       a read-only snapshot, as the turn left it
```

A directory directly under the folder is one person's. A directory directly
under that is one kept changing turn.

**It said `0700 ada:ada` until 2026-10-17, and the group half is now not set at
all.** The owner is set and the group is left as it was, which on this layout is
`root` — so a person's directory reads `ada root` to `ls -l`. The mode is `0700`,
so the group bits grant nobody anything either way, and this is a change to what
the document claims rather than to what anybody may reach.

Three measurements, in the order they bear on it:

- **It was achievable.** `image/usr/lib/sysusers.d/alo.conf` declares the person
  as `u alo 1000` with no group number, and `systemd-sysusers --root` on that one
  line writes `alo:x:1000:1000` to `passwd` and a group `alo:x:1000:` — so on
  this image a person's primary group *is* their account number, and
  `chown(uid, uid)` would have made the old line exactly true. The amendment is
  not *we could not*.
- **It does not hold off the image.** The same call was refused with `EPERM` on
  the machine this was written on, where account 501's primary group is not 501:
  a process may not give a file to a group it is not in. So setting the group to
  the account number would run one way on a certified machine and another way in
  a gate, which is the one difference worth avoiding here.
- **Reading the real group costs a dependency.** The primary group is a field of
  the user database, and
  [ADR 0119](../decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
  chose a separate component for this *because it is small*. A user-database call
  in the one program on the machine that may give a directory away, in order to
  write a number into bits that grant nothing, is the wrong trade.

`crates/alo-giving-a-folder` is what makes this folder, and
`tests/the_contract_describes_where_an_undo_keeps.rs` reads the lines above
rather than agreeing with another crate's constant.

**Who makes what.** The folder and each person's directory inside it are made by
a **privileged step at the moment a person first signs in** — the session cannot
make a directory in a root-owned folder, and image build time does not know who
the people will be. Everything below a person's directory is made by **that
person's own session**, which is what takes the bracket and needs no capability
to do it.

**One of those is built and one is not, measured 2026-10-17.**

**The folder and each person's directory are made** by
`crates/alo-giving-a-folder`, through `alo-giving-a-folder@<account>.service`,
which holds `CAP_CHOWN` and no other capability
([ADR 0119](../decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)).
The unit asks systemd for the directory with `StateDirectory=alo-undo` at mode
`0711`, and the program puts the mode and the owner back on every call, so a
second call for the same person repairs rather than refuses. **Nothing starts
that unit yet**: the two callers
[ADR 0118](../decisions/0118-more-than-one-person-on-a-machine.md) names — the
first account's road and the later route for a person who is not the first — are
not built, so on a machine today the folder is still absent at runtime even
though the thing that makes it exists.

**Nothing takes a bracket** — `alo-turn` is where ADR 0045 puts it and names no
snapshot, unchanged since 2026-10-16. So the sections below about what a bracket
holds remain the design and not a description of a running machine, and they are
marked as such here rather than leaving a reader to discover that the directories
their code is about have never been written to. What *is* built is everything that
reads the folder: the window, the person's surface, and the privileged remover.

**So everything below a person's directory is that person's own writing, and the
remover reads it as root.** That is the whole reason for the three rules that
follow — the link that is never followed, the name that is never parsed, and the
settings folder whose owner is checked. Under the `0700 root:root` this document
used to claim, all three would have been guarding against nobody. **Nothing else is read**: a file where a
directory is expected is stepped over, and so is a link — a link here would be
a way to point the one privileged remover on the machine at somebody else's
subvolume, so nothing under this folder is ever followed.

**Both directory names are opaque.** Nothing reads them, parses them or orders
by them. A person's directory is named after them for whoever has to look at
the machine; a kept turn's name is whatever the process that took the bracket
chose. Everything a reader needs is in the two files, and the reason is the
failure the other shape invites: a name is read by everything and checked by
nothing, so the first reader that parses a moment out of one differently is a
machine removing the wrong snapshot.

## `theirs.json` — where this person's settings are

```json
{"format":1,"settings":"/var/home/ada/.config/alo"}
```

- `format` is `1`, and a file saying anything else is refused **whole and
  first**, before its other keys are judged, so a file a later alo OS wrote is
  not reported as a missing field.
- `settings` is that person's own folder as
  `docs/contracts/person-settings.md` defines it — `$XDG_CONFIG_HOME/alo`, or
  `$HOME/.config/alo` where that says nothing — **absolute**, and a relative
  path is refused.
- **And it is owned by that person**, or it is refused. This file is written by
  the person's own session, and a root unit on a timer then reads `undo.toml`
  from wherever it says; without this, one person could make root follow an
  absolute path of their choosing. The owner it is checked against is the uid
  that owns that person's own directory in this folder, which the shape above
  makes authoritative — so the check needs no guess about where homes live and
  no list of the people on the machine.

  *Added 2026-10-16 with the mode above, and for the same reason: the two
  belong together. A folder the person can write and a path root follows without
  checking is a privilege boundary with nothing on it.*

**Why it exists.** How far back an undo reaches is the person's one setting, in
`undo.toml` in that folder (ADR 0045's first term). `$XDG_CONFIG_HOME` is a
variable of the person's *session*, which a root unit firing off a timer has no
way to know. Guessing `$HOME/.config` would quietly give the shipped window to
exactly the people who had moved theirs — the ones who had bothered to decide.
So the session that takes the bracket, which knows, writes it down, and the
unit reads the person's own file at the path the person's own session named.

It is written when a person's first bracket is taken and replaced whenever the
path changes. It carries **no window** and no copy of any setting: a second
copy of a value is a value that can disagree with itself.

## `kept.json` — when this turn ran, and what it did

```json
{"format":1,"taken":1760000000,"did":"move March.pdf into Invoices"}
```

- `format` is `1`, judged first, as above.
- `taken` is the moment the turn ran, in **whole seconds since the epoch**. It
  is the only thing that orders kept turns, and how many days ago a turn was is
  counted **down** from it — a turn made six days and twenty-three hours ago is
  six days ago, and inside a window of seven.
- `did` is **the sentence the person approved at the time**, copied in, and it
  is refused when it is empty.

**Why the sentence is here.** When the machine lets go of a snapshot, the record
names the turn that lost its undo *in the person's own words* rather than as a
number (ADR 0045's second term). The unit that removes it runs long after the
turn and the person's own record is not its to read, so the process that had
those words writes them down beside what they are about. It is the same *a
copy, not a pointer* the record keeps for its own reason: the record file is
shortened, and a position in it is not a name that lasts.

## `before/` and `after/`

Read-only `btrfs` snapshots of that person's home subvolume, taken either side
of the turn. `before/` is what an undo puts back from; `after/` is what an undo
is checked against, so that a change made since is found and the undo refused
rather than written over (ADR 0045's third point).

**A turn the machine had no room to bracket has neither**, and that is not an
error: the turn still ran, and it says plainly that it cannot be undone (ADR
0045's third term). A directory with `kept.json` and no snapshots is read and
removed like any other.

## What a reader may and may not do

- **A file that does not read leaves everything beside it exactly where it is.**
  A person's directory whose `theirs.json` will not read is stepped over whole;
  one kept turn whose `kept.json` will not read is stepped over on its own. In
  both cases the snapshots stay on the disk. Removing an undo the machine
  cannot describe would take it away and leave nothing able to tell the person
  it had gone, which is the second term's whole point — and a snapshot left
  behind costs disk, while one removed silently costs the only account there is.
- **One person's broken directory does not stop anybody else's disk being
  tidied.** The walk goes on and says what it stepped over.
- **What is removed is removed in one order**: `after/`, then `before/`, then
  `kept.json`, then the directory. A run interrupted anywhere therefore leaves a
  directory that can still say which turn it was.
- **Nothing is written to the record before the snapshot is gone**, and only for
  what actually went.

## What is not in this folder

- **No window, and no setting of any kind.** The window is the person's, in
  their own folder (`docs/contracts/person-settings.md`, `undo.toml`).
- **No record.** What the machine let go of is written to
  `/var/lib/alo-letting-go/record.jsonl` as a `let-go` entry
  (`docs/contracts/record-file.md`), which is not inside the folder being
  removed from. What a person's **own act** forgot is written the same way to
  `/var/lib/alo-forgetting/record.jsonl`, by the other unit that removes from
  this folder (`docs/contracts/asked-to-forget-folder.md`): two record files,
  because a record file has one writer and those are two units a machine may run
  in the same second.
- **No asking.** How a person asks their machine to forget everything it is
  keeping for them is `/run/alo/asked-to-forget`, which is not in here and is
  not on a disk that survives a restart
  (`docs/contracts/asked-to-forget-folder.md`).
- **Nothing of any other person's.** A person's directory holds that person's
  home and nothing else, and no reader ever crosses from one into another.

## On a machine that keeps nothing

A machine installed on a filesystem without subvolumes has no such folder, and
that is not an error: it is ADR 0045's sixth term, *a machine installed on
`ext4` keeps working and answers* not yet on this machine *for every undo,
honestly, until it is reinstalled.* A folder that is not there means nobody has
kept anything, which is also the ordinary state of a machine on which no agent
has ever changed a file.
