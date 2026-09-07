# Kernel protection for deletion and links

- Date: 2026-09-07
- Workstream: filesystem security (`crates/alo-bounding-kernel`, `crates/alo-bounding`, `crates/alo-boundaryd`)
- Contributor: Claude Code, filesystem-security workstream
- Status: ready for integration

## The gaps, reproduced before anything was built

The rename-enforcement report named two more mutations the boundary did not
watch. Both were reproduced first, on this kernel with the real programme
loaded, using temporary fixtures under `/tmp` — a turn bound to `Invoices`, with
`Private` beside it holding a file nobody granted:

| What a bound turn tried | Before |
|---|---|
| delete `Private/secret.txt` | **succeeded** |
| hard-link `Private/secret.txt` → `Invoices/notes.txt` | **succeeded** |
| hard-link `Invoices/march.pdf` → `Private/march.pdf` | **succeeded** |

The reproduction is
`crates/alo-bounding/tests/the_kernel_refuses_a_delete_or_a_link.rs`, run before
the hooks existed: three tests failed with `left: DidIt, right: Refused(13)`, and
the four cases that had to keep working already passed. That ordering matters —
the must-not-break cases were specified and green *before* the change, so they
are evidence rather than something adjusted afterwards to fit.

The hard link is the one with history. `docs/quirks.md` has recorded since
2026-09-02 that a hard link inside a granted folder is inside every check a path
can make, and `alo-files` answers that by refusing to *read* a file the machine
knows by more than one name. This is the other half: a turn cannot **make** one.

## The checks, defined before implementing

| Hook | Arguments | What must be inside the bound |
|---|---|---|
| `inode_unlink(dir, dentry)` | decision at 2 | the **entry being removed** |
| `inode_link(old_dentry, dir, new_dentry)` | decision at 3 | the **source file**, and the **parent of the new name** |

A link is the same question a rename asks — *this file, into that folder* — so
it shares `a_name_being_made` with it rather than having a second copy of the
walk. A delete is one end.

**Why the entry and not its parent**, for both: a grant can be over a single
file, and its folder is then not a place the call named. Judging by the parent
would refuse work the grants allow — the same trap the rename hook had to avoid,
where the folder a `move_file` takes a file out of is not in its reach.

**Why the *parent* for a link's destination**: the new name does not exist yet,
so its directory entry is negative and has no inode to be asked about. The
folder it would be made in is what a call names anyway.

## Decisions and acceptance criteria

**No approval is requested and no accepted decision moved.** ADR 0015 names the
security hooks as the mechanism and this adds two more of them; ADR 0013's
description of the boundary as a floor under a verb with a bug in it is what
these implement. `alo_files::Reaching` is untouched, no grant covers more, no
new agent capability exists, and no map or field was added — both hooks use the
walk and the offsets that were already there. `every_map_the_kernel_holds` still
answers two, which `the_boundary_decides_and_forgets.rs` holds the programme to.

**Inside the bound is not the same as authorised, and nothing here assumes it
is.** None of the six verbs deletes anything and none makes a link, so a turn
doing either is a verb with a bug in it. `alo-capability` is what refuses such a
call, long before a syscall; what the kernel adds is that the bug cannot reach
outside the places the call itself named. Reading *a delete inside a granted
folder is allowed* as authorisation would be reading the boundary as the
capability model, which it is not and does not replace. That sentence is in
`deciding.rs` beside the code, not only here.

**One legitimate delete exists and is tested.** When writing an archive fails
part of the way through, `alo-files` removes the half-written file it made, in
the folder the archive was going into — a place that call named. A boundary
judging deletes by the parent folder would have broken a verb that works today.

**Every attach, or no boundary.** A machine with some hooks attached and one
refused would look like a working boundary while watching less than it claims,
so a failure on any of the four takes the earlier ones' pins away again.

## Verification

Platform: Ubuntu on WSL2, kernel 6.18.33.2, stable Rust 1.98.0 for the workspace
and the pinned `nightly-2026-06-01` for the BPF half, with a `CARGO_TARGET_DIR`
belonging to this checkout alone.

Executed and passing:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `cargo test --workspace` — **112 test binaries, all ok, no failures** (was 111).
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — clean.
- In `crates/alo-bounding-kernel`, on the pinned nightly: `cargo fmt --all
  --check` and `cargo clippy --release --target bpfel-unknown-none -Z
  build-std=core -- -D warnings` — both clean.

### Against the real loaded BPF LSM

`the_kernel_refuses_a_delete_or_a_link.rs`, each case one turn in its own cgroup
with the programme attached on this kernel:

