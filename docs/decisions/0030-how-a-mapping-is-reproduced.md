# ADR 0030 — How a mapping is reproduced, before a hook decides one

**Status:** proposed — the owner decides, because every option either amends a
rule in `CLAUDE.md`'s neighbourhood or accepts weaker evidence than every other
task in this workstream has produced.
**Date:** 2026-09-13
**Proposed by:** the kernel-enforcement workstream, as task 21 of
`docs/autonomy/kernel-enforcement-plan.md`
**Context:** [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md) (the grant
is enforced by the kernel), [ADR 0015](0015-the-kernel-learns-what-a-turn-is.md)
(no kernel is patched and no fork is kept),
[ADR 0029](0029-what-the-kernel-writes-down-about-a-turn.md) (proposed; the two
maps stay two until it is answered), `Cargo.toml`'s
`unsafe_code = "forbid"`, the plan's rule *no `unsafe` outside
`alo-bounding-kernel`'s one permitted file*, `crates/alo-bounding`

## The question in one line

**A file mapped into memory is read by the processor, not by a syscall** — so
every hook this workstream has attached is stepped around by one `mmap`, and the
hook that would decide it cannot be written, because this suite reproduces a gap
before it closes one and **there is no safe spelling of `mmap` in Rust**.

## What is true today, verified rather than remembered

- **The gap is real and is the last of its kind.** Section 3's filesystem rows
  are closed except this one: tasks 12 to 20 closed reads and writes,
  creation, attributes, ownership, size, inode flags, extended attributes and
  access lists. A mapping of a descriptor opened before the turn began — the
  daemon's own descriptor table, exactly what task 12 closed for `read` and
  `write` — still reaches the file's contents.
- **The hook exists on this kernel and its shape was read today**, from
  `/sys/kernel/btf/vmlinux` on `6.18.33.2-microsoft-standard-WSL2`, not from
  documentation:

      [42910] FUNC 'bpf_lsm_mmap_file' type_id=42909 linkage=static
      [42909] FUNC_PROTO ret_type_id=12 vlen=4
              'file'    type_id=768   →  [768] PTR → [766] STRUCT 'file' size=184
              'reqprot' type_id=1     →  [1] INT 'long unsigned int' size=8
              'prot'    type_id=1
              'flags'   type_id=1

  So the file is `arg(0)` — unlike the attribute hooks, there is no mount
  mapping in front of it — and `decide_use`'s walk from a `struct file` is the
  one the four hooks beside it already make. **No new deciding function is
  needed; the question is the same question.**
- **Every safe road to `mmap` is closed.** The standard library offers none.
  `rustix::mm::mmap` and `memmap2::Mmap::map` are both `unsafe fn` at the call
  site, and `unsafe_code = "forbid"` is a workspace lint that a test cannot
  opt out of. A worker may not lift that rule, which is why this task is the
  decision and not the hook.

## What this must not decide

**A mapping with no file behind it is every allocator on the machine.** Each
`malloc` of any size, every thread stack, every dynamic loader is an anonymous
mapping, and a hook that walked one would ask the question millions of times a
second of every process in a turn's cgroup. `decide_use` is asked **only of a
mapping that names a file**; a null `file` returns at once. That is not a
performance note, it is the difference between a boundary and a machine that
stops working.

## The options

### A — one audited `unsafe` block, in one named test fixture

`crates/alo-bounding/tests/` gains one file whose head is
`#![allow(unsafe_code)]`, containing the smallest `mmap` that can read a
mapping, and the plan's rule is amended to name it as the **second** permitted
file, with the reason and the review it gets written beside it.

- **What it buys:** the reproduction every other task in this workstream has
  produced — the gap shown reaching the contents *before* the hook, and the
  same test flipped to `EACCES` after. A regression is caught by the suite,
  which is the only mechanism that catches one.
- **What it costs:** the rule stops being one file and becomes a list, and a
  list grows. The mitigation is that the amendment names *this* file and this
  reason rather than a category, so a third file is another ADR rather than a
  precedent already set.
- **Law 3** (*done means the full path works*) is served best by this option
  and it is the only one that serves it.

### B — write the hook, measure it once by hand

The hook is written with no committed reproduction; the measurement is made
once at a terminal and written into `docs/quirks.md` with the command, the
kernel version and the date, as this repository already records things it
measured rather than tested.

- **What it buys:** the rule is untouched, and the gap closes now.
- **What it costs:** **a regression there is found by nobody.** Every other row
  in section 3 has a test that would go red; this one would have a paragraph
  that stayed true-looking. It is the weakest evidence in the workstream, in
  the one place that moves a file's contents past a grant.

### C — reproduce it through something already on the machine

A pinned upstream component that maps a file it is handed is driven from the
test, so the suite maps nothing itself. The runtime (`/usr/bin/ollama`) maps
weights; a dynamic loader maps libraries.

- **What it buys:** the rule untouched *and* a committed reproduction.
- **What it costs:** the test is then tied to a rented component's behaviour —
  it passes or fails for reasons that belong to somebody else's release notes,
  which is exactly the coupling *engines are configured, never patched* exists
  to avoid. It also proves the hook fires for **that** program's mapping, which
  is a weaker statement than the one the other tests make about a turn.
- **Not investigated to a conclusion here**, and the ADR says so rather than
  dismissing it: whether any pinned component maps a file *the test chooses*,
  inside a turn's cgroup, on demand, was not established. If the owner prefers
  this road, establishing that is the first work under it.

## The recommendation

**Option A.** The rule exists so that `unsafe` is rare, reviewed and reasoned
about — not so that the count stays at one. A single block in a test fixture,
named in the rule with its reason, is the cheapest thing that keeps this
workstream's evidence standard intact; and that standard — *reproduce the gap,
then close it* — is what has made every one of the nine closed rows
believable. Option B trades the one thing the suite is for. Option C buys the
rule's letter with a test that answers a question about somebody else's
program.

## Consequences if it is accepted

- One test file in `crates/alo-bounding` may use `unsafe` for `mmap` and
  nothing else; the plan's rule names it and says why.
- The hook is **the task after this one**, and is not written until this ADR's
  status line changes.
- The two maps stay two: nothing here anticipates ADR 0029.
- `access(2)` stays named rather than closed — it is not this decision's.

## Consequences if it is rejected in favour of B

- `docs/quirks.md` gains a measurement with a command, a kernel and a date, and
  section 3's mapping row is closed on that basis.
- The workstream's completion sentence must say that one row of it is held by a
  measurement rather than a test, because a reader comparing rows would
  otherwise believe they were evidenced alike.
