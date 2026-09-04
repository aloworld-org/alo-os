# Quirks

Where reality and the specification disagree.

An operating system meets three kinds of reality that no document describes
correctly: hardware and firmware, applications being driven through their own
automation, and pinned upstream engines behaving unlike their manuals. When you
lose an afternoon to one of them, write it down here. The next person should
inherit the knowledge, not the debugging session.

## How to write an entry

One entry per quirk, newest first within each section. Every entry says: **what
it is, what version, what actually happens, what we do about it, and the date.**
A quirk with no version and no date is a rumour.

Keep the accommodation and the reason together. Six months from now the code
will look wrong to somebody, and this file is the only thing standing between
them and reintroducing the bug.

The rule this file serves: **strict in what we do, tolerant in what we accept.**
We behave correctly; we cope with hardware and applications that do not.

---

## Hardware and firmware

### `struct file`'s `f_path` is inside an anonymous union, and a search over named members does not find it
**Version:** `6.18.33.2-microsoft-standard-WSL2`, measured 2026-09-04 by reading
`/sys/kernel/btf/vmlinux` on the machine the boundary would not load on.
**Behaviour:** the kernel's type information describes `struct file` with
nineteen members, and three of them **have no name**. `f_path` is not one of the
nineteen — it is a member of the unnamed union that is, sixty-four bytes in,
sharing its bytes with a second member called `__f_path`:

```
struct file  vlen 19  size 184
  ...
  f_cred     at 48
  f_owner    at 56
  <unnamed>  at 64   union { struct path f_path; ... __f_path; }
  <unnamed>  at 80   union { struct mutex f_pos_lock; u64 f_pipe; }
  f_pos      at 112
```

This is ordinary C — an anonymous struct or union's members belong to the
structure around it — and the format keeps the source's shape rather than
flattening it. What makes it a trap is the failure: `alo-bounding`'s reader
searched the named members only, found nothing, and refused to impose the
boundary with *this kernel has no `file.f_path`, so the boundary has nowhere to
look*. **That sentence is a true statement about the search and a false one about
the kernel**, and it points whoever reads it at their machine rather than at our
code. Asked directly, the same BTF is 6,677,359 bytes and contains `f_path`,
`f_inode`, `dentry`, `d_name` and `mnt_root`; `bpf_lsm_file_open` is in
`kallsyms`. Everything the message doubted was there.

Kernel 6.6 kept `f_path` as a plain member, so this appeared as a kernel upgrade
breaking a boundary that had never run.
**Our response:** `crates/alo-bounding/src/btf.rs` implements the rule rather
than the case — a member with no name is walked into, and what is found inside
it comes back at the outer member's offset plus its own. The descent is bounded
by the same `PATIENCE` the width lookup uses, because this file is read from
`/sys` rather than written by us, and it is never itself an answer: asking for
`""` finds nothing. The fixture in `testing.rs` now keeps `f_path` where 6.18
keeps it, so every test of the reader is a test of the walk into it, and a fourth
fixture whose anonymous member leads back to the structure it is in asserts the
bound. Nothing was special-cased for `file` or for a version.
**Date:** 2026-09-04

### The BPF LSM is compiled into the WSL2 kernel and does not start
**Version:** `6.6.87.2-microsoft-standard-WSL2`, Ubuntu under WSL2 on Windows 11,
measured 2026-09-03 by reading the kernel's own config and its own list of
running security modules.
**Behaviour:** the kernel has `CONFIG_BPF_LSM=y`, so every account of it that
stops there says the BPF LSM is available. It is not. `CONFIG_LSM` is
`"landlock,lockdown,yama,loadpin,safesetid,integrity,selinux,apparmor,tomoyo"`
with no `bpf` in it, `/proc/cmdline` carries no `lsm=` parameter to replace that
list, and the kernel's own answer — `/sys/kernel/security/lsm` — is
`capability,landlock,yama,safesetid,selinux`. A security module that is not in
that list never registered its hooks, so nothing it would have decided is asked
of it. **Compiled in and started are two different questions**, and only the
second one matters.

Two smaller things go with it, both of which cost time before the answer
appeared. `securityfs` is **not mounted** on this kernel, so
`/sys/kernel/security/lsm` reads as a missing file rather than as an answer until
`mount -t securityfs securityfs /sys/kernel/security` is run — an empty result
here means *you have not asked yet*, not *no modules*. And `bpftool` is not
installed in the Ubuntu image, so `bpftool feature probe` returns nothing at all
and exits `0`, which reads exactly like a clean probe that found no LSM support.
Neither absence is an answer; both look like one.
**Our response:** the measurement is the finding, and no work around it was
attempted. `docs/autonomy/QUEUE.md` items 26 and 27 — the whole of ADR 0015 —
are blocked on a kernel that starts the BPF LSM, and `docs/hardware.md` now
states the requirement as two checks in order rather than one, because checking
only `CONFIG_BPF_LSM` is how this kernel passes. WSL2 can be given
`lsm=…,bpf` through `kernelCommandLine` in a `.wslconfig`, and the build loop
deliberately did not: that is a change to somebody's own machine, made outside
this repository, that restarts every distribution running on it, and it is the
machine owner's to make rather than a loop's.

**Resolved on this machine, 2026-09-04, by the owner making that change.**
`kernelCommandLine = lsm=capability,landlock,yama,safesetid,selinux,bpf` in
`.wslconfig`, then `wsl --shutdown`. The kernel now answers
`capability,landlock,yama,safesetid,selinux,bpf`, and that survives a cold boot.
Three things came out of doing it that the first measurement could not show:

- **`lsm=` replaces the built-in list, it does not add to it.** `CONFIG_LSM`
  still reads `"landlock,lockdown,yama,loadpin,safesetid,integrity,selinux,apparmor,tomoyo"`
  and is simply not what this kernel used. Every module that must keep enforcing
  has to be named again on that line: `lsm=bpf` alone would have started the BPF
  LSM and silently stopped the five that were already running. A boot parameter
  that turns a protection on can turn four others off in the same breath.
- **`securityfs` is not mounted at boot, and `systemd=true` does not mount it.**
  This distribution has systemd enabled and the mount was still absent, so it
  went into `/etc/fstab`, which WSL does process. Without that step the fix looks
  like it failed, because the file that would report success is the one missing.
