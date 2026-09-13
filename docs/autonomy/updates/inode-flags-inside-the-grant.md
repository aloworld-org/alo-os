# A file's inode flags are inside the grant

- **Date:** 2026-09-13
- **Workstream:** kernel enforcement (`alo-bounding-kernel`, `alo-bounding`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **A file's inode flags are inside the grant** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 17
- **Status:** ready for integration. The last row of the hardening table that
  a filesystem hook closes is closed and measured closed; two remainders are
  named with the reason each is bounded rather than built. Nothing is ticked
  *on the machine*.

## What changed, in one paragraph a person can read

Until today an agent's turn holding a file that was already open when the
turn began could still change that file's inode flags — the `nodump` and
`noatime` marks `chattr` sets — even on a file the kernel had just refused to
let it open, because a flag is set with an `ioctl` on a descriptor and not by
naming the file. The boundary now decides that request too, by where the file
is: setting a flag on a file outside the grant is refused with `EACCES` before
the flag moves, the same flag on a file inside the grant lands, and every
other `ioctl` on the machine — a terminal asked its size, a device driven — is
let through without the boundary reading anything at all. Nothing about
grants, the indicator, the record or the egress policy changed, and the loader
holds the same two capabilities it held yesterday.

## What changed, for whoever reads the code

### The thirteenth hook, and the one whose first question is the request

`file_ioctl(struct file *file, unsigned int cmd, unsigned long arg)` is
declared in `crates/alo-bounding-kernel/src/kernel.rs` beside the other
twelve: three arguments, the previous module's decision fourth, the file
first as `file_open` and `file_permission` have it. It calls
`decide_request` in `deciding.rs`, which is the one function in that file
whose first question is **not** *is this a turn*:

| Request | Number | Why it is decided |
|---|---|---|
| `FS_IOC_SETFLAGS` | `0x40086602` | how `chattr` sets `nodump`, `noatime`, `append-only`, `immutable` |
| `FS_IOC32_SETFLAGS` | `0x40046602` | the same request in its 32-bit width, which a kernel before 6.8 hands to this same hook |
| `FS_IOC_FSSETXATTR` | `0x401c5820` | the other spelling of the same change, through `struct fsxattr` |
| anything else | — | **allowed before any map is read** |

For one of the three the decision is `decide`'s exactly — the walk from the
file's own directory entry that `file_open` makes — so an open and a flag
change on the same descriptor cannot come to different answers about where
it is. For every other request, inside a turn or not, it is three
comparisons and a return: no hash lookup, no read of kernel memory, no walk.
That order is the task's constraint (*a request the hook does not recognise
is allowed through with no walk*) made the cheapest it can be, and the test
below measures it rather than trusts it.

The numbers are the kernel's own encoding of `_IOW('f', 2, long)`,
`_IOW('f', 2, int)` and `_IOW('X', 32, struct fsxattr)`, read from this
machine's `/usr/include/linux/fs.h` and from `linux-raw-sys`'s `x86_64`
table, which agree. Written as constants because the programme has no
headers, the same reason `EACCES` is `-13` there.

### Loader and pins

`imposing.rs` attaches thirteen hooks; `pinned.rs` pins thirteen links, the
new one at `/sys/fs/bpf/alo/file_ioctl`, `0600 root:root` like the rest, and
gains `flags_hook()`. `every_hook()` and `every_hook_named()` are still the
one list, now of thirteen, and the pin test asserts the thirteenth is the
last attached. `in_place.rs` asks the machine about thirteen pins before
every turn with no change to its loop. `alo-boundaryd` counts hooks through
`every_hook()` and needed no change; its tests were run.

### The reproduction, before and after

`a_files_flags_are_not_yet_inside_the_grant` in
`crates/alo-bounding/tests/the_kernel_refuses_an_attribute_change.rs` was
task 14's reproduction, asserting the flag **landing** on a file the turn was
refused `open` on. It was run against the programme as committed at
`af4c8e1`, before a line of this change, and passed — the gap was there. It is
now `a_files_flags_are_inside_the_grant`, going through the same
`refused_outside_and_allowed_inside` every other attribute test uses:
control refused, the `ioctl` outside the grant `EACCES`, the same `ioctl`
inside the grant landing, the secret's contents, mode, owner, times **and
flags** undisturbed. Beside it, new:

