# Run A on the testing NUC: a failure before anything is lost

**For the person at the machine.** This is the first of the two metal runs the
owner's ruling of 2026-10-06 sets, in the order it sets them: *a failure
injected before the point of no return, with Windows still booting afterwards*
— and only then the real install.

Nothing here decides anything. Every judgement is made in advance so that
whoever is at the NUC is reading rather than choosing.

## Where the build comes from

**A public release page, downloaded signed out, exactly as a person would.**
The owner ruled *we shall not send users zip files but they will install it
from the online*, and this satisfies it rather than working around it:

- `aloworld-org/alo-os` is a **public** repository (`"visibility":
  "public"`, read from the API);
- `installer-candidate.yml` publishes with `gh release create --prerelease`,
  which is a release page, not a workflow artefact.

So nobody signs in and nothing is sent between machines. **Do not use a
workflow artefact** even if one is offered: those need a signed-in download
and are the route the owner ruled out.

## What is actually being measured, and why it is not what the ruling's words suggest

The ruling says *before the point of no return*. On the road being walked there
is no point of no return in that sense, and the difference matters:

- `crate::the_point_of_no_return::crossed` is called from **one** place,
  `the_replacing_road.rs`. That is the road that erases Windows.
- The keep-Windows road **never destroys anything of the person's.** Windows is
  made smaller; alo OS's space is new and empty. There is nothing to be past.

So the claim this run tests is `staging.rs`'s own, which is stronger and is
the one a person actually meets:

> **A run killed at any step leaves a Windows that starts.** A smaller Windows
> starts; an extra partition does not stop it; an entry listed last is not
> started.

**Killed, not failed.** A step that fails gracefully puts its own changes back —
the installer has a journal for that, and the scripted tests cover it. A killed
process runs no put-back at all, so whatever is on the disk stays there. That is
what happens when somebody closes the window, the machine loses power, or the
battery goes, and it is the case no test can reach.

## Where to kill it, named by what the screen says

**Do not use a step number.** The first version of this checklist gave one, and
the testing lane found that its own copy of `staging.rs` numbers the steps
differently — so *kill at step 6* meant *before the firmware entry exists* in
one list and *after it is written* in the other. A number is an index into a
list that differs between readers. **The sentence on screen is what the person
actually sees**, it is in the vocabulary so it is translated, and it does not
move when a step is added.

So, exactly:

> **Copying the installer into its area, and checking the copy**

**Kill it while that line is the last one on screen.** That step writes about
223 MB — a kernel and an initramfs — onto the area and reads each file back, so
on a SATA SSD it takes seconds rather than milliseconds. It is the only moment
wide enough to hit on purpose.

**What has already happened when that line appears:** Fast Startup is off,
Windows has been shrunk, the installer's area exists and is formatted, and **alo
OS's own partition exists and is labelled `ALO-ROOT`.** Every change to the
partition table is done.

**What has not:** no firmware entry, no next-start. So nothing would carry the
machine forward, which is the point.

That step copies about 223 MB — a kernel and an initramfs — onto the FAT area
and reads each file back. On a SATA SSD it takes seconds rather than
milliseconds, and it is **the only step wide enough for a person to hit on
purpose.** On screen it is the line about copying the installer.

**Why not the worse moment, which is after the firmware entry is written.**
Because the lines around it pass in well under a second, and the testing lane
said the thing that settles it: *I cannot kill faster than the output scrolls.*
A person who is late sets the next start, and the run then measures something
else; a person who is early may catch the entry half-written.

**What this run therefore does not prove:** *an entry listed last is not
started.* No entry exists when that line is on screen. That half of the claim
is measured by run B, where the entry is written and the machine must still
start Windows by default when nobody chooses otherwise.

**If the kill lands somewhere else anyway**, record **the last line that was on
screen** and carry on. The run is still worth having; it answers a different
question and the line says which. **Do not retry for a cleaner kill** — a second
run starts from a changed disk and is not the same test.

## Which build this describes

**The step order above is this commit's**, and it must be the commit the test
build was made from. The testing lane's own checkout has a different order and
no step for alo OS's partition at all, because it predates that road.

So before running: check that the candidate's release page names the commit, and
that it is the one this document shipped with. If they differ, **read the
`staging.rs` of the build you are holding** — its header lists the steps, and
since 2026-10-08 it lists both roads.

## Before you start

Capture these, elevated, read-only. They are the before-state the after-state is
compared against, and a run without them measures nothing.

