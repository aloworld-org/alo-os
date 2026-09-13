# How a mapping is reproduced, decided

- Date: 2026-09-13
- Workstream: kernel enforcement, task 21
- Contributor: Claude Code, in `C:\dev\alo-os-claude`
- Status: **the decision is the deliverable; no hook and no `unsafe` was added.**

Nine rows of section 3's filesystem table are closed — reads and writes,
creation, attributes, ownership, size, inode flags, extended attributes, access
lists. One is not, and every task since 12 stepped around it for the same
reason: **a file mapped into memory is read by the processor, not by a
syscall**, so one `mmap` of a descriptor opened before the turn began reaches a
file's contents past every hook this workstream has attached.

The hook that would decide it is `mmap_file`. It has not been written because
it cannot be *reproduced*, and this suite reproduces a gap before closing it.
There is no safe spelling of `mmap` in Rust: the standard library has none, and
both `rustix::mm::mmap` and `memmap2::Mmap::map` are `unsafe fn` at the call
site, which `unsafe_code = "forbid"` refuses in a test as firmly as anywhere. A
worker may not lift that rule. So the task was the decision.

## What the ADR says

[ADR 0030](../decisions/0030-how-a-mapping-is-reproduced.md), proposed, three
options with what each costs:

- **A** — one audited `unsafe` block in one named test fixture, with the plan's
  rule amended to name that file and say why. Buys the reproduction every other
  row has; costs the rule being one file rather than a list.
- **B** — write the hook, measure it once by hand into `docs/quirks.md` with
  the command, the kernel and the date. Buys the rule untouched; costs that
  **a regression there is found by nobody**.
- **C** — drive a pinned component that maps a file it is handed, so the suite
  maps nothing itself. Buys both the rule and a reproduction; costs a test tied
  to somebody else's release notes, and it proves the hook fires for *that*
  program rather than for a turn. Whether any pinned component can be made to
  map a file the test chooses was **not established**, and the ADR says so
  rather than dismissing the option.

It recommends **A**: the rule exists so `unsafe` is rare and reasoned about,
not so the count stays at one, and *reproduce the gap then close it* is what
made the nine closed rows believable.

## The hook's shape, read rather than recited

From `/sys/kernel/btf/vmlinux` on `6.18.33.2-microsoft-standard-WSL2`, today:

    [42910] FUNC 'bpf_lsm_mmap_file' type_id=42909
    [42909] FUNC_PROTO ret_type_id=12 vlen=4
            'file'    →  PTR → STRUCT 'file' size=184
            'reqprot' →  INT 'long unsigned int' size=8
            'prot'    →  same
            'flags'   →  same

Four arguments, and the file is `arg(0)` — no mount mapping in front of it,
unlike the attribute hooks. So `decide_use`'s existing walk from a `struct file`
answers it and **no new deciding function is needed**.

## What it must not decide, written down before anybody writes it

**A mapping with no file behind it is every allocator on the machine**: every
`malloc`, every thread stack, every dynamic loader. A hook that walked one would
ask the question millions of times a second of every process in a turn's
cgroup. The question is asked only of a mapping that names a file, and a null
`file` returns at once. That is not a performance note — it is the difference
between a boundary and a machine that stops working.

## What this did not do

No hook. No `unsafe`. The two maps stay two, because ADR 0029 is proposed and
nothing here anticipates it. `access(2)` stays named rather than closed. The
hook is the task after this one and is not written until ADR 0030's status line
changes — which is the owner's to change.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick; this is a
decision, and a decision is not evidence that anything was built.
**QUEUE.md/STATE.md** — the mapping row names the ADR it waits on.
