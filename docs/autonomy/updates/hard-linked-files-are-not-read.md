# Hard-linked files are not read

- Date: 2026-09-07
- Workstream: filesystem security (`crates/alo-files`)
- Contributor: Claude Code, filesystem-security workstream
- Status: ready for integration

## What changed

`docs/quirks.md` has carried an unanswered exposure since 2026-09-02 —
*Resolving a path does not defeat a hard link* — and named the shape of its
answer: "a policy about link counts at the moment of opening, not a cleverer
path comparison". This is that policy.

A **hard link** is a second *real* name for one file. `alo-files` resolves every
path a verb names and asks the grants where it really leads, which stops a
symbolic link out of a granted folder; it cannot stop this one, because
`/home/anna/Invoices/notes.txt` can be a name for a file that also lives in
`/home/anna/.ssh`, and resolving it answers with the granted name. The granted
name genuinely *is* a name for that file, so no comparison of paths could do
better.

What a file will answer is **how many names it has**, and a handle is what to
ask. `crates/alo-files/src/opening.rs` now asks the open file — not the path —
and refuses to hand back a regular file with more than one name. Both verbs that
read bytes go through that one door, so `read_file` and `archive_folder` answer
the same way about the same file.

### User-readable change description

> **A file that has been given a second name somewhere else is no longer read.**
> On Linux and macOS, a file can have more than one name, and the other names
> can be anywhere on the machine — including outside anything you granted. That
> made it possible to put a second name for a private file inside a folder you
> had shared with an agent, and have the agent read it while the record showed
> only the folder you granted.
>
> Your agent now checks, at the moment it opens a file, whether the machine
> knows that file by more than one name, and does not read it if so. You are
> told which file, that it was not read, and what to do instead — make a copy
> and read that, or grant the folder the other name is in. Making an archive of
> a folder that holds such a file stops and says so, rather than quietly
> producing an archive with a document missing.
>
> This errs on the side of not reading: a file whose second name sits harmlessly
> in the same folder is refused too, because a file cannot say where its other
> names are. On Windows the check does not apply.

### Source paths

- `crates/alo-files/src/opening.rs` — the `Opening` outcome type, the count
  asked of the open handle in `more_than_one_name`, and the policy in
  `read_only`. Two unit tests.
- `crates/alo-files/src/failed.rs` — the `Failed::HasAnotherName` variant, its
  `said` arm, and `Failed::opening`, the companion to `Failed::machine` that
  keeps *the machine said no* and *this crate declined* apart at one place
  rather than at each call site.
- `crates/alo-files/src/words.rs` — `HAS_ANOTHER_NAME`, added to `EVERY_WORD`
  (41 → 42), with the sentence a translator meets.
- `crates/alo-files/src/looking.rs`, `crates/alo-files/src/zip.rs` — the two
  call sites, now reporting through `Failed::opening`.
- `crates/alo-files/src/testing.rs` — the new failure added to `every_failure`,
  which is what holds *every failure this crate can have is something it can
  say* to the new variant.
- `crates/alo-files/src/lib.rs` — the crate's own account of what it can and
  cannot do, which said this was unsolved.
- `crates/alo-files/tests/a_file_with_another_name_is_not_read.rs` — new, six
  tests.
- `crates/alo-bounding/tests/a_hard_link_is_inside_every_boundary.rs` — new, the
  measurement below. `crates/alo-bounding/Cargo.toml` unchanged by this task.
- `docs/quirks.md`, `docs/contracts/agent-verbs.md` — updated in this task.

## Decisions and acceptance criteria

**No approval is required for this change.** No accepted security decision moved:
no ADR changed, `alo_files::Reaching` is untouched, the kernel boundary is
untouched, and nothing a turn may reach got wider. The change only ever refuses
more than before.

**The refusal is a `Failed`, not a `Refused`.** The grants really did permit this
path; saying they refused it would tell a security review that the capability
model caught something it cannot catch. So it travels inside a `Did` with the
authorisation beside it, which is `failed.rs`'s existing rule.

**Refuse, do not report.** The alternative was to read the file and note the
extra names in the answer. Rejected: the exposure is that content leaves a grant
and can leave the machine, and a note in an answer nobody reads does not stop
that.

**Refuse, do not omit.** For `archive_folder` the alternative was to skip the
file and count it, the way `walking.rs` already counts symbolic links. Rejected
on `archiving.rs`'s own rule — a bound refused in words beats an archive missing
a document nobody mentioned. A symbolic link is a pointer; a hard-linked file is
a real document in that folder, and leaving it out silently is the other way to
be wrong.

**Directories are exempt.** Every directory has at least two names — its own and
the `.` inside it — and one with subdirectories has one more for each. Counting
names on directories would refuse every folder on the machine. There is a test
whose only job is to fail if that is ever got wrong.

### The acceptance criterion, and it was measured

A refusal that will sometimes cost somebody a file they were entitled to needs
the justification that **nothing else catches this**. The capability model cannot,
for the reason above. The claim about the kernel boundary — that it decides an
open by walking up from the file's own directory entry, and a hard link's entry
sits in the granted folder — was a claim about how
`crates/alo-bounding-kernel` works, so it was put to the kernel rather than
reasoned about.

`crates/alo-bounding/tests/a_hard_link_is_inside_every_boundary.rs` binds a turn
to one granted folder with the real programme loaded, and opens one file by two
names:

| | |
|---|---|
| the private file, under its own name | `EACCES` |
| **the same file, under its second name inside the granted folder** | **opens, and is read** |

