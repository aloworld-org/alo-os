# Attributes, ownership and size are inside the grant

- **Date:** 2026-09-12
- **Workstream:** kernel enforcement (`alo-bounding-kernel`, `alo-bounding`,
  and one test in `alo-files`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **Attributes, ownership and size are inside the grant** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 14
- **Status:** ready for integration. The last row of the hardening table that
  did more than litter is closed and measured closed; one remainder is named,
  reproduced in the direction it behaves today, and not closed, with the
  reason. Nothing is ticked *on the machine*.

## What changed, in one paragraph a person can read

Until today the kernel decided what an agent's turn could open, read, write,
move, remove and link, and nothing about what a file *is*. A turn refused
`open` on a file could still change that file's mode, its owner, its times and
its extended attributes — and could empty it, because a truncation reaches the
kernel without an open. The boundary now decides on every change to a file's
size, mode, owner, times, extended attributes and access list, and decides it
by where the file is: a change to a file outside the grant is refused with
`EACCES` before the attribute has moved, and the same change to a file inside
the grant lands. Nothing about grants, the indicator, the record or the egress
policy changed, and the loader holds the same two capabilities it held
yesterday.

## What changed, for whoever reads the code

### Five hooks, not two, and they ask one question

The task named `inode_setattr` and `inode_setxattr`. The kernel splits what a
file *is* five ways, and a boundary with only those two would have had a hole
one `setxattr` call wide:

| Hook | What it is | Why it is here |
|---|---|---|
| `inode_setattr` | size, mode, owner, times | `truncate(2)`, `chmod`, `chown`, `utimensat`, and the `O_TRUNC` an open carries |
| `inode_setxattr` | an extended attribute set | `setxattr` on any name but an access list's |
| `inode_removexattr` | an extended attribute taken away | stripping a `security.` or `user.` attribute from a file outside the grant is the same class of change as setting one |
| `inode_set_acl` | a POSIX access list set | since Linux 6.2 the kernel takes `setxattr` on `system.posix_acl_access` here and **never** to `inode_setxattr`, so a boundary watching the extended-attribute hooks alone refuses `chmod` and allows the same change spelled as a list |
| `inode_remove_acl` | a POSIX access list taken away | the other half of the same |

`crates/alo-bounding-kernel/src/kernel.rs` declares all five; `deciding.rs`
decides them as one function, `decide_attribute`, which is the walk
`inode_unlink` already makes from the entry being changed — `decide_delete`
now calls the same private function, so a delete and an attribute change
cannot walk from different places. No new offset: every attribute hook is
handed a `struct dentry`, and `Fields` already knows how to walk one. No third
map; `the_program_has_nowhere_to_write_what_it_sees` still asserts exactly
`["BOUNDS", "FIELDS"]` and is untouched.

**The entry is the second argument.** Since Linux 6.9 every attribute hook
begins with the mount's identity mapping — `inode_setattr(struct mnt_idmap *,
struct dentry *, struct iattr *)` — so the entry is `arg(1)`, and the previous
module's decision is `arg(3)` for `setattr`, `removexattr` and `remove_acl`,
`arg(4)` for `set_acl` and `arg(6)` for `setxattr`. Those were read from this
kernel's own BTF (`bpf_lsm_inode_setattr` and its siblings) with a throwaway
parser before a line of the programme was written, because the rename hook's
trap in `docs/quirks.md` is what reading the wrong pointer looks like:
everything refused, for no reason anybody can see. `docs/quirks.md` records
the shape.

### Loader and pins

`imposing.rs` attaches twelve hooks; `pinned.rs` pins twelve links, the five
new ones at `/sys/fs/bpf/alo/inode_setattr`, `inode_setxattr`,
`inode_removexattr`, `inode_set_acl` and `inode_remove_acl`, `0600 root:root`
like the others. Every attach or no boundary at all, as before.
`alo-boundaryd`'s tests count hooks through `every_hook()` and needed no
change; they were run.

### The reproductions, before and after

`crates/alo-bounding/tests/the_kernel_refuses_an_attribute_change.rs` is a new
file, because the file that reproduced the unwatched mutations asserts each in
the direction it behaves today and these no longer behave that way. Its two
attribute tests and their child branches moved here and flipped; the file
keeps its five remaining rows.

**Every reproduction was run against the programme before the hooks existed.**
The file was written in its final, refusing form and run first against the
programme as it was: eight of its ten tests failed, each with `left: Allowed,
right: Refused(13)` on the change outside the grant, with the control — the
same turn refused `open` on the same file — passing every time. That is the
gap, measured, in Rust, in the committed suite. The other two passed both
times: the rewrite, which is refused at the open as it always was, and the
process that is not a turn.

| Change | Outside the grant, before | Outside the grant, now | Inside the grant |
|---|---|---|---|
| size, `ftruncate` through a descriptor opened before the turn | the file emptied | **`EACCES`, and the file holds what it held** | emptied |
| a rewrite, `open` with `O_TRUNC` | `EACCES` at the open | `EACCES` at the open | the open goes and so does the truncation it carries |
| mode, `chmod` | changed | **`EACCES`, mode unchanged** | changed |
| owner, `chown` | changed | **`EACCES`, owner unchanged** | changed |
| times, `utimensat` | changed | **`EACCES`, times unchanged** | changed |
| an extended attribute set | set | **`EACCES`, not there** | set |
| an extended attribute taken away | gone | **`EACCES`, still there** | gone |
| an access list set | set | **`EACCES`, not there** | set |
| an access list taken away | gone | **`EACCES`, still there** | gone |
| inode flags, `FS_IOC_SETFLAGS` through a descriptor opened before the turn | `nodump` set | `nodump` set — **not closed** | set |

Every refusal asserts the file nobody granted is exactly as it was —
contents, length, mode, owner, modification time, and the attribute or list
in question — and every allowance asserts the file inside the grant was
changed in the one way asked, both read from outside the turn. A process that
is not a turn makes all ten changes with the programme loaded and is refused
none.

**How the truncation reached the suite.** `docs/quirks.md` had recorded, by
hand and with a Python child, that a bound turn could empty a file it could
not read, and that no call this repository can make reaches `truncate(2)`
without an open: `std` has no path truncate and `rustix` has only `ftruncate`.
It does — on a descriptor opened **before** the turn began, the way the
daemon's own descriptors are. `ftruncate` is not a read or a write, so
`file_permission` never sees it; it is `inode_setattr` on the file's own
entry, which is the call `truncate(2)` makes and the reason the two share a
hook. The child opens the file before it joins the control group and shortens
it from inside. Before the hook the file emptied. After it, `EACCES`.

### What else moved

- `the_boundary_decides_and_forgets.rs` makes every one of these changes to
  every ordinary file outside a turn beside its opens and messages — mode,
  owner and back, times, an attribute set and taken away, an access list set
  and taken away, a shortening — each asserted to have landed, and still finds
  two maps, no turns, the same offsets and no trace line.
- `the_unwatched_mutations_are_written_down.rs` names twelve hooks exactly and
  requires each to be mentioned in `deciding.rs`, `lib.rs`, the quirks table's
  entry and the reproduction file; its self-check's *a hook that arrived
  without the documents moving* case now uses `inode_mkdir` as the
  hypothetical, since `inode_setattr` is no longer hypothetical.
- `what_a_bound_turn_can_still_change.rs` loses its two attribute rows and
  says where they went; `docs/quirks.md`'s *Four hooks are not a filesystem*
  table loses the same two rows, and the two paragraphs about the truncation
  that could not be reproduced say what reproduced it.
- `docs/quirks.md` gains an entry, *Attributes, ownership and size are inside
  the grant*, with the table above, the three things worth an afternoon to
  the next reader, and the remainder.
- `docs/contracts/agent-verbs.md` no longer tells an adapter author that a
  bounded turn can change a file's mode, owner and attributes, because it
  cannot; additive, and no verb's shape changed.
- `alo-files`' `failed.rs` gains one unit test: a refused truncation, `chmod`,
  `chown` or `setxattr` is `Failed::TheMachineSaidNo` with the machine's own
  words, equal to a refused open with the same verb, and never *it went away*.
  No production code in that crate changed and no variant was added, which is
  what *one sentence, not four* means in a vocabulary `alo-saying` collects.
- The plan: task 14 marked done; four rows added to section 1; the attribute
  row of section 3 struck and one row added beside it for the flags.

## What is not closed, and why it is named rather than built

**A file's inode flags.** `FS_IOC_SETFLAGS` — `chattr`'s `nodump`,
`noatime`, `append-only` and `immutable` — is an `ioctl` on a descriptor,
which is `file_ioctl` and not a change to an inode by name, so none of the
five hooks sees it. It is reproduced in the same file, asserted in the
direction it behaves today: a turn with a descriptor opened before it began
sets `nodump` on a file it is refused `open` on. What bounds it is real and is
not the boundary's — a descriptor to a file outside the grant cannot be opened
inside a turn, so only a file already open is reachable; `alo-agentd` runs as
the person with no capability at all, so the two flags that would change what
a person can do with their file need `CAP_LINUX_IMMUTABLE` and are refused by
the kernel itself; and no byte moves. A hook on every `ioctl` on the machine
is a decision about cost as much as a hook, and it is in the hardening table
with v0.5 rather than added here on the strength of `nodump`.

## Decisions taken here, and why

- **Five hooks, not two.** Above. The access-list pair was found by reading
  the kernel's routing rather than by a failing test, and then shown reaching
  before it was shown refused. The removal pair came because *a turn can
  strip an attribute from a file outside its grant* would have been the next
  row in the table the day after the task closed.
- **One function for all five, shared with the delete hook.** A delete and an
  attribute change are the same question of the same entry; two walks would
  be two answers waiting to disagree.
- **The entry, not its folder.** For `decide_delete`'s reason: a grant can be
  over a single file, and its folder is then not a place the call named.
- **Nothing in the attributes is read.** Which attribute is being changed
  would only ever be a reason to allow some, and *a mode change outside the
  grant is fine* is the argument this task exists to retire.
- **The truncation is reproduced through a descriptor rather than left by
  hand.** A path truncate does not exist in safe Rust and a descriptor one
  does; both reach the same hook on the same entry, so the reproduction is of
  the hook and not of the spelling.
- **The flags are named, reproduced, and not hooked.** Above.
- **No `unsafe` outside `kernel.rs`, no kernel patch, no third map, no new
  capability, no widened grant, no exemption.** No verb in this repository
  changes a file's attributes inside a turn — every `set_permissions` and
  `chown` in `alo-agentd` runs at start-up or in a test — so nothing
  legitimate needed stepping aside from.

## Acceptance criteria, each with the test that says so

| Criterion | Test |
|---|---|
| A truncation on a file outside the grant is refused at the syscall inside a turn, and the same inside the grant succeeds | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_size_is_inside_the_grant` |
| A `chmod` outside the grant is refused; inside it succeeds | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_mode_is_inside_the_grant` |
| A `chown` outside the grant is refused; inside it succeeds | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_owner_is_inside_the_grant` |
| A `setxattr` outside the grant is refused; inside it succeeds | `. alo-bounding the_kernel_refuses_an_attribute_change an_extended_attribute_is_inside_the_grant` |
| The rest of the same hook, and the two hooks the task did not name: times, an attribute taken away, an access list set and taken away | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_times_are_inside_the_grant`, `removing_an_extended_attribute_is_inside_the_grant`, `an_access_list_is_inside_the_grant`, `removing_an_access_list_is_inside_the_grant` |
| An open with `O_TRUNC` inside the grant carries its truncation through | `. alo-bounding the_kernel_refuses_an_attribute_change a_file_is_rewritten_inside_the_grant_and_not_outside_it` |
| A reproduction for each was in the crate before the hook, so the hook is shown to close it rather than believed to | the same file, run against the programme before the hooks — eight failures with `Allowed` where `Refused(13)` is asserted, recorded under *Verification* below |
| Nothing outside a turn is affected: a process that is not a turn makes every change | `. alo-bounding the_kernel_refuses_an_attribute_change a_process_that_is_not_a_turn_changes_what_it_always_could` |
| Nothing outside a turn is observed: the test that holds that is run for these hooks too | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| The four refusals are one sentence in `alo-saying`'s vocabulary, not four | `. alo-files lib failed::tests::a_change_to_a_file_the_kernel_refused_is_the_sentence_a_refused_open_is` |
| The remainder is reproduced in the direction it behaves today | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_flags_are_not_yet_inside_the_grant` |
| The twelve hooks are documented where an auditor reads, and nothing listed as unwatched is watched | `. alo-bounding the_unwatched_mutations_are_written_down every_mutation_this_boundary_does_not_watch_is_written_down` |
| Every hook has a pin and the list is one | `. alo-bounding lib pinned::tests::the_boundary_is_pinned_where_the_decision_says_it_is` |

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started, from `/mnt/c/dev/alo-os-claude` with this checkout's own build
directory as the supervisor chooses it. **WSL is development evidence and
never certified-hardware acceptance**; no *on the machine* box is affected.

