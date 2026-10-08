# Fast Startup is read from the disk too

The installer plan's task 7, and the last piece of code the task 4 chain owed.

On the walk of 2026-10-07 the testing NUC was told **"Whether Windows' Fast
Startup is on could not be found out"** and was never asked the question
[ADR 0064](../../decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
term 9 requires. That machine's Fast Startup is on.

## What was actually wrong

Not the reading's logic. `crate::fast_startup` asked two registry values and
answered *not read* when either was absent, which was honest and was the only
safe answer available to it.

What was wrong is that **the registry cannot answer the question on that
machine.** Measured there 2026-10-08, read-only:

| | |
|---|---|
| `HiberbootEnabled` | present, `1` |
| `HibernateEnabled` | **absent** — only `HibernateEnabledDefault`, at `1` |
| `C:\hiberfil.sys` | **present**, 3,372,613,632 bytes |

A default is not a state, and nothing here has measured what Windows does with
that value when the other is missing, so reading it would be guessing in a
confident voice. The disk, on the other hand, answers plainly: there is a 3.37 GB
hibernation image on it.

## The fix, and the one thing it refuses to do

`Program::ReadingFastStartup` now also reports whether `hiberfil.sys` is among
the entries at the root of the system drive — **found by enumerating the
directory, never by opening the file.**

That distinction is the whole of it. `Test-Path` and `Get-Item -Force` report a
locked system file as one that does not exist, with the same error a missing
file gives, and Windows holds the hibernation image open whenever it is running.
The obvious way to write this check would have told **every machine in the
world** that Fast Startup was off, with confidence, and never asked anybody the
question. `pagefile.sys` and `swapfile.sys` behave identically; the entry is
`docs/misreadings/a-locked-system-file-reads-as-a-file-that-does-not-exist.md`.

**And presence is trusted while absence is not.** A present file was measured.
An absence is what a wrong instrument returns, and it is also what a right
instrument returns on a machine nobody has read — and **nothing anywhere has
measured a Windows with hibernation genuinely off.** So the file can turn *not
read* into *on*, and never into *off*.

## It prints its own control

`RootEntries` is how many entries the enumeration found. The root of a Windows
system drive always holds some, so a count of zero is a broken instrument rather
than an empty disk — and the file's answer is then no answer at all, rather than
*absent*.

This is the same discipline as the `probe_should_be_7=$( (exit 7); echo $? )`
marker that every probe result in this project carries, moved inside a
PowerShell reading: **a measurement has to be able to say that it ran.**

A reading from an older release's script, which prints neither field, is treated
the same way — missing is a zero control, not a missing file.

## Two readings that disagree make no answer

A registry zero beside a present hibernation file is a machine nobody here has
measured. The installer does not pick a winner between two readings of
somebody's computer: *could not be found out* stays the true sentence.

Believing the value would be trusting the registry over the disk. Saying *on*
would put **"Windows' Fast Startup is on"** in front of a person while one of
this program's own readings denies it.

## An existing test caught a real bug in this change

The first version let a registry zero fall through to *it hibernates* whenever
the file could not be read. So a computer with `HibernateEnabled` at zero, read
by an older script that prints no file fields, would have gone from *off* to
*on* and been asked about something that cannot happen on it.

`fast_startup_that_is_off_or_unread_is_not_asked_about` failed with exactly that
input. **It encoded the honest behaviour this change was not supposed to alter,
and the change altered it** — which is what a test for existing behaviour is
for, and why one is worth writing even when the behaviour seems too obvious to
break. That arm now has a test of its own saying why it exists.

## What the NUC will see now

`the_testing_nucs_own_reading_is_on_and_asks` holds that machine's exact triple:
Fast Startup reads **on**, and the question is asked. The same test holds that
without the file's answer it is still *could not be found out* — the sentence
that machine was actually shown — so the change is pinned to the thing that
changed it.

No program of this installer names `powercfg`, and
`fast_startup_is_the_value_and_never_powercfg` still holds.

## What is not measured, and so not claimed

Whether a Windows with hibernation turned off has no `hiberfil.sys`. It is the
obvious reading and it is the one direction this reading does not rest on,
precisely because nobody has read such a machine. Settling it needs a
`powercfg /h off` on the NUC, which no program here may run — so it is a
measurement for a person, recorded in `docs/quirks.md` as owed rather than
assumed.

## The chain this finishes

Tasks 1, 3, 4, 5, 6 and 7 of the installer plan's task 4 chain are now done. A
computer with one disk is offered alo OS beside Windows, asked which road it
wants, told what that road costs including that Windows does not get the space
back, and asked about Fast Startup where that can be found out.

**What remains is not code.** The two metal runs, in the owner's order of
2026-10-06: a failure injected before the point of no return with Windows still
starting afterwards, and only then the real install with switching both ways.
And the way back from the same-disk road, recorded with task 4 — seven removal
sentences written for a road where alo OS has a disk of its own, and a promise
in the first of them whose truth is currently an accident of a guard somewhere
else.