| What the turn tried | Bound to | Before | Now |
|---|---|---|---|
| delete `Private/secret.txt` | `Invoices` | succeeded | **`EACCES`** |
| link `Private/secret.txt` → `Invoices/` | `Invoices` | succeeded | **`EACCES`** |
| link `Invoices/march.pdf` → `Private/` | `Invoices` | succeeded | **`EACCES`** |
| delete `Archive/invoices.zip` (the archive verb's own cleanup) | `Invoices`, `Archive` | allowed | allowed |
| link within `Invoices` | `Invoices` | allowed | allowed |
| delete, **not a turn** | — | allowed | allowed |
| link, **not a turn** | — | allowed | allowed |

**Data unchanged after refusal** is asserted in each refusal test: the source
still holds its own contents, and no second name was created at the destination.

**Existing enforcement still passes**: `the_kernel_refuses.rs` (four tests on
opening), `the_kernel_refuses_a_rename.rs` (six on moving),
`a_hard_link_is_inside_every_boundary.rs` and `what_an_o_path_handle_is.rs` all
pass unchanged, as does `alo-agentd`'s `a_turn_is_bounded_by_the_kernel`, which
carries out a real approved read, archive and move inside a real boundary.

### Loader attachment and cleanup

- `a_machine_that_already_has_a_boundary_keeps_it` now runs **once per hook**:
  whichever single pin is left on a machine — by a killed loader, or by an older
  build with fewer hooks — a second loader refuses *before* it loads anything
  and the leftover pin is byte-for-byte what it was. That is the property that
  keeps a loader from removing somebody else's pins, and it matters more with
  four hooks than it did with one, because a partially-pinned machine is now a
  state that can exist.
- `taking_a_boundary_away_leaves_none_of_its_hooks_attached` is new: every pin
  goes, the directory goes, and `nothing_is_there` is satisfied afterwards. A
  pin left behind is a hook still attached on a machine somebody believes has no
  boundary. It is written against the *count* of `Pinned::every_hook` rather than
  four names, so a fifth hook cannot be added to attaching and not to removing.
- `alo-boundaryd`'s existing `a_second_loader_refuses_and_leaves_the_first_boundary_alone`
  covers the same property end to end on a real kernel, and still passes.

**Not forced, and not claimed:** a genuine mid-attach kernel failure — the third
hook refusing after two attached — cannot be provoked on a healthy machine, so
the unpin-on-failure path is covered by the shape of the code and by the tests
above rather than by having been executed. Said plainly rather than implied.

### Coordination of shared BPF state

The new test takes the existing `on_this_kernel::one_at_a_time()` lock and pins
under a path named for its own process, as the five existing files in that
directory do; this is a sixth and changes none of the arrangement. `bpffs` was
mounted at `/sys/fs/bpf` where a WSL restart had lost it. **No pin belonging to
anything else was removed** — the fixture's `Drop` removes only its own
per-process path, and no test calls `taken_away` on a real machine root. No other
worker's processes were stopped.

**Note for the other worker:** the pin directory now holds six entries, two more
than before (`inode_unlink`, `inode_link`). A machine carrying pins from an older
build will refuse a new loader until they are removed, which is the intended
behaviour and is what the per-hook test above covers.

## Remaining limitations — what is still not enforced

Stated as a list rather than folded into prose, because the point of it is that
this is **not** complete filesystem enforcement. The same list is in
`deciding.rs` beside the code.

- **Symbolic links** (`inode_symlink`) are unwatched. A turn can create one
  pointing anywhere. Following it to read something is a `file_open`, which is
  watched, and `alo-files` refuses a path with a link in it — so this leaves a
  *name* somewhere rather than any contents.
- **Directories** (`inode_mkdir`, `inode_rmdir`) are unwatched: a turn can make
  and remove empty ones. Removing a directory that holds anything needs its
  contents gone first, which is `inode_unlink` and is watched.
- **Creating a file** (`inode_create`) is unwatched. Writing to it is an open,
  which is watched, so what this leaves is an empty file somewhere.
- **Attributes and ownership** (`inode_setattr`, `inode_setxattr`) are
  unwatched.
- **What happens to a file already open** is not revisited: a boundary on
  `file_open` decides at the moment of opening and says nothing afterwards.
- **Anything that is not a filesystem** — sockets, signals, memory. ADR 0013
  names the network boundary as its own piece of work; it is not built.
- **A hard link made before the turn began** is still invisible to every hook.
  That half is answered by `alo-files` refusing to read a file with more than
  one name, and the two do not replace each other.
- **Physical hardware acceptance is outstanding.** Every measurement is from
  WSL2, which `docs/hardware.md` says cannot certify a machine. No *On the
  machine* box is affected and none should be ticked from this.

## Proposed shared-document updates

For the integration owner; I have not edited those four files.

**CHANGELOG.md**, under Unreleased:

> **An agent can no longer delete your files outside what you granted, or give
> one of them a second name somewhere you did not approve.** Deleting and
> hard-linking were the two ways left to move a file's contents past a grant
> without ever opening it: a second name inside a folder you shared makes every
> later read of somebody else's file look entirely legitimate. Both are now
> refused by the machine itself rather than by alo OS's own code, a refused
> operation changes nothing at either end, and deleting or linking inside the
> folders you granted still works — as does everything a person's own programs
> do, which the boundary never applies to.

**ROADMAP.md** — no box changes. The `alo-agentd` line's *The code* clause could
gain: *and the kernel now watches what a turn removes and what it links as well
as what it opens and moves, so the four ways a turn can put a file's contents
somewhere nobody approved are all refused by the machine — with the directory,
symbolic-link and attribute hooks written down as not yet watched rather than
implied to be.*

**docs/autonomy/QUEUE.md** — the follow-up named in the rename report is done.
Suggested entry:

> **Kernel protection for deletion and links.** The two mutations left after
> renames: a bound turn could delete a file nobody granted, and could hard-link
> one into a granted folder — both reproduced on a running kernel before
> anything was built. Two more BPF LSM hooks, sharing the walk and the offsets
> the others already used. No ADR changed, no grant widened, no new capability
> or map. Seven tests against the real loaded programme, plus per-hook loader
> tests that a leftover pin is refused over rather than removed. What is still
> unwatched — symlinks, directories, file creation, attributes — is listed in
> `deciding.rs` and in the report rather than left implied.

**docs/autonomy/STATE.md** — reference this report at
`docs/autonomy/updates/kernel-protection-for-deletion-and-links.md`. The fact
worth carrying: the boundary is four hooks and six pins, a partially-pinned
machine is now a state that can exist, and the loader refuses over any single
leftover pin without removing it.
