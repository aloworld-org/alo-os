# Kernel enforcement for file renames

- Date: 2026-09-07
- Workstream: filesystem security (`crates/alo-bounding-kernel`, `crates/alo-bounding`)
- Contributor: Claude Code, filesystem-security workstream
- Status: ready for integration

## What changed

The boundary a turn runs inside watched what a turn **opened** and not what it
**moved**. That was measured on a running kernel while a different task was
under way: with a turn bound to one folder and the real programme loaded, a
plain rename of a file nobody granted simply succeeded. ADR 0015's own mechanism
section names `inode_rename` beside `file_open`; only `file_open` had ever been
built.

It is built now. `crates/alo-bounding-kernel` carries a second BPF LSM program
on `inode_rename`, and it validates **both ends** of every rename a turn makes:
the source must be inside the bound, and so must the folder the destination goes
in. A rename with either end outside is `EACCES` from the kernel.

### User-readable change description

> **An agent can no longer move a file out of what you granted, or move an
> ungranted file in.** Your grants were already enforced by the kernel when an
> agent *read* a file; moving one was still only checked by alo OS's own code.
> That mattered because moving is a way around reading: a file you never
> granted, renamed into a folder you did, is a file every later read of which
> looks entirely legitimate.
>
> Both directions are now refused by the machine itself rather than by our
> software, and a refused move changes nothing — the file stays where it was,
> and anything already at the destination keeps its own contents. Moving and
> renaming files inside the folders you granted works exactly as before.

### Source paths

- `crates/alo-bounding-kernel/src/kernel.rs` — the second `#[lsm]` hook. It
  reads the two directory entries and the previous module's decision, and the
  two `struct inode` arguments are deliberately not read.
- `crates/alo-bounding-kernel/src/deciding.rs` — `decide_rename`, and the walk
  that both hooks now share (`upwards_from`) rather than a second copy of it.
- `crates/alo-bounding/src/imposing.rs` — `THE_HOOKS`, and one attach-and-pin
  loop run twice. Both attaches or neither boundary.
- `crates/alo-bounding/src/pinned.rs` — the second pinned link,
  `/sys/fs/bpf/alo/inode_rename`, at `0600` like the first.
- `crates/alo-bounding/tests/the_kernel_refuses_a_rename.rs` — new, six tests.
- `crates/alo-bounding/tests/what_an_o_path_handle_is.rs` — the regression that
  recorded the gap, updated.
- `crates/alo-files/src/opening.rs`, `docs/quirks.md` — the prose that said
  renames were unwatched.

## Decisions and acceptance criteria

**No approval was required and none is requested.** This implements an accepted
security decision rather than changing one: ADR 0015 names `inode_rename` in the
mechanism it describes, and this is that hook finally existing. No ADR text
needed to move, and none was touched.

**Nothing was widened.** `alo_files::Reaching` is unchanged, no grant covers
more than it did, and no new map, field or capability was added. The `FIELDS`
map already carried every offset the rename walk needs, because it is the same
walk; the `BOUNDS` map is read by both programs. `every_map_the_kernel_holds`
still answers two, which is what `the_boundary_decides_and_forgets.rs` holds the
programme to.

**The two entries are asked different questions, and that is the whole design.**
`Reaching` says a move is bound to *the file, and the folder it is going into*,
and a rename to *the file, and the folder it sits in*. So:

- the **source** is asked about the entry itself, because the file is what the
  call named and what the bound is made of;
- the **destination** is asked about the entry's **parent**, because a rename to
  a name nothing is at — every no-clobber rename — is handed a *negative*
  directory entry with no inode to ask about, and the folder it would be made in
  is what the call named anyway.

Asking the source's *parent* instead would have refused every legitimate move,
because the folder a `move_file` takes a file out of is not a place its call
names. That is the trap this design exists to avoid, and it is why the
allow-tests matter more than the refuse-tests here.

**Both hooks or neither.** A machine with `file_open` attached and
`inode_rename` refused would watch reads and not moves while looking like a
working boundary — exactly the state this task removed. A failure on the second
attach takes the first one's pin away again, which is ADR 0015's *a turn whose
boundary cannot be applied does not run*, applied to a boundary that is now two
pieces.

**Nothing is written down by the new hook.** No map, no counter, no
`bpf_printk`. ADR 0015's *the LSM decides and forgets* applies to it exactly as
to the first, and the existing test that counts the programme's maps is what
holds it there.

## Verification

Platform: Ubuntu on WSL2, kernel 6.18.33.2, stable Rust 1.98.0 for the
workspace and the pinned `nightly-2026-06-01` for the BPF half, with a
`CARGO_TARGET_DIR` belonging to this checkout alone.

Every check below was executed and passed:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — zero warnings.
- `cargo test --workspace` — **111 test binaries, all ok, no failures** (was 110
  before this task).