- **The remedy `docs/hardware.md` predicted is now measured rather than
  supposed** — a boot parameter, on the same kernel, with no kernel of our own.

The certified machine inherits the requirement and not this workaround: what it
needs is a kernel that boots with `bpf` in its active list, however its image
arranges that.
**Date:** 2026-09-03; resolved 2026-09-04

### The kernel starts the BPF LSM and still cannot attach a program to it
**Version:** `6.6.87.2-microsoft-standard-WSL2`, Ubuntu under WSL2 on Windows 11,
measured 2026-09-04 by attaching the programme in `crates/alo-bounding-kernel`
and then reading `dmesg`.
**Behaviour:** the two checks in `docs/hardware.md` both pass — `CONFIG_BPF_LSM=y`
and `bpf` in `/sys/kernel/security/lsm` — and the attach never returns. The
thread goes into **uninterruptible sleep in `bpf_trampoline_get` and stays
there**: it cannot be killed, `SIGKILL` leaves the process a zombie with that
thread still in the kernel, and every later BPF attach on the machine blocks
behind the same mutex. Nothing in userspace reports anything; there is no error
because there is no return.

The kernel says why, in its own log, every ten seconds:

```
tasks_rcu_exit_srcu_stall: rcu_tasks grace period number 13 (since boot)
  gp_state: RTGS_POST_SCAN_TASKLIST is 634853 jiffies old.
Please check any exiting tasks stuck between calls to
  exit_tasks_rcu_start() and exit_tasks_rcu_finish()
```

Attaching a BPF LSM programme builds a trampoline, and that waits on an
RCU-tasks grace period. On this machine grace period 13 has never completed: at
the first stall message it was already 634853 jiffies old, which at
`CONFIG_HZ=250` is about forty-two minutes, on a machine that had been up for
forty-three. **The grace period stalled about a minute after boot and more than
an hour before any of this repository's code ran**, so the boundary did not
cause it and cannot avoid it — a `synchronize_rcu_tasks()` that will never
return is a `synchronize_rcu_tasks()` that will never return.
**Our response:** the measurement is the finding, and it is a **third**
requirement that neither ADR 0015 nor `docs/hardware.md` had: a kernel whose
RCU-tasks grace periods complete. It is the same shape as the two before it —
`CONFIG_BPF_LSM=y` is true and useless on a kernel that does not start the
module; a started module is true and useless on a kernel that cannot attach to
it — and it is worse than both, because the failure is a hang rather than an
answer. The check is one line and belongs before any attach:
`dmesg | grep -c tasks_rcu_exit_srcu_stall`, where anything but zero means no
BPF LSM, fentry or fexit programme will attach on that machine until it is
rebooted.

`crates/alo-bounding/tests/the_kernel_refuses.rs` was written, is correct, and
**has never run**: on this machine it hangs at the first attach, and no claim
about the kernel refusing anything is made anywhere in this repository. Queue
item 26 is not ticked.

**Reboot tested, and it reproduces — this kernel is out.** The machine was
restarted and measured again the same day. The stall is **deterministic, not bad
luck**: same grace period number (13), first stall message at **31 seconds** of
uptime with the period already 2507 jiffies (10 seconds at `CONFIG_HZ=250`) old,
so it stalls about **21 seconds after boot**, before anything of ours can run.
The attach was attempted again under a deadline and hung again, leaving the same
unkillable remnant. Two boots, same result on that kernel version.

**Fixed by a kernel upgrade the same day, and the scope of the finding was
wrong.** `wsl --update` took WSL from 2.6.1.0 to 2.7.12.0 and the kernel from
`6.6.87.2` to `6.18.33.2`. On the new kernel the stall count is **zero** past the
blind window, and the attach that had hung twice now returns in **0.08 seconds**.
So the sentence this entry originally carried — *WSL2 cannot host a BPF LSM* —
was **too broad by one word**: it was true of that kernel and false of WSL2. A
finding measured twice on one version is still a finding about that version, and
naming the platform instead of the build is how a temporary fact becomes a
permanent belief. The requirement in `docs/hardware.md` is unchanged and is what
should be quoted; this entry is the worked example of a kernel that failed it.

The `.wslconfig` `lsm=` line and the `securityfs` entry in `/etc/fstab` both
survived the upgrade, and the new kernel starts `bpf` — with `ima` alongside it,
which the old one did not.
**Superseded:** 6.18.33.2 passes all three checks.

One correction worth keeping, because it nearly became a rule. The entry above
reads as though the first stall message arrives about forty minutes in, which
would give the `dmesg` check a forty-minute blind window and make a zero
meaningless on a fresh machine. That is wrong: messages repeat every ten seconds
from the moment of the stall, and the forty-three-minute figure was simply the
**oldest message still in the ring buffer** when it was read, not the first one
emitted. The real blind window is about **thirty seconds**. So the one-line check
in `docs/hardware.md` is sound, with one qualification: *ask it on a machine that
has been up for more than a minute.*
**Date:** 2026-09-04, reboot-tested the same day

<!--
### <Machine or component> — <one-line summary>
**Version:** firmware / kernel / driver version the behaviour was seen on
**Behaviour:** what actually happens, as opposed to what is documented
**Our response:** what we do, and why this rather than something else
**Date:** YYYY-MM-DD, and who saw it
-->

## Application automation

Applications driven through adapters (`docs/contracts/app-adapters.md`) change
their automation surfaces between versions, sometimes silently. This is where
that gets recorded: which version, what changed, and what the adapter now does.

_(no entries yet)_

<!--
### <Application> <version> — <one-line summary>
**Mechanism:** api | accessibility | dbus | synthetic
**Behaviour:** what the API or the accessibility tree actually does
**Our response:** what the adapter does about it
**Date:** YYYY-MM-DD
-->

## Pinned engines

The kernel, Mesa, systemd, the model runtime and the fine-tuning stack are
configured, never patched. When one of them behaves unlike its documentation,
the accommodation lives in our configuration and the reason lives here.

An entry here that says "we patched it" is a bug in the process: a source patch
to an engine requires an ADR first.

