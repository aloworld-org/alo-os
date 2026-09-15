# ADR 0040 — What an application's grant is over, and where it is kept

**Status:** accepted, 2026-09-15 — option **C**, all four parts. Written by
task 1 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`. The
additive change to `alo-capability` (parts 1–3) and the grants file's new
format in `alo-remembering` (part 4) are **moved into that plan's partition in
writing**: the local-network plan, which owned `alo-capability`, has no
remaining task that edits it and yields it for this change.
**Date:** 2026-09-15
**Context:** [ADR 0001](0001-the-capability-model.md) (grants are enumerated,
revocable and expiring); [ADR 0005](0005-applications-are-sandboxed-and-ask.md)
(a portal request is ADR 0001's sentence with *application* in place of
*agent*, and a person keeps one list, not two);
[ADR 0009](0009-a-good-computer-without-the-agent.md) (declining the agent ends
its grants); [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md) (a turn's
grant is enforced by the kernel);
[ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
(no lane edits another lane's crates); `crates/alo-capability` (`Reach`, `Ask`,
`Grantee`, `Grants`, `Agent`); `crates/alo-remembering/src/written.rs` (the
grants file); `docs/features.md` (*each portal request is a grant in the sense
of ADR 0001*; *★ one list of what has been granted to what — agents and
applications in the same place, revoked the same way*).

## The question in one line

**Task 1 says a portal request is judged against a grant that names the
application and the same `Reach` an agent's grant names, using
`alo-capability`'s types and never editing that crate. For more than half of the
portals `docs/features.md` lists, there is no such `Reach` — so what is an
application's grant over, and where is it kept?**

## Why a worker could not simply choose

The plan is right that there must be one model and one list. It assumed the
model already had room for applications. It has room for some of them. Four
facts, each read from code that is on `main` today:

1. **A `Reach` is a folder, a file or an application, and nothing else.** Of
   the fifteen v0.5 portals, some ask about a path or an application, and
   `Reach` covers those exactly. The others ask about something that has no
   path and is not an installed application:

   | Portal | What a request is for | A `Reach` today |
   |---|---|---|
   | file chooser and documents | the file or folder a person picked | `Folder`, `File` |
   | open-with and default applications | the file, and the application to open it | `File`, `Application` |
   | print | the document | `File` |
   | trash | the file | `File` |
   | wallpaper | the person's desktop background | **none** |
   | notifications | the person's attention | **none** |
   | screenshot | the screen, once | **none** |
   | screen capture | the screen, continuously | **none** |
   | camera | the camera | **none** |
   | microphone | the microphone | **none** |
   | clipboard | what the person copied | **none** |
   | settings | the person's appearance settings | **none** |
   | inhibit | whether the machine sleeps | **none** |
   | network monitor | the state of the network | **none** |
   | power-profile monitor | the power profile | **none** |

2. **A device is not a path that a grant can hold honestly.** The camera
   portal gives an application a PipeWire connection that can only see camera
   nodes. It does not give it a file. The kernel numbers `/dev/videoN` in the
   order devices are found, so the same number can mean a different camera
   after a replug or a reboot. A grant over `/dev/video0` would then cover a
   camera nobody granted, which is a grant made wider without anybody acting.
   The stable names under `/dev/v4l/by-id/` are symbolic links, and
   `alo-capability` requires an ask to arrive with its links resolved
   (`crate::path`). A grant over the link would therefore match no ask at all.
   A microphone under PipeWire has no device node of its own.

3. **The grants live inside the choice to have an agent.** `Agent::Declined`
   holds no `Grants`, and ADR 0009 is the reason. If an application's grants
   were kept in that list, declining the agent would end them. A machine
   whose owner declined the agent could never let a video-call application
   use the camera. ADR 0009 promises a good computer without the agent, and
   this would break that promise for every application.

4. **A grantee is an agent, in the types and in the words.** `Grantee` is
   documented as one agent, and the grants file stores it under `agent`. The
   refusal a person reads when nothing was granted says *grants are made by
   picking a folder, never by asking for one*. That sentence is wrong for a
   camera. On a machine with no agent the refusal says *this machine has no
   agent*, which is irrelevant to a video-call application.

None of these can be fixed from `alo-portals`. Every fix is a change to
`alo-capability` and to the grants file in `alo-remembering`, and the plan says
lane A owns that crate. [ADR 0028](0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md)
settles that no lane edits another lane's crates.

One thing is **not** an obstacle. The kernel boundary of ADR 0013 is built for
each turn from the paths an agent's verb resolves (`alo-bounding`'s
`places.rs`), not from the list of grants. A new kind of reach that the kernel
cannot enforce therefore changes nothing a turn is bound by. The portal backend
enforces an application's grant, and the kernel keeps enforcing an agent's.

**No portal is answered yes by default under any option below.** Every option
refuses a request that no grant covers. The options differ in what a grant can
say, not in whether one is needed.

## The options

### A — Stretch today's types to fit

Treat a device as its `/dev` path. Record a portal that reaches nothing as
`Reach::Application("org.freedesktop.portal.Screenshot")`. Keep applications'
grants in the agent's `Grants`, with an application's identifier as the
`Grantee`.

**What it costs:** a camera grant can come to cover another camera (fact 2).
The one list tells a person they granted *the application
org.freedesktop.portal.Screenshot*, and the grants file stores that pun as
though it were true, so every later reader of the file inherits it. Declining
the agent ends every application's camera (fact 3). A refusal tells somebody to
pick a folder to fix a microphone (fact 4). Each of these is a small untruth,
and the file that keeps them is a contract.

### B — A second list for applications

`alo-portals` keeps its own grants over its own kinds of reach, and the settings
surface shows the two lists next to each other.

**What it costs:** it breaks the starred promise. `docs/features.md` says *one
list, agents and applications in the same place, revoked the same way*, and
ADR 0005 says *not an agent list and an application list, because nobody keeps
two*. A second store is a second revocation path that can disagree with the
first. This option narrows a promise, and a worker may not take it.

### C — `alo-capability` learns what an application's grant is over

An additive change, made in `alo-capability` by the lane that owns it:

1. **`Reach` and `Ask` gain one kind, over a closed list of what this machine
   has that is not a path:** camera, microphone, the screen once, the screen
   continuously, notifications, the clipboard, the desktop background,
   appearance settings, sleep, network state and power profile. It is matched
   exactly, as an application identifier already is. A camera grant is a grant
   to *the camera*. It is not a grant to a device number, and it never covers
   the microphone. The v1 portals (USB, global shortcuts, launchers, remote
   desktop, location) are **absent** from the list, not present and refused.
2. **A `Grantee` says whether it is an agent or an application**, and the
   refusal words differ between the two. An application is not told to pick a
   folder, and no refusal calls an application an agent.
3. **Declining the agent ends the agents' grants and keeps the applications'.**
   On a declined machine, nothing can be granted to an agent, and that stays
   true because of how the types are built. An application can still be granted
   the camera. ADR 0009's promise is about the agent's reach, and applications
   were never part of it.
4. **The grants file gets a new format number**, because a reader of today's
   format would refuse a whole list containing one grant over the camera. The
   contract changes additively and gets versioned, as `CLAUDE.md` requires of a
   public surface.

`alo-portals` then maps each of the fifteen portals to the `Ask` its request
makes, and judges it with `Grants::permitting`, as task 1 intended.

**What it costs:** a change to the crate that every grant passes through, made by
a lane other than the one that needs it, plus a new grants-file format. It
affects every `match` on `Reach` in the workspace (`alo-capability`'s own, and
the grants file's in `alo-remembering`), and each of those has to decide what
the new kind means rather than ignore it.

## The recommendation

**C**, with all four parts. It is the only option in which the one list tells
the truth about what was granted, a device grant cannot drift to another
device, and a person who declined the agent keeps a working computer.

Three things must happen before task 1 is ready again:

- the owner **accepts, amends or rejects** this decision;
- **the crate that owns** `alo-capability` makes parts 1 to 3, or the owner
  moves that change into this plan's partition in writing;
- the **grants file** contract and `alo-remembering` move to the new format
  (part 4).

Until then, no `crates/alo-portals` is created. Building the portal model beside
a decision that is still *proposed* would mean the code chose option A or B
without telling anyone. `crates/alo-granted/tests/a_portal_grant_waits_on_its_decision.rs`
enforces that.

## Consequences

- Tasks 2, 3 and 4 of the plan depend on task 1 and wait with it. Task 4's core
  (what opens a kind of file, as a setting a person owns) does not need a portal
  reach. If the owner wants work to continue before this is answered, that task
  can have its dependency on task 1 narrowed to its last clause.
- An application's grant ends, as every grant does. How long a person's answer
  to *may this application send notifications* lasts is chosen when the grant is
  made, the same way a folder's is. The portal dialog, which belongs to the
  desktop lane, will have to make that choice easy to read. This decision does
  not make any grant last for ever.
- Once C is built, the closed list in part 1 is a public surface, because
  third-party portal clients and the grants file will depend on it. Adding to it
  is additive; renaming anything in it needs versioning.