- `a_read_of_a_files_flags_is_not_walked` — `FS_IOC_GETFLAGS` on the same
  inherited descriptor to the same file outside the grant, inside the same
  bound turn, is answered. It is the nearest safe spelling of `TIOCGWINSZ`
  against a file, and it is what proves the request is compared before the
  control group is looked up.
- `a_process_that_is_not_a_turn_changes_what_it_always_could` now makes
  eleven changes, the flag set and the flags read among them, and is refused
  none.
- `the_boundary_decides_and_forgets.rs` sets and clears `nodump` on every
  file of its ordinary day through a held descriptor, beside the attribute
  changes, and finds nothing written down.
- `the_unwatched_mutations_are_written_down.rs` names `file_ioctl` among the
  thirteen and holds the two auditing files and the reproduction file to
  mentioning it.

### What else moved

Counts of twelve became thirteen in `lib.rs`, `imposing.rs`, `in_place.rs`,
`failing.rs`, `pinned.rs`, `a_turn_without_a_boundary_does_not_run.rs` and
`what_a_bound_turn_can_still_change.rs` — prose only; no loop changed.
`deciding.rs`'s list of what is not watched lost the flags and says when and
how they were closed. `docs/quirks.md`'s attributes entry now says its
remainder was closed the next day and points at a new entry, *A file's inode
flags are inside the grant*, with the hook's shape, the cost, and the two
things below.

## What is not closed, and why it is named rather than built

- **`FS_IOC_FSSETXATTR` is refused by the same `match` arm and is not in the
  committed suite.** `rustix` has a safe `ioctl_setflags` and `ioctl_getflags`
  and no safe spelling of the `fsxattr` request; `rustix::ioctl::ioctl` is
  `unsafe`, and `unsafe` is forbidden outside the kernel package's one file.
  Three constants in one arm, one of them measured; the arm is the test of
  the comparison. Named in `docs/quirks.md`, the way the mapping is.
- **`file_ioctl_compat` is not hooked.** Since Linux 6.8 a 32-bit program's
  `ioctl` reaches a hook of its own. What bounds it: a turn is one thread of
  a 64-bit `alo-agentd`; a turn cannot start a program outside its grant
  (`execve` is a `file_open`); and every descriptor Rust's standard library
  opens is close-on-exec, so an inherited descriptor does not survive into a
  program a verb with a bug in it might start inside the grant. A hook on it
  is one `#[lsm]` function calling the same `decide_request` the day it is
  wanted, and recognising `FS_IOC32_SETFLAGS` on `file_ioctl` already covers
  kernels before 6.8.

## Decisions taken here, and why

- **The request is compared before the control group is looked up.** The
  plan asked that unrecognised requests be allowed with no walk; doing the
  comparison first makes them cost no lookup either, which matters because
  `ioctl` is the busiest hook on the machine outside reads and writes. A
  refusal can only come from a request that was recognised, so nothing about
  the refusal path is weaker for it.
- **Three numbers, not two.** `FS_IOC32_SETFLAGS` is not a different request;
  it is `FS_IOC_SETFLAGS` as a 32-bit program spells it, and on a kernel
  before 6.8 it arrives at this same hook. Recognising it costs one
  comparison and closes those kernels; on this one it is unreachable through
  `file_ioctl` and is documented as such.
- **A read of the flags is the measurement of *and for nothing else*.** The
  plan names `TIOCGWINSZ`, which needs a terminal a child process inside a
  turn does not reliably have. `FS_IOC_GETFLAGS` on the very descriptor the
  set is refused on is a stronger measurement: same file, same turn, same
  hook, and it goes through.
- **Task 18 is the five creation hooks.** The plan named no task after 17.
  The remaining filesystem rows — `inode_create`, `inode_mknod`,
  `inode_mkdir`, `inode_rmdir`, `inode_symlink` — are reproduced, bounded to
  a folder's parent by the same reasoning `a_name_being_made` already holds,
  and the only ones left that a hook closes; the mapping needs `unsafe` to
  reproduce and the proxy waits on ADR 0021. Written with acceptance and
  constraints in the plan's shape.

## Acceptance criteria, each with the test that says so