### bpf-linker 0.11.0 — the LLVM it was built against must be the one rustc emits
**Version:** `bpf-linker` 0.11.0, `rustc` nightly, Ubuntu 26.04, found
2026-09-04 by building `crates/alo-bounding-kernel`.
**Behaviour:** `bpf-linker` does not link objects the way a linker does — it
reads the LLVM bitcode `rustc` produces and runs LLVM's own passes over it. So
the LLVM it was built against has to be the LLVM the compiler emits, and when it
is not, **nothing says so**. A programme with a single BPF map in it makes the
linker die of a segmentation fault:

```
error: linking with `bpf-linker` failed: signal: 11 (SIGSEGV)
  PLEASE submit a bug report to https://github.com/llvm/llvm-project/issues/
  1. Running pass "sroa<modify-cfg>" on function "file_open"
```

Every part of that message points somewhere else. It names LLVM's bug tracker,
so it reads as an LLVM bug; it names our own function, so it reads as our code;
and it names an optimisation pass, so it reads as an optimiser problem. The
actual cause is two version numbers that are never printed together. A
programme with no map in it links perfectly, which is what makes the first hour
of this go into the code rather than into the toolchain.
**Our response:** the version is pinned in the repository rather than left to
whichever nightly a machine has.
`crates/alo-bounding-kernel/rust-toolchain.toml` names the compiler, and
`crates/alo-bounding/build.rs` starts the nested build **in that directory** so
the file is what decides — naming a channel on the command line would silently
overrule it. Whoever builds alo OS needs a `bpf-linker` built against the LLVM
that compiler emits: `rustc +<channel> -vV` says which, and
`cargo install bpf-linker --no-default-features --features llvm-<n>` is how it is
built against it. `docs/autonomy/LOOP.md` has what that took on this machine.
**Date:** 2026-09-04

