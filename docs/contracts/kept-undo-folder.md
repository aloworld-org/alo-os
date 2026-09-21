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

**Nothing an agent can reach is in it.** There is no verb over this folder and
there is not going to be one (ADR 0045's seventh term): an agent that can
forget an undo can erase the evidence of what it did.

## Where it is

```
/var/lib/alo/undo
```

`0700 root:root`. Outside every grant — there is no grant to `/` and none to
this, and a person's agent cannot name a path under it because no verb takes a
path at all.

## The shape

```
/var/lib/alo/undo/                 0700 root:root
  ada/                             one directory per person
    theirs.json                    where that person's settings folder is
    4f1c8a…/                       one directory per kept changing turn
      kept.json                    when it ran, and what it did in their words
      before/                      a read-only snapshot, as the turn found the home
      after/                       a read-only snapshot, as the turn left it
```

A directory directly under the folder is one person's. A directory directly
under that is one kept changing turn. **Nothing else is read**: a file where a
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
  removed from.
- **Nothing of any other person's.** A person's directory holds that person's
  home and nothing else, and no reader ever crosses from one into another.

## On a machine that keeps nothing

A machine installed on a filesystem without subvolumes has no such folder, and
that is not an error: it is ADR 0045's sixth term, *a machine installed on
`ext4` keeps working and answers* not yet on this machine *for every undo,
honestly, until it is reinstalled.* A folder that is not there means nobody has
kept anything, which is also the ordinary state of a machine on which no agent
has ever changed a file.