| Criterion | Test |
|---|---|
| Inside a bound turn, setting a flag on a file outside the grant through a descriptor opened before the turn is `EACCES` with the flags undisturbed; the same on a file inside the grant lands | `. alo-bounding the_kernel_refuses_an_attribute_change a_files_flags_are_inside_the_grant` |
| The hook decides the two flag requests and nothing else: a request it does not recognise is not walked | `. alo-bounding the_kernel_refuses_an_attribute_change a_read_of_a_files_flags_is_not_walked` |
| A process that is not a turn is refused nothing | `. alo-bounding the_kernel_refuses_an_attribute_change a_process_that_is_not_a_turn_changes_what_it_always_could` |
| The existing reproduction is reversed into the refusal, in the same file, beside its allowance | the same test, run before the hook (see *Verification*) and after |
| `Pinned` gains the hook where the other twelve are and `every_hook_named()` is the one list | `. alo-bounding lib pinned::tests::the_boundary_is_pinned_where_the_decision_says_it_is` |
| `a_turn_without_a_boundary_does_not_run.rs` refuses a turn when the thirteenth pin is gone without a line changing | `. alo-bounding a_turn_without_a_boundary_does_not_run a_turn_is_refused_when_the_programme_is_not_held_on_a_hook` |
| The hook outside a turn leaves no trace, added to the ordinary day | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| The two maps stay two | `. alo-bounding the_boundary_decides_and_forgets the_program_has_nowhere_to_write_what_it_sees` |
| The thirteen hooks are documented where an auditor reads, and nothing listed as unwatched is watched | `. alo-bounding the_unwatched_mutations_are_written_down every_mutation_this_boundary_does_not_watch_is_written_down` |
| The row moves from section 3 to section 1 and `docs/quirks.md` says what closed and what the kernel already bounded | the plan and the quirks entry, in this change |

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started, from `/mnt/c/dev/alo-os-claude` with this checkout's own build
directory as the supervisor chooses it
(`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`). **WSL is development
evidence and never certified-hardware acceptance**; no *on the machine* box is
affected.

### The reproduction, before the hook

`cargo test -p alo-bounding --test the_kernel_refuses_an_attribute_change
a_files_flags_are_not_yet_inside_the_grant` against the programme at `af4c8e1`:

```text
test a_files_flags_are_not_yet_inside_the_grant ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out
```

Passing there means the flag landed on the file the turn was refused `open`
on, which was the gap.

### After the hook

- `cargo test -p alo-bounding` — every target passing: 36 unit tests and the
  eighteen integration files, every kernel test against the real loaded
  programme with thirteen hooks attached;
  `the_kernel_refuses_an_attribute_change` 12 passed, 1 ignored (the child).
- `cargo test -p alo-boundaryd` — passing, 10 tests.
- `cargo fmt --all --check` — clean, in both workspaces.
- `cargo clippy -p alo-bounding -p alo-boundaryd --all-targets -- -D warnings`
  — clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean.
- `cargo doc --no-deps -p alo-bounding -p alo-boundaryd` with
  `RUSTDOCFLAGS=-D warnings` — clean.

### Not run here

- **The whole workspace's suite**, by instruction; the supervisor runs it.
- **Certified hardware.** Every measurement above is WSL2.

## Limitations

- `FS_IOC_FSSETXATTR` and `file_ioctl_compat`, as above.
- **Cost.** One hook more on every `ioctl` on the machine, three comparisons
  and a return for every request that is not a flag change, inside a turn or
  not. Not measured on a certified machine; `docs/hardware.md` is where that
  belongs.
- **The measurement is on `/tmp`**, which on this machine accepts inode
  flags. A filesystem that does not support `FS_IOC_SETFLAGS` refuses the
  call with `ENOTTY` before any hook, and the test would say so rather than
  pass.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **A file's inode flags are inside the grant.** The one thing left that a
> turn could change about a file it was refused, through a descriptor that
> was open before the turn began, was the file's inode flags — `chattr`'s
> `nodump` and `noatime`, since the two that would matter already needed a
> capability the daemon does not hold. The boundary now decides that request
> by where the file is, refuses it outside the grant before the flag moves,
> and lets every other `ioctl` on the machine through without reading
> anything. The thirteenth hook, and the last on the filesystem that closes a
> row by itself.

### `docs/autonomy/QUEUE.md`

Task 17 of the kernel-enforcement plan is complete. Task 18, *What a turn
makes is inside the grant*, is written and ready.

### `docs/autonomy/STATE.md`

Reference `docs/autonomy/updates/inode-flags-inside-the-grant.md`. The
hardening table's filesystem rows are the five creation hooks and the
mapping; the honest sentence is still *implementation complete for the
in-scope v0.01 requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed.