### The reproduction, before the hooks

`cargo test -p alo-bounding --test the_kernel_refuses_an_attribute_change`
against the programme as it stood at `9ce732f`:

```text
test result: FAILED. 2 passed; 8 failed; 1 ignored
```

Each of the eight with `left: Allowed, right: Refused(13)` on the change
outside the grant: size, mode, owner, times, an attribute set, an attribute
taken away, an access list set, an access list taken away. The two that
passed were the rewrite and the process that is not a turn.

### After the hooks

- `cargo test -p alo-bounding` — passing; every kernel test against the real
  loaded programme with twelve hooks attached.
- `cargo test -p alo-boundaryd` — passing.
- `cargo test -p alo-files` — passing, under WSL and on Windows.
- `cargo test` in `crates/alo-bounding-kernel` — nothing to run, and the
  package builds.
- `cargo fmt --all --check` — clean, in both workspaces.
- `cargo clippy --workspace --all-targets -- -D warnings` on Windows — clean.
- `cargo clippy -p alo-bounding -p alo-boundaryd -p alo-files -p alo-bounding-map --all-targets -- -D warnings`
  under WSL — clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean.
- `cargo doc --no-deps` for the touched crates with `RUSTDOCFLAGS=-D warnings`
  — clean.

### Not run here