### ureq 3.4.0 — `send_json` puts a pretty-printed body on the wire
**Behaviour:** the request body is `serde_json::to_writer_pretty`-shaped —
indented, with a newline after every field — where the obvious assumption is the
compact form. It is documented nowhere either way. Observed on a real socket by
`alo-asking`'s stub, which reads what actually arrives.
**Our response:** nothing is configured, because nothing is wrong: a provider
parses either, and the few hundred extra bytes on a question that is already
kilobytes of somebody's text are not worth a hand-built body. What changed is
the **test**: `alo-asking` asserts on the request body *parsed* rather than on
its text, so it says *these three fields and nothing else* — which is the
promise worth keeping (nothing of the person's leaves except the question) and
is also the assertion that does not break the next time ureq changes its
whitespace.
**Upstream:** not reported; it is not a defect.
**Date:** 2026-09-03

### Ollama — a question has two APIs, and 404 means the model rather than the address
**Version:** Ollama's documented HTTP API as of 2026-09-03. **Not observed
against a running Ollama on any machine**; `alo-models`' tests drive a stub on a
real socket, and a run against the real runtime is owed with the rest of the
hardware verification (`ROADMAP.md` carries it as the model stack's machine
half).
**Behaviour:** the runtime answers questions two ways — its own `/api/chat`, and
an OpenAI-compatible `/v1/chat/completions` that speaks the shape a hosted
provider does. They are not the same reply: the native one puts the answer at
`message.content` and the compatible one at `choices[0].message.content`. And on
either, a model the runtime does not hold comes back **404 on an endpoint that
exists**, which at the protocol level is indistinguishable from an address that
is wrong — the same ambiguity a hosted provider has, recorded below.
**Our response:** `ollama.rs` uses `/api/chat`, the runtime's own. ADR 0006 says
Ollama's API is not our API and that one file may know what it is; using the
surface that imitates somebody else's would make the adapter's shape a guess
about a provider rather than a fact about the runtime, and the two could drift
apart in a release without anything here noticing. The 404 becomes
`RuntimeError::NotInstalled`, which names what was needed rather than what to
fix, and a person who typed the endpoint wrongly reads *that model is not
installed* — wrong, and wrong in the direction that costs them the least,
because a runtime alo OS ships is not an address anybody typed.
**Upstream:** not reported; both are documented behaviour.
**Date:** 2026-09-03

<!--
### <Engine> <version> — <one-line summary>
**Behaviour:** what it does, versus what is documented
**Our response:** the configuration we apply, and why
**Upstream:** issue link if reported
**Date:** YYYY-MM-DD
-->

## Filesystems and paths

A grant is over a place, and a path is only a name for one. Where the two come
apart, a capability check can be correct and still be wrong — so this is where
that gets written down rather than discovered.

### The device number `stat` reports is not the one the kernel keeps
**Version:** Linux, any; found 2026-09-04 while writing `crates/alo-bounding`.
**Behaviour:** `stat` reports a file's device in `st_dev`, and the kernel holds
the same device in `super_block->s_dev`. **They are different packings of the
same two numbers**, and nothing anywhere says so:

```
stat reports    minor & 0xff | major << 8 | (minor & ~0xff) << 12
the kernel has  major << 20 | minor
```

For an ordinary partition at major 8, minor 2, one is `0x802` and the other is
`0x800002`. A comparison between them does not fail loudly — it simply never
matches, so a boundary keyed on a device number would find every file to be
outside every grant while looking perfectly healthy, and the code doing it reads
like the obviously correct code.
**Our response:** the conversion is in one function,
`alo_bounding::as_the_kernel_keeps_it`, with the two packings written above it,
and it is the only place a device number crosses between the two. The test that
would catch a mistake in it is not its own unit test — it is
`a_turn_granted_a_folder_opens_a_file_inside_it`, because a wrong conversion
refuses the granted file rather than allowing an ungranted one, which is a
failure in the safe direction and therefore the failure nobody notices.
**Date:** 2026-09-04

### `Path::is_absolute` answers about the host, not about the path
**Version:** Rust 1.97 `std`, seen 2026-09-03 in `alo-saying` on Windows 11
26200 and Ubuntu under WSL2
**Behaviour:** `Path::new("/usr/share/alo/translations").is_absolute()` is
`true` on Linux and **`false` on Windows**, because a Windows absolute path
needs a drive or a UNC prefix and this one has neither. It is documented
behaviour and it is right — the question `std` answers is *would this resolve
without a working directory on the machine you are running on* — but it is not
the question a test about a path alo OS ships asks.
**Our response:** where a constant is a path on the machine alo OS runs on
rather than a path on the host the tests run on, the test is written against the
text (`starts_with("/usr/")`) and says so. Reaching for `is_absolute` gives a
test that passes on the loop's Linux half and fails on its Windows half, having
found nothing wrong with anything. The same caution applies to `Path::join`,
`parent` and `components` on any path this repository writes down for a machine
it does not run the tests on.
**Date:** 2026-09-03

### A zip has nowhere to say which clock its timestamps came from
**Version:** the zip format as every reader implements it; seen 2026-09-02 in
`alo-files`, against Windows 11 26200's own reader
**Behaviour:** a zip keeps each file's time as a DOS date and time, which
carries no timezone and is **conventionally the local time of whoever wrote the
archive**. `std` cannot say what this machine's offset from UTC is, and every
crate that can does it either through a dependency whose local-offset lookup is
unsound in a threaded process or through code this repository forbids.
**Our response:** the moment written is **UTC**, consistently, and it is
documented where the archive is made rather than left to be discovered. A reader
on a machine two hours ahead of UTC shows a file archived at 20:04 as 18:04.
Seconds are also kept in two-second steps, which is the format and not us. The
alternative — a guessed offset, or a dependency to find the real one — would be
wrong more interestingly rather than less often.
**Date:** 2026-09-02

### Resolving a path does not defeat a hard link
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** `alo-files` resolves every path a verb names and asks the grants
about where it really leads, which stops a symbolic link out of a granted
folder. A **hard** link is not a link in that sense: it is a second real name
for the same file, so a hard link inside a granted folder to a file that also
lives outside it resolves to the granted name and passes the check.
**Our response:** nothing in the path layer, because there is nothing honest to
do there — the granted name genuinely is a real name for that file. Making a
hard link needs write access to the granted folder and read access to the
target, so it is not a way *in*; it is a way for somebody who can already write
to a granted folder to widen what an agent may read. It is documented here, in
the contract and in `alo-files`, and the answer if it ever matters is a policy
about link counts at the moment of opening, not a cleverer path comparison.
**Date:** 2026-09-02

### A path checked and then opened by name can change in between
**Version:** every filesystem alo OS will run on; seen 2026-09-02 in
`alo-files`
**Behaviour:** the real path is resolved, the grants permit it, and then the
file is opened by that name. Anything with write access to a folder on the way
can swap a link in between the two.
**Our response:** the check is where it can be, and the fix is not another
check. Whatever opens the file holds on to *what it opened* rather than
resolving the same name twice. The acting half in `alo-files` does as much of
that as `std` allows — a file is opened once and its size is asked of the open
handle rather than of the name, and nothing resolves a path a second time — and
what `std` does not allow is the rest: opening relative to a directory handle
(`openat`) and renaming without replacing (`renameat2` with `RENAME_NOREPLACE`)
are Linux calls with no portable spelling, so a destination is checked and then
renamed onto, with a gap between the two. Narrowing it is item 6b in
`docs/autonomy/QUEUE.md`, written down rather than left to be rediscovered.
**Date:** 2026-09-02, extended 2026-09-02 by the acting half

### Windows returns a path spelled differently from the one it was given
**Version:** Windows 11 26200, Rust 1.97 `std::fs::canonicalize`
**Behaviour:** canonicalising `C:\Users\x\Temp\Invoices` gives
`\\?\C:\Users\x\Temp\Invoices`. The two are the same folder and compare as
different paths, component by component, because the verbatim prefix is a
component.
**Our response:** none in the comparison, which is right to be exact — a grant
that matched loosely would match more than the person picked. **A grant is made
over a resolved path**: the folder a person picks is resolved when they pick it,
so both sides of every later comparison are spelled the way the machine spells
them. Written into the contract and asserted in `alo-files`' integration test,
which grants a resolved folder for exactly this reason.
**Date:** 2026-09-02

## Clocks and moments

A record is evidence about when something happened, so what a moment means when
it is written down, read back and compared is this section's subject.

### `SystemTime` walks back past 1970, and how far is the platform's
**Version:** Rust 1.97 `std::time::SystemTime`, seen 2026-09-03 in
`alo-keeping` against Windows 11 26200
**Behaviour:** a retention rule is naturally written as *keep anything after
`now - 30 days`*, and `SystemTime::checked_sub` is the obvious way to say it.
On Windows a `SystemTime` is counted from 1601, so subtracting thirty days from
a machine whose clock says it is the first minute of 1970 answers with a moment
in **1969** rather than `None`. On a platform where the representation is a Unix
`timespec` the same call can answer `None` instead. Both are correct for the
type; they are not the same boundary.
**Our response:** the window is measured **from the epoch forwards**, not from
`now` backwards. `Keeping::oldest_kept` asks how far `now` is past the epoch,
subtracts the window from *that*, and answers `None` when it does not reach —
so a boundary before 1970 is *nothing is removed*, identically on every
platform. It matters because the case it covers is a machine whose clock is
wrong, and a wrong clock must never be a way to empty a record. The test that
says so is `a_wrong_clock_never_removes_more`, and it was the failing test that
found this.
**Date:** 2026-09-03

### A record is replaced while it is open for appending, and Windows allows it
**Version:** Rust 1.97 `std::fs::rename`, Windows 11 26200; seen 2026-09-03 in
`alo-keeping`
**Behaviour:** shortening a record writes the replacement beside the old file
and renames it over. On Windows that is `MoveFileEx` with
`MOVEFILE_REPLACE_EXISTING`, and replacing a file another handle has open is
the classic way to get *access is denied*. It succeeds here, because `std`
opens files with `FILE_SHARE_DELETE` among the share flags — which is `std`'s
choice rather than a documented guarantee of the platform.
**Our response:** the rename happens with the writer's own append handle still
open, and the handle is **reopened immediately afterwards** — an old handle
goes on writing into a file that is no longer the record, which is a lost entry
rather than an error. Shortening is therefore a method on the writer taking
`&mut self`, so nothing can append during it and nothing else is expected to be
holding the record open. If a filesystem ever refuses the replace, the answer
is to close the handle before renaming and not to copy over the old file in
place: nothing is removed until the replacement is whole on the disk.
**Date:** 2026-09-03

### A record whose folder has been removed goes on accepting writes
**Version:** Rust 1.97 `std::fs`, Windows 11 26200; seen 2026-09-03 in
`alo-turn`
**Behaviour:** a turn that cannot write down what it did stops doing anything
else, and the integration test for that wanted a real disk to refuse a real
write. Removing the folder the record lives in does not do it: on Windows
`remove_dir_all` **succeeds** with the record file open — `std` opens files
with `FILE_SHARE_DELETE` — and the open handle then goes on accepting writes
and syncing them, into a file no longer reachable by any name. The write does
not fail, so the turn never learns anything is wrong. There is no portable way
to make a filesystem refuse a write to a handle it has already given out.
**Our response:** the closing is tested against a `Kept` that refuses
everything (`alo-turn`'s
`a_turn_that_could_not_write_something_down_does_nothing_else`), and the
integration test asserts the half a real disk *can* answer: that every entry is
on the disk before the door that made it answers. The rest of it —
`NotKept::NotAddedTo` really arriving from a full or failing disk — is owed
with the hardware verification. It is the same share-flag behaviour as *a
record is replaced while it is open for appending* above, met from the other
side.
**Date:** 2026-09-03

## Languages and counting

A sentence with a number in it is the one string that cannot be translated
line for line. Where what a plural form is called and what it actually covers
come apart, write it here — because the person who would notice is the person
reading that language, and there is nobody here who reads all 24.

### A plural form's name says nothing about which numbers it covers
**Version:** CLDR cardinal rules, `common/supplemental/plurals.xml` from
`unicode-org/cldr`, read 2026-09-02. Not a disagreement with CLDR — CLDR is
right — but with what the names lead a reader to assume.
**Behaviour:** three assumptions that all look safe and are all wrong. **Every
language has `other`:** Polish does not, for a whole number — its `one`, `few`
and `many` cover every integer between them, and CLDR's Polish `other` has
decimal samples only. A file offering a Polish translator `one` and `other` asks
them for one sentence nothing will ever show and leaves out the two that most
numbers take. **`one` means one:** Croatian's `one` covers 1, 21, 31 and 101;
French's covers 0 as well as 1; Latvian's `zero` covers 0, 10, 11 and 20 alike.
A translation that spells the number out — *jedna datoteka* — is then shown to
somebody with twenty-one files. **A form is picked by the number:** it is picked
by the number *and the language*, so English's forms cannot be used to look up a
Polish sentence.
**Our response:** `alo-strings`' `cldr.rs` holds the rules as code with each
CLDR condition quoted beside the arm it became, and three things are refusals
rather than conventions. A translation into a form its own language never uses
is refused, naming the forms it does use. A form may leave the number out only
where `names_one_number` says exactly one whole number takes it. A countable
string translated into a language whose rules are not in the table is refused
outright, in words addressed to whoever is contributing that language — nothing
falls back to English's two forms, because a sentence wrong for most numbers in
a language nobody here reads is worse than one that has not arrived.
**Date:** 2026-09-02

### Half the keys on a keyboard are not printed with a word anybody translates
**Version:** physical keyboard layouts, EU national variants, observed 2026-09-02.
**Behaviour:** *what a key is called* looks like one list of strings and is two.
`Q`, `7`, `,` and `F1` are printed identically on every keyboard sold in the
union, and translating them is not translation at all — it names a **position**,
which is the model `alo-shortcuts` exists to reject, since `Super+Q` on a French
keyboard is the key marked Q and not the one where an English keyboard has Q. The
other sixteen print a *word*, and it is a different word almost everywhere: a
German keyboard prints **Entf** for Delete, **Einfg** for Insert, **Pos1** for
Home, **Strg** for Ctrl and **Bild ↑** for Page Up; a French one prints **Maj**
for Shift. A shortcuts panel translated from one English list would either name
keys that are not on the keyboard in front of the person, or invite a translator
to render `Q` as `Й`.
**Our response:** the two kinds are different questions in the code.
`Key::mark` answers for the fifty-three that print a mark and is not a string at
all; `Key::said` answers for the sixteen that print a word, each declared in
`alo-shortcuts`' `words` with a note naming what a keyboard in another country
prints; and `Key::shown` is what a panel draws for either. Declaring all
sixty-nine was the alternative and is worse twice over: it hands a translator
forty-one rows reading `A`, `B`, `C`, and it makes `Strings::unanswered` — *what
a release note has to count* — report fifty-three strings nobody should ever
translate.
**Date:** 2026-09-02

### A machine cannot punctuate a list it assembled
**Version:** Greek orthography; CLDR list patterns, `common/main/*.xml`, not
implemented here.
**Behaviour:** a sentence that names two or more things has to join them, and
the joining is not punctuation a program can pick. Greek writes `;` where
English writes a question mark and `·` where English writes a semicolon, so a
list joined with `"; "` reads as a row of questions to the people it is for. The
conjunction before the last item is a word — *and*, *und*, *et* — that would
have to be its own string, placed by a machine that does not know the sentence.
**Our response:** no sentence in this repository joins a list. Where one thing
is named it goes in a gap, as `alo-shortcuts`' *{chord} is already {action}*
does; where two or more are, the sentence says so and the things are handed over
to be drawn as rows — `Clash::said` names the chord and `Clash::actions` hands
over what wants it, each said in the reader's own language. If a sentence ever
genuinely needs a list inside it, the list patterns are CLDR data like the plural
rules and are read rather than recalled.
**Date:** 2026-09-02

### A deserialiser is required to have a sentence and has nobody to ask for one
**Version:** `serde` 1, `#[serde(try_from = "…")]`, observed 2026-09-02.
**Behaviour:** every value in this repository that a settings file holds is
checked again on the way in, because a settings file is a thing a person edits —
a colour, a screen's name, a rotation, a schedule, a text size, a time of day, a
key combination. `serde` implements that with `try_from`, and it requires the
error to have a `Display`, because it turns it into a message with
`de::Error::custom`. That is exactly the thing our rule forbids: a `Display` on a
user-facing refusal is an English sentence one `to_string()` away from a screen.
And the deserialiser is the one caller that genuinely cannot obey the rule the
other way either — it is handed a value and a format, never the language the
person in front of the machine reads, and there is no argument to give it one.
**Our response:** what a refusal writes at that point is the **key** of the
string rather than the string. `alo-appearance`'s `NotRead` is that, shared by
its six deserialisers, and `alo-shortcuts`' `Chord` has a private one of its own
from item 9c. Whoever reports a settings file that did not read looks the key up
and shows the same words a settings panel shows for the same refusal — one
rendering, in the reader's own language, rather than an English line in a log
beside a translated line on a screen. The refusal itself is unchanged: the same
files are refused as before, and `said(&Strings)` is still the only road to
words. What is given up is `std::error::Error` on ten types that were never
errors a programmer handles.
**Date:** 2026-09-02

### Two gaps in a translated sentence arrive in the language the code was written in
**Closed by item 9g on 2026-09-03.** Kept because the shape of the mistake is
worth recognising again, and because the fix cost a public surface change.
**Version:** `alo-capability` at item 9e, observed 2026-09-02.
**Behaviour:** a translated sentence is only as translated as what goes into
its gaps, and two here came from somewhere that had not moved.
`capability.call.missing` — *{verb} needs {argument} — {purpose}* — filled
`{purpose}` from what the verb was declared with, which is the source string
rather than the reader's; the crate that declared the verb had the translation,
and the crate that refuses the call did not, because a `Verb` carried the
declaration and not a key. `capability.answer.lapsed` quotes the approval
sentence, which `alo_capability::Call` rendered at the moment the call was made
and kept as a string. So a German machine could read a German sentence with an
English clause inside it, which is exactly the failure `alo-appearance` closed
for colour names in item 9d.
**Our response while it stood:** the note on each of those two words said so, in
the words a translator needs, so nobody spent an afternoon looking for the
string that would fix it. **What closed it:** item 9g. A verb is declared from
`alo_strings::Word`s, a `Call` carries the key of its sentence and the values
that fill it, and `CallError::Missing` carries the key of the argument's
purpose — so both gaps are looked up with the same vocabulary as the sentence
around them. It was never worked around, because working around it would have
meant a second copy of a declaration, and one string rather than two that agree
is the rule the whole 9-series is built on.
**Date:** 2026-09-02, closed 2026-09-03

### A translated error cannot be a `std::error::Error`
**Version:** Rust 1.x, `std::error::Error: Debug + Display`, met again at item
9f on 2026-09-02 and at item 9h on 2026-09-03.
**Behaviour:** `std::error::Error` requires `Display`, and `Display` takes no
argument but a formatter — so a type that can only say what it is when it is
handed the reader's language cannot implement it. Everything downstream of that
trait goes with it: `?` into a `Box<dyn Error>`, `#[from]`, `anyhow`, and the
`{e}` a programmer writes without thinking. It is the same collision the
deserialiser entry above describes, met from the other side, and item 9f is
where it reached a type in a **public trait's** signature — `ModelRuntime`
returns `RuntimeError`, and third parties implement `ModelRuntime`.
**Our response:** the types a person reads give up `Display` and answer
`said(&Strings)`, and the ones a programmer reads keep it. The line between
them is *who is holding the machine when this appears*: `CatalogueError`
refuses the catalogue this repository ships, `VerbError` refuses a verb
declaration, `alo-shortcuts`' `DefaultsError` refuses a release's own defaults —
all read by whoever is fixing the thing that failed, so all still English and
still `std::error::Error`. What an adapter author gives up is `?` into a boxed
error, and what they get is a refusal their user can read; `RuntimeError`'s own
documentation says so where they will look. Two doctests in `alo-models` had to
drop their `?` for this reason and were re-checked afterwards, because a
`compile_fail` that starts failing on a missing conversion has stopped testing
what it was written for.

Item 9h met it in the place where it costs the most and is still worth paying:
`alo_egress::NotPermitted` is what `Indicator::beginning` returns, so an egress
refusal no longer arrives as an `Error` a caller can `?` into a box. The person
holding the machine when that appears is the owner watching the indicator, so
the refusal gives up `Display` like the rest — and three doctests, one of them
in `alo-record`, dropped their `?`. The `compile_fail` beside them was
re-checked outside the doctest harness and still fails on **E0624, associated
function `new` is private**, which is what it was written to test.
**Date:** 2026-09-03

## Models

Open-weight models in the catalogue have their own personalities: refusing
formats they claim to emit, ignoring stated context limits, or answering in the
wrong language. Where a model in the catalogue misbehaves in a way that affects
the agents, record it here with the exact model and quantisation — "it was fine
for me" is usually a different quantisation.

### What `drives_verbs` measures, and the two things it does not say
**Version:** `alo-driving` as of 2026-09-03. **Not run against any real model
on any machine** — the fixed set, the scoring and the grade are built and unit
tested, and every entry in `data/catalogue.toml` says `not-measured` because of
it. Running it is owed alongside the rest of the hardware verification.
**Behaviour:** the measurement puts ten requests to a model and scores each
answer through `alo_protocol::FromAnAgent` and `alo_capability::Verbs::call` —
the daemon's own door and the same validation a real turn does. Two consequences
that a grade does not carry on its face.

**It is asked in English.** The prompt is not a string a person reads, so it is
not an `alo_strings::Word` and this crate declares no vocabulary; what follows
is that a grade says how a model drives the verbs *when it is asked in English*.
A model asked in Latvian may do worse, and nothing here would know. Measuring in
twenty-four languages is a real question, it is a different one, and pretending
the current grade answers it would be the kind of claim `Driving::NotMeasured`
exists to prevent.

**The envelope is part of what is measured.** A model has to produce the whole
message — `{"format":1,"asks":{"read":{…}}}` — rather than a bare verb and
arguments. That is deliberate: a lighter shape invented for the measurement
would be a second parser for one syntax, which is the failure item 9g removed
one level down. But a real agent composes the envelope around whatever its model
emitted, so **a model wrapped by such an agent may drive the verbs better than
its grade says**.

**Our response:** both errors fall the same way — toward not giving a model the
agent — which is the direction every other decision about this property takes,
and a machine that refuses says so in words naming the two places ADR 0008
leaves open rather than substituting one. Neither is worked around. Whoever
raises the measurement's coverage changes the fixed set, and a changed set means
every grade in the catalogue is stale: `alo_driving::THE_SET` is where that
version lives, and it is a `&'static` array in the source so it cannot drift
quietly.
**Date:** 2026-09-03, iteration 42.

## Providers and their APIs

A provider somebody adds themselves is a service nobody here operates, behind an
address nobody here chose. Where the convention every provider claims to follow
turns out to be followed differently, record it here — with what the evidence
actually is, because a provider's documentation is not a run against it.

### There is no status that means "the account has run out", so there is a list
**Version:** the OpenAI-compatible convention and its largest publishers' error
documentation as of 2026-09-03. **Not observed against any live provider**;
`alo-asking`'s tests drive a stub on a real socket, and an account that has
really run out is owed alongside the rest of the hardware verification — it is
also the one condition in this file nobody can produce on demand without letting
a real balance empty.
**Behaviour:** HTTP has had `402 Payment Required` since 1997 and the large
providers do not send it; the services that do are gateways and resellers. What
the publishers document instead is an ordinary status carrying a
machine-readable name: `429` with `insufficient_quota`, `403` with a billing
name for an account they have stopped serving. So the two statuses a person
most needs told apart mean two things each — `403` is *your key was refused* or
*your account is empty*, `429` is *slow down* or *your account is empty* — and
the status alone cannot say which. Worse, the names collide across publishers in
the wrong direction: Google's `RESOURCE_EXHAUSTED` and everybody's
`rate_limit_exceeded` sound like running out and mean asking too fast.
**Our response:** `alo-asking`'s `ran_out.rs` holds a closed list of the
identifiers that mean an account has nothing left **and mean nothing else**, and
`openai.rs` reads the body of a `403` or a `429` — and of no other reply — to
compare against it. `402` is answered on the status alone. Three rules keep it
honest. The identifier is compared and dropped, so nothing a provider wrote
travels into a sentence a person reads. Spelling is not tracked: the letters are
matched, so `insufficient_quota` and `InsufficientQuota` are one entry. And
**when in doubt it has not run out** — a name that is not on the list leaves the
refusal exactly as it was, because a wrong *the account has run out* sends
somebody to pay for something that was never the problem, which is worse than
the number they would otherwise have been shown. `RESOURCE_EXHAUSTED` is
deliberately absent, and there is a test that says so.
**Date:** 2026-09-03

### What a provider's status code means when a *question* fails
**Version:** the OpenAI-compatible convention as documented by its publishers as
of 2026-09-03. **Not observed against any live provider**; `alo-asking`'s tests
drive a stub on a real socket, and checking this against a provider somebody
pays for is owed alongside the rest of the hardware verification.
**Behaviour:** the convention says what the *endpoint* is and says almost
nothing about which status a provider answers with when it will not answer a
question. In particular **404 means the model, not the address**: a provider
that does not offer the model somebody asked for answers 404 on an endpoint that
exists, which is indistinguishable at the protocol level from an address that is
wrong. 400 is used for a request the provider would not accept and 429 for one
it would have accepted later, and neither is a thing the person who asked can
do anything about.
**Our response:** `alo-asking`'s `hosted.rs` maps each status to the sentence a
person is actually told, and the mapping is written down there beside the
reasoning: 404 and 405 become *the model this question needed was not there*,
400 and 422 become *something answered, and not with an answer*, 401 and 403
become *the key was not accepted*, and everything else becomes *it answered
{status}, which is a problem at that end rather than yours*. What is
deliberately **not** done is guessing between "the model is gone" and "the
address is wrong" — both send a person to look at something, and only one of
them is worth their afternoon, so the sentence names what was needed rather than
what to fix.
**What this entry no longer covers:** two of those statuses have a second
meaning, and it is the entry above. `403` and `429` are read one step further —
the name inside the refusal, against a closed list — because *the account has
run out* is a third sentence and neither status carries it.
**Date:** 2026-09-03

### An OpenAI-compatible address is documented both with and without `/v1`
**Version:** documented behaviour as of 2026-09-02 — Mistral publishes an
address ending `/v1`, the pinned runtime's OpenAI-compatible surface is the bare
address with `/v1/…` beneath it. **Not yet observed against either live
service**; the tests in `alo-models` are against a stub on a real socket, and
checking this against a provider somebody pays for is owed alongside the rest of
the hardware verification.
**Behaviour:** there is no single spelling of "the address of the API". Half the
world writes `https://api.example.com/v1` in the settings field and half writes
`https://api.example.com`, and appending `/v1/models` to the first gives
`/v1/v1/models` and a 404.
**Our response:** `trying.rs` appends `/v1/models`, or just `/models` when the
address already ends `/v1`. It is one line and it is deliberately not cleverer
than that: a 404 here would be read by a person as *my address is wrong* when
their address was right, which sends them to change the one thing that was
correct. A provider that answers on neither is reported as one this system
cannot use, which is what it is.
**Date:** 2026-09-02

### Loopback is taken at face value, and one thing can therefore lie
**Version:** the whole of this repository as of 2026-09-03 — `alo-models`'
`Provider::source`, `alo-egress`' `Leaving::asking` and `alo-asking`'s three
doors. Reasoned rather than observed: there is nothing to observe, because it is
what the code believes rather than what a service does.
**Behaviour:** `Provider::source` answers `InferenceSource::ThisMachine` for a
loopback address, and everything downstream follows:
nothing leaves, law 1 shows nothing, the answer says *on this machine*, and
`SourcePolicy::ThisMachineOnly` permits it. That is right for a runtime and for
a service somebody runs on their own machine. It is **wrong for a proxy** — a
process listening on loopback that forwards the question off the machine would
be believed by every type in this repository, and a person reading their
indicator would see a quiet day.
**Our response:** nothing here, deliberately, and it is written down rather than
worked around. Deciding it in code would mean either refusing loopback (which
breaks the ordinary case ADR 0007 makes the default) or inspecting what is
listening (which is a guess about a process, and a guess this repository is not
in a position to make). The place it is caught is **egress enforcement at the
network boundary**, which is a Linux item in `docs/autonomy/QUEUE.md` and is
where law 1's measurement actually happens: a proxy's own connection leaves the
machine, whatever this repository believed about the socket in front of it. Law
2 is what keeps the hole small — an agent cannot start the proxy.

**What this entry did not cover, and now does not need to:** *whether an address
is loopback at all* was decided by a prefix match until item 18b, so
`http://localhost.attacker.example` and `http://127.0.0.1@attacker.example/`
were this machine to every type here. That was a hole rather than a quirk and it
is fixed — `alo_models::address` parses the host — but it is worth recording
that the two questions look alike and are not: *is this loopback* is answerable,
and *is what is listening on loopback honest* is the one above.
**Date:** 2026-09-03

### Two addresses that really are this machine are treated as somewhere else
**Version:** `alo-models`' `address.rs` as of 2026-09-03. Reasoned rather than
observed, and both cases were checked against `std::net`'s parsers rather than
recalled.
**Behaviour:** `http://127.1` is loopback to curl, to browsers and to most of
libc, which read a short-form IPv4 address; `std::net::Ipv4Addr` refuses it, so
alo OS reads `127.1` as a **name**, which is not `localhost`, and therefore as
somewhere else. `[::ffff:127.0.0.1]` genuinely reaches loopback and
`Ipv6Addr::is_loopback` answers `false` for it, with the same result.
**Our response:** left as it is, because the consequences all fall the safe way.
An address alo OS reads as somewhere else is refused over `http://` (so no key
travels in clear), is shown on the indicator if it is asked at all, and cannot
become an `alo_asking::Served`. The cost is that a person who typed `127.1` is
told to write the address in full, which is a sentence about a keystroke; the
cost of guessing the other way is a question leaving with the indicator quiet.
Handling short-form IPv4 would mean writing an address parser more permissive
than the standard library's, in the one file where being wrong is law 1 failing.
**Date:** 2026-09-03

### The kernel has a name for *that was a symbolic link* and the standard library does not
**Version:** rustc as pinned by `rust-version = "1.97"`, checked by compiling.
**Behaviour:** `alo-agentd` opens its machine description with `O_NOFOLLOW` so
that the file it checks and the file it reads cannot be two different files. The
kernel answers `ELOOP` when the last part of the path is a symbolic link, and
`std::io::ErrorKind` still has no stable spelling for it: `ErrorKind::FilesystemLoop`
is behind the unstable `io_error_more` feature (rust-lang issue #86442), and
using it is a compile error rather than a warning.
**Our response:** the comparison is made against `rustix::io::Errno::LOOP` in
`crates/alo-agentd/src/unix.rs`, which is the file that already holds every
other question this crate asks the kernel, and it hands back a two-variant
`NotOpened` so that nothing outside that file compares a raw number. When the
variant stabilises, the change is one line in one file. Nothing else in the
workspace is affected — this is the only place alo OS asks the kernel to refuse
to follow a link.
**Date:** 2026-09-03

### A function that reads an environment variable cannot be tested
**Version:** edition 2024, as the workspace sets.
**Behaviour:** `std::env::set_var` and `remove_var` are `unsafe` in edition
2024, because another thread reading the environment while one is written is
undefined behaviour. `CLAUDE.md` forbids `unsafe` workspace-wide, so a test
cannot set `$XDG_RUNTIME_DIR` — and a function that goes and reads it is one
whose refusals cannot be exercised at all.
**Our response:** the decision is separated from the lookup. It was
`alo_agentd::session::where_it_runs` taking what the variable said, with
`from_the_environment` as the single line that went and looked; that file is
gone with ADR 0017, which took the socket out of the session, and
`alo_choosing::where_it_is` is the same shape still running — it takes
`$XDG_CONFIG_HOME` and `$HOME` as arguments and reads neither itself.
Every rule is a test over the first, the second has nothing in it to be wrong,
and the shape is worth reaching for anywhere else this workspace ends up reading
the environment.
**Date:** 2026-09-03

### A `poll` resumed after a signal cannot say how much of its timeout is left
**Version:** `rustix` 1.1, Linux, checked by reading `poll(2)` and the crate's
signature.
**Behaviour:** `poll` takes a length of time rather than a deadline, and a
signal ends it early with `EINTR`. There is no way to ask how much of the
timeout was left: `poll(2)` says outright that the remaining time is not
reported, and the portable answer is to read a clock before and after and work
it out. `alo-agentd` resumes the wait rather than reporting it, because on this
machine a signal *is* how a stop arrives, so a resumed wait starts the whole
timeout again.
**Our response:** left as it is, deliberately, and written down here rather than
worked around. The only thing a timeout decides in this service is when a record
is shortened; the interval is an hour (`alo_agentd::ageing::EVERY`) and the rule
it serves is counted in whole days, so an extra hour at the far end of a signal
is inside the granularity of the promise either way. The signal that causes it
is the one that ends the service. Working it out exactly would mean a second
clock read in `crates/alo-agentd/src/unix.rs`, which is the file whose whole
value is that it asks the kernel and decides nothing.
**Date:** 2026-09-03

### An agent that is a login of its own cannot reach a socket under `$XDG_RUNTIME_DIR`
**Version:** systemd-logind as shipped with Ubuntu 24.04, kernel 6.6, found by
running `alo-agentd` as two real users rather than by reading anything.
**Behaviour:** `logind` creates `/run/user/<uid>` with mode `0700`, owned by the
person. `alo_agentd::place` then makes `alo/` beneath it `0750` and hands it to
the group the agent is in, and the socket `0660` — all of which is correct and
none of which helps: reaching a path means traversing every directory above it,
and the agent is a **different user** with no `x` on the person's session
directory. Every connection from the agent's login is `EACCES` before the two
locks this repository designed are consulted at all. The person's own door works,
which is why item 21c's tests never saw it: telling the two doors apart takes two
logins and a test process has one.
**Our response:** written down and left when it was found, then moved. It was
never a bug in `place.rs` — the directory and the socket are exactly the modes
they should be — it was the socket being in the wrong place, and where it goes
instead was a decision with security in it: a directory outside the session has
to be made by something privileged, has to be per-person on a machine that may
have more than one, and has to disappear when they sign out, which is the whole
reason `$XDG_RUNTIME_DIR` was chosen. ADR 0017 took that decision and the socket
is now `/run/alo/<uid>/agentd.sock`: the parent is the image's through
`tmpfiles.d`, the person's directory is the daemon's and goes when the daemon
does. The three requirements are met by three different owners rather than by
one directory that met all of them and could not be entered.

**What has still not been done is the measurement that found this**: a
connection from a second real login. The code is built and unit tested, the
image entry does not exist yet (`docs/autonomy/QUEUE.md` item 28), and no claim
that the agent's door works on a real machine is made anywhere until somebody
runs the two users again.
**Date:** 2026-09-03, and the move on 2026-09-04