| | |
|---|---|
| `Get-Disk \| Format-List` | every disk |
| `Get-Partition \| Format-List *` | number, type, GPT type, offset, size |
| `Get-PartitionSupportedSize -DriveLetter C` | and **note the time** — this moves, see `docs/quirks.md` |
| `Get-Volume \| Format-List` | labels |
| `bcdedit /enum firmware` and `bcdedit /enum {fwbootmgr}` | order and timeout |
| `Confirm-SecureBootUEFI` | and the registry value |
| the two hibernation values, and `cmd /c dir /a C:\` | **not** `Test-Path` — see the misreadings entry |

Keep the whole screen too. `Tee-Object` on the installer's output, as the walk of
2026-10-07 did.

## The run

1. **Download the candidate from the release page**, in a browser, as a person
   would. Not with PowerShell — that attaches no Mark-of-the-Web and SmartScreen
   never appears, which is why *what the warning looks like* is still owed.
   **Record what SmartScreen shows**, and that it can be got past.
2. **Read the first line.** It must say *This is a test build of alo OS from
   2026-10-08. It is not a release.* If it does not, stop and say so — the build
   either is not a candidate or did not carry its date, and both are findings
   worth more than the run.
3. Let it check the computer. **Record every line**, especially what it now says
   about Fast Startup: that machine's reading should be **on**, and the question
   should be asked. Until today it said *could not be found out* and asked
   nothing.

   **When it asks, answer *turn off*.** Decided here rather than at the machine,
   because the answer writes to Windows and a killed run puts nothing back.
   The reason is about evidence: with Fast Startup left on, the restart this run
   depends on **resumes a hibernated kernel session** rather than starting cold,
   and a resumed session is weaker evidence that Windows survived having its
   partition shrunk — it does not re-read the disk the way a cold start does. A
   cold start is the measurement; a resume would look like one.

   **It stays off afterwards**, because nothing puts it back. That is a real
   change to the owner's Windows, it is small, and the clean-up below says to
   restore it.
4. Answer the questions as a person keeping Windows would: **both systems**, and
   then — if it asks — the same disk. On a one-disk machine it should not ask
   where, because there is only one place.
5. **Check the numbers against these before agreeing.** Computed from that
   machine's own bytes, three ways: Windows 110.71 GiB → 85.71 GiB, the
   installer's area 1 GB, **alo OS 24 GB exactly**. The sentences must add up:
   *25 GB smaller*, *1 GB area*, *alo OS gets 24 GB*. If they do not, stop.
6. Agree, and watch for the line **Copying the installer into its area, and
   checking the copy**. That is the window.
7. **Kill it.** Close the window, or end the process. Do not power off — a
   power cut is a different test and a harsher one; this one is the common case.
8. **Restart the machine, and let it start normally.** Do not touch the boot
   menu.

   **Watch the screen through the restart, and photograph anything about the
   TPM.** That machine has a physical-presence request still pending — a
   restart on 2026-10-07 did not consume it — and this is a restart, so the
   firmware may finally act on it. If it asks for a key press about clearing or
   enabling the TPM, **photograph it before pressing anything** and say what was
   pressed. A request consumed without a record is a change to that machine
   nobody can describe afterwards.

## What must be true afterwards

- **Windows starts.** This is the whole claim. It should start by itself, with
  no menu choice and no media.
- Windows is about 25 GB smaller, and says so.
- There are two new partitions: a 1 GB one labelled `ALO-INSTALL` and a 24 GB one
  labelled `ALO-ROOT`. **Both are expected.** Nothing put them back, because
  nothing ran.
- There is a firmware entry named alo OS, **listed last**, and the machine did
  not start it.
- Nothing of the person's is gone. The Windows volume's files are untouched —
  only its size changed.

Then capture the after-state with the same commands and line-diff it against the
before-state, ignoring timestamps and the two things known to move: C:'s free
space, and `SizeMin`.

## How to leave the machine

**There is no firmware entry to remove, and no removal program to run.** This
section said there were, and it was wrong: it was written when the kill was
after the entry was written, and the kill moved to the file copy without this
moving with it.

At the moment you kill it, steps 7 and 8 have not happened. **The installer's
copy of itself, its Start-menu shortcut and the firmware entry named alo OS do
not exist.** So the removal program is not on the machine, and running it from
elsewhere would say *alo OS is not among the systems this computer can start*,
which is correct and is not the thing being measured here.

**So the clean-up is three things, all in Windows' own tools:**

| | what | with |
|---|---|---|
| 1 | delete the 1 GB `ALO-INSTALL` area and the 24 GB `ALO-ROOT` partition | Disk Management |
| 2 | extend C: back by about 25 GiB | Disk Management |
| 3 | put Fast Startup back on | Windows settings |

**Check the firmware list anyway, and expect it unchanged.** `bcdedit /enum
firmware` should match the before-state exactly. If it names alo OS, the kill
landed later than intended and the run measured something else — say so, and
then `bcdedit /delete` is needed and **who runs it is the owner's to say.** It
is a Windows tool and a measurement action rather than a code change.

**Say what the whole of it took**, because that is what a person who changed
their mind faces, and it is the measurement behind the removal work.

### What changes for run B, and is not true yet

Pull request 583 gives the removal a road on this disk: it takes alo OS's
partition away and removes the firmware entry, under the same guard that made
the partition. It does **not** grow Windows back — that stays one step in Disk
Management, deliberately.

**It is open and not merged, and the candidate on the release page was built
before it.** So the build being carried tonight does not have it, and run B's
clean-up is the thing to re-read this for rather than this run's.

## What this run does not show

That an install works — that is run B, and it comes second deliberately: a walk
that installs first has already spent the thing this one protects.

That a graceful failure puts things back — the scripted tests cover that, and
this run deliberately denies the installer the chance.

## If it does not start Windows

Stop. Do not install anything, do not try again, and do not repair it from
inside Windows. Capture the firmware's own view — `bcdedit /enum firmware` from
recovery media if need be — and say exactly what the screen showed. **That
outcome is the most valuable result this run can produce**, and it is worth more
intact than fixed.