- `RUSTDOCFLAGS=-D warnings cargo doc --workspace --no-deps` — clean.
- In `crates/alo-bounding-kernel`, on the pinned nightly:
  `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  — both clean.

### Against the real loaded BPF LSM

`crates/alo-bounding/tests/the_kernel_refuses_a_rename.rs`, each case one turn
in its own cgroup with the programme loaded and attached on this kernel:

| What the turn tried | Bound to | Kernel said |
|---|---|---|
| move `Private/secret.txt` → `Invoices/` (**ungranted source**) | `Invoices` | `EACCES` |
| move `Invoices/march.pdf` → `Private/` (**ungranted destination**) | `Invoices` | `EACCES` |
| rename within `Invoices` | `Invoices` | allowed |
| move `Invoices/march.pdf` → `Archive/` | `Invoices`, `Archive` | allowed |
| the same ungranted move, by a process that is **not a turn** | — | allowed |

**Failed operations preserved data**, asserted after each refusal: the source is
still at its own path with its own contents, and nothing arrived at the
destination. One test aims a refused rename at a destination that is a real file
somebody else's and asserts it still holds `"a file that must survive"`, because
an ordinary `rename` would have replaced it.

**Existing read enforcement still passes**: `the_kernel_refuses.rs` — a granted
file opens, a private key is `EACCES`, authority is gone when the turn ends, and
a process that is not a turn is unaffected — all four still pass with the second
hook attached.

**And the whole journey**: `alo-agentd`'s `a_turn_is_bounded_by_the_kernel`
carries out a real `move_file`, proposed and approved through the capability
model, inside a real boundary. It passes with the rename hook enforcing, which
is the end-to-end evidence that no legitimate move was broken.

**The regression that recorded the gap was updated, not deleted.** The last
probe of `what_an_o_path_handle_is.rs` asserted that a rename of an ungranted
file succeeded, with a message naming the documents to change on the day it
stopped. It failed on this change, exactly as intended, and now asserts
`EACCES`. Its `O_PATH` reasoning is unchanged and did not depend on renames
being unwatched — that is said explicitly in the file, because it is the kind of
argument somebody would otherwise assume had been undermined.

### Coordination of shared kernel state

The new test takes the existing `on_this_kernel::one_at_a_time()` lock and pins
under a path named for its own process, which is the arrangement the four
existing files in that directory already use; this is a fifth and changes none
of it. `bpffs` was mounted at `/sys/fs/bpf` where a WSL restart had lost it,
which is what the supervisor does and what `docs/hardware.md` asks for. No other
worker's processes were stopped and nothing was unloaded. **Note for the other
worker:** a machine that had a boundary pinned by an older build will refuse a
new loader over the leftover pins, and the directory now has a fifth entry
(`inode_rename`); `Pinned::taken_away` removes all of them.

## Remaining limitations

- **Only renames.** `inode_unlink`, `inode_link`, `inode_symlink`,
  `inode_mkdir` and `inode_create` are still unhooked, so a *bugged* verb could
  delete or hard-link outside a grant without the kernel objecting. None of the
  six verbs does any of those, so nothing is wrong today — but that is the
  daemon's honest account of itself again, which is the thing ADR 0013 exists to
  stop being the only thing standing there. Worth a follow-up task; I have not
  filed one in the queue, since that file is the integration owner's.
- **`RENAME_EXCHANGE` is bounded conservatively.** The hook is not given the
  rename flags at all, so an exchange is treated like any other rename: the
  destination is judged by its parent folder. If the parent is granted the file
  under it is too, so this is sound; a single-file grant on an exchange's
  destination whose folder is not granted would be refused. Nothing in alo OS
  makes an exchange.
- **A rename is judged by inode identity, like every other decision here**, so
  the hard-link caveat in `docs/quirks.md` applies unchanged: a second name for
  a file inside a granted folder is inside the bound. `alo-files` refuses to
  *read* such a file, which is where that is answered.
- **This is not a Windows or macOS change.** `alo-bounding` compiles to nothing
  off Linux, by design.
- **Physical hardware acceptance is outstanding.** Every measurement above is
  from WSL2, which `docs/hardware.md` says cannot certify a machine. No *On the
  machine* box is affected by this task and none should be ticked from it.

## Proposed shared-document updates

For the integration owner; I have not edited these four files.

**CHANGELOG.md** — the user-readable description above, under Unreleased.

**ROADMAP.md** — no box changes. The `alo-agentd` line's *The code* clause could
gain: *and since renames were enforced the kernel watches both ends of a move as
well as every open, so a file nobody granted cannot be renamed into a granted
folder and read from there — which was the one thing ADR 0015 named and nothing
had built.*

**docs/autonomy/QUEUE.md** — the entry filed as item 6d in the previous task is
now done and can be marked so, renamed descriptively. Suggested text:

> **Kernel enforcement for file renames.** The gap found by measuring during the
> handle-based moves task: a plain rename of an ungranted file succeeded under
> the boundary, because ADR 0015 named `inode_rename` beside `file_open` and
> only `file_open` had been built. A second BPF LSM program now validates both
> ends of every rename a turn makes. No ADR changed, no grant widened, no new
> map or field: it is the same walk on a second hook. Six tests against the real
> loaded programme, including both refusals, both legitimate directions, and
> data preserved on refusal.

A follow-up worth filing, from the limitations above: **the hooks a bugged verb
could still reach** — `inode_unlink`, `inode_link` and the rest are unwatched.

**docs/autonomy/STATE.md** — reference this report at
`docs/autonomy/updates/kernel-enforcement-for-file-renames.md`. The fact worth
carrying into the journal: the boundary is now two hooks and two pins, and a
loader that cannot attach the second leaves nothing attached at all.
