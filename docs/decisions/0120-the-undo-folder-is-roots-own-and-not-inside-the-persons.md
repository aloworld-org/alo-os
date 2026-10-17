# ADR 0120 — The undo folder is root's own, and not inside the person's

**Status:** accepted — it corrects the location
[ADR 0045](0045-what-undoing-rewinds-to.md) term 8 and
`docs/contracts/kept-undo-folder.md` both named, and it corrects the measurement
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
rests on
**Date:** 2026-10-17
**Context:** [ADR 0045](0045-what-undoing-rewinds-to.md) terms 7, 8 and 9,
[ADR 0118](0118-more-than-one-person-on-a-machine.md),
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md),
`docs/contracts/kept-undo-folder.md`, `docs/contracts/machine-description.md`,
`image/usr/lib/tmpfiles.d/alo.conf`, `crates/alo-giving-a-folder`,
`crates/alo-letting-go`

## The decision in one line

What an undo would put back lives in **`/var/lib/alo-undo`**, a directory of
root's own under `/var/lib`, and not in `/var/lib/alo/undo` — because
`/var/lib/alo` is the **person's** directory, `0700` and theirs, and putting the
undo folder inside it made two of ADR 0045's own terms false.

## How this was found

[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
built `crates/alo-giving-a-folder`, the privileged step ADR 0045 term 8 asked
for. Its unit was installed on a machine and started, and it **did not start**:

```
alo-giving-a-folder@1000.service: Failed to set up mount namespacing:
  /var/lib/alo: No such file or directory
alo-giving-a-folder@1000.service: Main process exited, code=exited, status=226/NAMESPACE
```

That first failure was the test machine's and not the product's — the image
creates `/var/lib/alo` through `tmpfiles.d` at boot and the virtual machine had
never run it. With the line applied by hand, the unit started and the program
failed instead:

```
alo-giving-a-folder: /var/lib/alo-undo could not be made, so nothing of this
  person's files can be kept for an undo: Permission denied (os error 13)
```

**Permission denied, as root.** `CapabilityBoundingSet=CAP_CHOWN` drops
`CAP_DAC_OVERRIDE`, and `image/usr/lib/tmpfiles.d/alo.conf` declares

```
d /var/lib/alo 0700 alo alo -
```

`/var/lib/alo` is the **person's** directory, deliberately so: it holds the
machine's record and `grants.toml`, and the comment on that line says why —
*what an agent did on somebody's machine is theirs.* A root process without
`CAP_DAC_OVERRIDE` cannot write into a directory it does not own.

## The real fault is not the capability

Granting `CAP_DAC_OVERRIDE` makes the program work, and that was measured too.
It does not fix anything, and the measurement is why:

| | `/var/lib/alo/undo` with `CAP_DAC_OVERRIDE` | `/var/lib/alo-undo` with only `CAP_CHOWN` |
|---|---|---|
| the unit starts | yes | yes |
| the folder is `0711 root:root` | yes | yes |
| a person writes in their own directory | yes | yes |
| **a person can move the whole folder away** | **yes** | no |
| **a second person reaches their own directory** | **no** | yes |

The two rows in bold are the decision.

**A person could move the folder away.** Running as `alo`, with no privilege at
all: `mv /var/lib/alo/undo /var/lib/alo/gone` **succeeded**. The person owns the
parent, so the parent's write bit is theirs, and an entry in a directory you can
write is an entry you can rename. ADR 0045's seventh term is that there is no
verb over this folder because *an agent that can forget an undo can erase the
evidence of what it did* — and at full trust (ADR 0064) the agent acts with
everything the person may do. The whole folder was one rename away from gone.

**A second person could not reach their own directory.** Running as `bea`,
account 1001: `ls -d /var/lib/alo/undo/1001` → `Permission denied`. Not because
of anything about `undo`, whose `0711` was chosen precisely so that every person
may traverse it, but because `/var/lib/alo` above it is `0700` and belongs to
somebody else. ADR 0118 accepted more than one person on a machine, and the mode
ADR 0045 term 8 chose for exactly that case was being defeated one level up by a
line nobody had connected to it.

**Neither of those is a bug in a program.** Both halves were correct — the
tmpfiles line is right about the person's directory, and term 8 is right about
the folder's mode — and the seam between them was wrong. No test could have
caught it, because nothing created the folder and nothing took a bracket, so
there was nothing to run.

## The new location, measured

`/var/lib` is `0755 root:root` on the pinned base, so root makes a directory
there with **no capability at all** — it owns the parent and the parent's write
bit is its own. Measured with `capsh`, holding `cap_chown` and with
`cap_dac_override`, `cap_dac_read_search`, `cap_fowner` and `cap_sys_admin`
dropped, then through the real unit with a caller holding **no capabilities**:

```
/var/lib/alo-undo        mode=711  owner=root:root
/var/lib/alo-undo/1000   mode=700  owner=alo:root
/var/lib/alo-undo/1001   mode=700  owner=bea:root
Result=success  ExecMainStatus=0  CapabilityBoundingSet=cap_chown
```

And the five properties the design rests on, each by reading the disk back rather
than by an exit code:

| | |
|---|---|
| a person writes in their own directory | **yes** |
| a person can list the folder | no — *`Permission denied`* |
| a person can read another person's directory | no — *`Permission denied`* |
| a person can move the folder away | no — *`Permission denied`* |
| a person can delete another person's directory | no — *`Permission denied`* |

**The control.** The same unit with `CapabilityBoundingSet=` and
`AmbientCapabilities=` emptied: `Result=exit-code`, `ExecMainStatus=1`, and the
person's directory left `owner=root:root`. So `CAP_CHOWN` is necessary **in this
location**, which is the measurement ADR 0119 reported from a directory root
already owned — right about the capability, taken about the wrong subject.

## Consequences

- **`ADR 0119`'s one capability survives, and that is not a coincidence.** The
  component needed a second, broad capability only to reach into somebody else's
  directory. Moving the folder to where it belongs removed the need rather than
  the symptom, and `CAP_CHOWN CAP_DAC_OVERRIDE` would have been the shape of a
  design that was wrong one level up.
- **The unit asks systemd for the directory.** `StateDirectory=alo-undo` with
  `StateDirectoryMode=0711`, not `ReadWritePaths=`. A `ReadWritePaths=` on a path
  that does not exist is not a loose end — it is `226/NAMESPACE` and a unit that
  never runs, which is the first failure above.
- **The program puts the mode and the owner back on every call, and that was
  false when this was written.** `StateDirectory=` sets the mode when the
  directory is created and the program sets it again, so a folder somebody
  widened by hand is narrowed. **The second call failed with `EPERM`** until it
  was fixed on the same day: setting a mode needs ownership or `CAP_FOWNER`,
  this unit holds only `CAP_CHOWN`, and by the second call the directory belongs
  to the person. It now takes the directory back, sets the mode and gives it
  away again — all three inside `CAP_CHOWN`, measured, with `cap_fowner`,
  `cap_dac_override` and `cap_dac_read_search` dropped.
  *This paragraph claimed the property before anybody had called the unit twice,
  and the test that existed widened the folder above the person's directory —
  the one with no owner, which is never given away and so can always be
  `chmod`'d. The right measurement of the wrong subject, for the third time in
  one day.*
- **A refused `chown` leaves a directory nobody has been given.** The control
  above left `/var/lib/alo-undo/1002` as `root:root`: the program made it and
  then could not give it away. It reports the failure and the next call repairs
  it. This is written down rather than fixed by removing the directory, because a
  half-made directory a person cannot write is visible and inert, and a remover
  running on a failure path is a second thing that can go wrong.
- **There is one caller, not two, and this document said two.** ADR 0118 names
  the first account's road and a later route for a person who is not the first,
  but `alo-greeting`'s `made_an_account` refuses outright when any account
  exists, so the later route **is not written**. Pointed out by the desktop
  lane, which owns ADR 0118. A road reached only by account creation would never
  reach a person whose account already exists — which is every person on every
  machine that has been switched on once —
  so [ADR 0121](0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)
  puts the call in the road that **opens a session** instead, which everybody
  takes.
- **The path is a published surface and this moves it.** It is named by ADR 0045,
  ADR 0088, ADR 0118, `docs/contracts/kept-undo-folder.md` and six crates, and
  all of them move in the change that carries this decision — 21 references in
  16 files. There is no deprecation window, and there does not need to be: ADR
  0045 recorded on 2026-10-16 that **nothing creates the folder and nothing takes
  a bracket**, confirmed again here, so no machine and no adapter has ever seen
  it. A path nothing has used is renamed, not broken.
- **`docs/quirks.md` keeps the old path** where it appears inside a recorded
  measurement transcript, and so do the published task reports under
  `docs/autonomy/updates/`. A transcript is what was measured on a day; editing
  it to match a later decision would make the record say something nobody ran.
- **`/var/lib/alo` is untouched.** It stays `0700` and the person's, holding the
  record and `grants.toml` as `docs/contracts/machine-description.md` and ADR
  0088 describe. Nothing about that line was wrong.

## Alternatives rejected

**Grant the helper `CAP_DAC_OVERRIDE`.** Measured, and it works, and it fixes
neither of the two faults: the person still owns the parent and can still move
the folder, and a second person still cannot traverse. It buys a working program
on top of a broken design, and it spends ADR 0119's argument — a component chosen
for being small gaining the capability to override every permission check on the
machine.

**Make `/var/lib/alo` traversable — `0711 root:root` — and keep the path.**
Rejected because the person's own daemon writes `grants.toml` and the record
*into* that directory, so the person needs write on it; and a person who can
write the directory can rename anything in it, which is fault one again. Keeping
the path would mean moving the person's own files somewhere else, which is a
larger change to a different contract in order to avoid a smaller one here.

**Leave the folder inside the person's and accept one person per machine.**
Rejected: ADR 0118 is accepted, and this would also leave fault one, which has
nothing to do with how many people there are.