The first is the control: without it the second would be a boundary that was
never in force. The second is the finding. Nothing is wrong with the boundary —
it is doing exactly what ADR 0015 describes — but it is not a second answer to
this question, so the check in `alo-files` is the only one there is. If that ever
stops being true the test fails, and the refusal can be reconsidered.

## Verification

Platform: Ubuntu on WSL2, kernel 6.18.33.2, stable Rust 1.98.0, with a
`CARGO_TARGET_DIR` belonging to this checkout alone.

Executed, all on Linux, all passing:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `cargo test --workspace` — **110 test binaries, all ok, no failures** (was 108
  before this task; the two new test files account for the difference). Includes
  the three tests that load the real BPF LSM on this kernel.
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — clean.

New tests, and what each is for:

- `opening.rs`: a file with a second name outside the folder is not opened, and
  the refusal is this crate's own rather than something the machine said; the
  same file opens again the moment the other name is removed, so what is refused
  is the sharing rather than anything about the file.
- `a_file_with_another_name_is_not_read.rs`: the granted name for somebody
  else's file is not read and its bytes do not come back; it is recorded as a
  failure with the authorisation beside it rather than as a refusal by the
  grants; the person is told which file and what to do, and is **not** told the
  other name, which this crate has no way of knowing; an archive of the folder
  is refused rather than made without the file; an ordinary file and a folder
  with a subfolder in it are untouched; and two names inside one granted folder
  are refused as well, which is a test that exists to state a cost rather than to
  celebrate the rule.
- `a_hard_link_is_inside_every_boundary.rs`: the measurement above.

**The tests were checked against the code without the change.** With
`more_than_one_name` forced to answer *no*, five of the six integration tests
fail and the ordinary-day control still passes. The fixture also asserts, before
anything else, that reading the granted name **by name** returns the private
file's bytes — so a failure here cannot be the fixture quietly not having worked.

Not executed, and not claimed:

- **No Windows run.** The check is `cfg(unix)` and its Windows half is a stub
  that answers *not that we can tell*; the tests are `cfg(unix)`. Nothing in this
  change alters Windows behaviour, but no Windows gate was run for it.
- **No macOS run.** The check is `cfg(unix)` and so should apply there, and it
  has not been executed on macOS.
- **No physical hardware acceptance.** Every measurement above is from WSL2,
  which `docs/hardware.md` says cannot certify a machine. No *On the machine*
  box is affected by this task and none should be ticked from it.

Shared kernel state: the new boundary test takes the existing
`on_this_kernel::one_at_a_time()` lock and pins under a path named for its own
process, which is the arrangement the three existing files in that directory
already use. It is a fourth such file and changes none of the arrangement. `bpffs`
was mounted at `/sys/fs/bpf`, where a WSL restart had lost it, which is what the
supervisor does and what `docs/hardware.md` asks for. No other worker's processes
were stopped, and nothing was unloaded.

## Remaining limitations

- **Windows is not covered.** `std` exposes no name count there and NTFS supports
  hard links, so the exposure remains on Windows. Written into `docs/quirks.md`
  rather than left to be discovered.
- **A harmless second name is refused too.** A file cannot say *where* its other
  names are — that would be a scan of every filesystem it could be on — so two
  names inside one granted folder are refused along with the case that matters.
  This is a real cost to whoever meets it, it is the safe direction, and there is
  a test that says so.
- **The hard link is still not *prevented*.** Somebody who can write to a granted
  folder can still create one; what changed is that the agent will not read
  through it. Preventing it is not this crate's to do.
- **Not a privilege escalation fix.** Making a hard link needs write access to
  the granted folder and read access to the file, so whoever could set this up
  could already read the file. What it closes is the widening of *what an agent
  sees* without widening a grant, and what an agent sees can leave the machine
  while the record names only a granted path.

## Proposed shared-document updates

For the integration owner; I have not edited these four files.

**CHANGELOG.md** — the user-readable description above, under Unreleased.

**ROADMAP.md** — no box changes. If the `alo-agentd` line's *The code* clause is
being extended, the sentence is: *and since the hard-link answer a file the
machine knows by more than one name is not read at all, because a second real
name for a file is one no comparison of paths can see and neither the grants nor
the kernel boundary catch it — measured on a running kernel rather than assumed.*

**docs/autonomy/QUEUE.md** — this task had no queue item; it came from
`docs/quirks.md`. Suggested entry under *Already built, outside the loop* or
beside the other filesystem-security items, named descriptively rather than by
code:

> **Hard-linked files are not read.** The exposure `docs/quirks.md` recorded on
> 2026-09-02 and left open, answered the way that entry predicted: a count of a
> file's names, asked of the open handle. Refuses rather than reports, and
> refuses rather than omitting from an archive. Measured against the real
> boundary first, which confirmed the kernel does not catch it either, so this
> check is the only one there is. Cost recorded: a second name inside the same
> granted folder is refused too.

**docs/autonomy/STATE.md** — reference this report at
`docs/autonomy/updates/hard-linked-files-are-not-read.md`. The one fact worth
carrying into the journal beyond the change itself is the measurement: **a hard
link inside a granted folder is inside the kernel boundary**, which is the
second time in three tasks that putting a claim to a running kernel changed what
got built.

Also still awaiting consolidation from the previous task, if it has not been
picked up: queue item **6d** — a plain rename of an ungranted file succeeds under
the boundary, because ADR 0015 names `inode_rename` beside `file_open` and only
`file_open` is built.