- **The whole workspace's suite**, by instruction; the supervisor runs it.
- **Certified hardware.** Every measurement above is WSL2.

## Limitations

- **Inode flags** through a descriptor opened before the turn began, as
  above.
- **Cost.** Five hooks more on every attribute change on the machine, each
  one hash lookup and a miss outside a turn. Not measured on a certified
  machine; `docs/hardware.md` is where that belongs.
- **The access-list measurement is on `tmpfs`**, which accepts one. A
  filesystem mounted without ACL support refuses the call before any hook,
  and the test would say so rather than pass.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **What a file *is* is inside the grant too.** Until now the kernel boundary
> decided what an agent's turn could open, read, write, move, remove and
> link, and nothing about a file's attributes: a turn refused a file could
> still change its mode, its owner, its times and its extended attributes,
> and could empty it, because a truncation reaches the kernel without an
> open. The boundary now decides on every change to a file's size, mode,
> owner, times, extended attributes and access list, by where the file is: a
> change outside the grant is refused before the attribute moves, and the
> same change inside it lands. One thing remains and is written down: a
> file's inode flags, set through a descriptor that was open before the turn
> began, which the kernel already bounds for the two flags that would matter.

### `docs/autonomy/QUEUE.md`

Task 14 of the kernel-enforcement plan is complete. Tasks 15 and 16 are
unchanged and ready.

### `docs/autonomy/STATE.md`

Reference `docs/autonomy/updates/attributes-inside-the-grant.md`. The
hardening table has one row fewer that destroys and one row more that is
named; the honest sentence is still *implementation complete for the in-scope
v0.01 requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed.
