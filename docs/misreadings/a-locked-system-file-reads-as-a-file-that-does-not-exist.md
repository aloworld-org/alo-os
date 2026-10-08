# A locked system file reads as a file that does not exist

**Concluded:** *`C:\hiberfil.sys` does not exist on the testing NUC, so that
machine does not hibernate, so its Fast Startup cannot be on.* Recorded
2026-10-07 02:14Z. **Every step after the first was sound and the first was
false.**

## What was true

Measured on the same machine 2026-10-08T01:27Z, read-only:

```
C:\hiberfil.sys
  Test-Path                      False
  Get-Item -Force                Cannot find path 'C:\hiberfil.sys' because it does not exist.
  cmd /c dir /a                  10/06/2026 07:07 PM   3,372,613,632 hiberfil.sys
  [IO.Directory]::EnumerateFiles C:\hiberfil.sys  3372613632  Hidden, System, Archive, NotContentIndexed
```

The file is there, and it is 3.37 GB. Two PowerShell calls say it is not.

## The mechanism

**A file Windows holds open is not a file PowerShell can stat.** `Test-Path`
and `Get-Item -Force` both open or query the file itself, and the kernel's own
exclusive hold on the hibernation image makes that fail — with
`ItemNotFoundException`, which is the *same* error a genuinely missing file
gives. A **directory enumeration** does not touch the file: it reads the
directory, where the entry plainly is.

So the two instruments disagree, and the one that is wrong is the one that is
easier to type and reads as a plain English question.

**It is not about hibernation.** The same machine, same session:

```
C:\pagefile.sys    Test-Path False, Get-Item -Force "does not exist"
                   dir /a and EnumerateFiles: 2,013,265,920 bytes
C:\swapfile.sys    the same: 16,777,216 bytes
```

Three files, all present, all reported absent by the same two calls. Any file
the running system holds open will do this.

## Why it was invisible for a day

**The wrong answer was confident and nothing contradicted it.** `Test-Path`
returned `False` rather than an error, which is an answer and not a complaint.
Nothing downstream could check it: the conclusion *this machine does not
hibernate* was consistent with everything else known about the machine, and the
installer's own reading had separately said *Fast Startup could not be found
out*, which looked like agreement.

It was then **built on, twice**. The testing lane concluded that Fast Startup
could not be happening. The third PC designed task 7 of the installer plan
around *read whether `hiberfil.sys` exists, so a machine with hibernation off is
told off rather than could not be found out* — a fix whose whole mechanism was
the broken reading.

## What it would have cost

This is the part worth the entry. Had task 7 been written as designed, using
`Test-Path`:

> **Every machine would have been told *Fast Startup is off*.**

The file is locked whenever Windows is running, which is whenever the installer
runs. So the check would have returned *absent* on every computer in the world,
the installer would have reported *off* with confidence, and the question ADR
0064 term 9 requires would never have been asked — on exactly the machines where
Fast Startup is on and the hibernated Windows volume must not be written into.

A vague right answer (*could not be found out*) would have been replaced by a
confident wrong one, and the test suite would have passed, because a scripted
machine answers whatever the script says.

## The cure

> **Check whether a file exists by enumerating its directory, never by opening
> or stat-ing the file.** `[IO.Directory]::EnumerateFiles` or `dir /a`, not
> `Test-Path` or `Get-Item`.

And, because the failure mode is a confident `False`:

> **Give any such check a positive control in the same breath, and prefer a
> control that cannot itself be absent.** `pagefile.sys` is a known-present file
> on most machines and is the control that exposed this one. Better still, check
> that the enumeration returned *anything at all*: an enumeration of `C:\` that
> comes back empty is a broken instrument, and an instrument that cannot say it
> failed will report *absent*.

**And the asymmetry that follows**, which is what the installer plan's task 7
will be built on and is **not** in `crate::fast_startup` as this is written.
Presence found by enumeration is a fact: it was measured, on one machine,
once. **Absence is not** - absence is what a wrong instrument returns, and it
is also what a right instrument returns on a machine nobody has measured,
and nothing here has measured a Windows with hibernation genuinely off.

So presence may be trusted and absence may not. The reading task 7 writes
will say *this machine hibernates* when the file is enumerated, and on
absence will fall back to the registry rather than to a conclusion - leaving
*could not be found out* where today it would have said *off*. The vague
right answer survives, and only the case that can be known becomes known.

## The general form

It is this directory's usual shape with an unusual edge. The usual shape is a
check whose answer is read as the answer to a question it was not asked:
`Test-Path` answers *can I reach this item* and was read as *does this item
exist*.

The edge is that **the failure direction is the dangerous one.** A check that
wrongly reported a missing file as present would have been caught by the next
thing that tried to use it. This one wrongly reports a present file as missing,
and *missing* was the input to a conclusion that something was safe.

## Found by

The testing NUC, 2026-10-08, when asked for the raw readings behind a report it
had made the day before — and it found its own earlier claim wrong rather than
restating it. The question that prompted it was asked because the fix being
designed depended on the reading, and a fix that depends on a reading is a
reason to re-read it.

That machine is test-only by the owner's ruling and cannot write to this
repository, so this entry is written by the third PC from its measurements. The
measurements are its; the conclusions are checked here.

## Related

- [`a-search-with-no-output-is-not-a-search-that-found-nothing.md`](a-search-with-no-output-is-not-a-search-that-found-nothing.md)
  — the same asymmetry in a different instrument: a negative needs a control, a
  positive validates itself.
- and the half neither of those had, which the Mac lane is writing up as
  *a negative result proves nothing about the search*: a control establishes
  the instrument, never whether the instrument can answer the question. That
  is this entry exactly. `Test-Path` worked perfectly.
