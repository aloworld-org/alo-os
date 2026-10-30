# ADR 0141 — The desktop is put inside the person's session by what already has the privilege

**Status:** **accepted, 2026-10-30, by the owner, choosing option (b)**:
systemd starts the person's desktop; `alo-sessiond` starts
`alo-desktop@<uid>.service`, and no alo program gains any capability. Amends
[ADR 0115](0115-the-desktop-runs-inside-the-session-alo-sessiond-opened.md).
Proposed the same day with the question in the last section and no code; the
code followed the answer, and *What building it found* records the one thing
the proposal had not seen.

**Context:** the owner's decision of 2026-10-30, *"Do it the ordinary way"* —
the sign-in screen starts the person's desktop inside their logind session, as
GDM, SDDM and greetd do. [ADR 0024](0024-what-a-person-signs-in-at.md) (what a
person signs in at), [ADR 0087](0087-the-signed-in-session-is-kept-by-one-owner-and-notifications-ask-it-twice.md)
(one owner), [ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
(a capability goes to the smallest thing that can hold it),
[ADR 0121](0121-the-thing-that-opens-a-session-makes-the-persons-folder.md),
[ADR 0140](0140-the-first-start-sets-the-time-zone-itself.md) (the time zone
and its polkit rule), #719 (the greeter's session class), #1029 (the sign-in
screen steps aside).

## What it changes for the person

Today a person who signs in gets a desktop that **is not in their own
session**, so the machine treats them as somebody who is not sitting at it:
restarting, switching off and changing the time zone all ask for a password,
and there is no box anywhere to type one. After this, those work the way they
do on any Linux desktop, without a password, and the agent still cannot do any
of them.

## What a booted machine showed

Image of `main` `ab247650` plus #1031, qemu, 2026-10-30, **measured once by
hand**, from the terminal the desktop starts:

- `/proc/self/cgroup` was `0::/system.slice/alo-desktop.service`;
- `loginctl show-session self` answered *Caller does not belong to any known
  session*;
- `loginctl` listed two sessions: `4` (class `manager`, no seat) and `c1`
  (class `user`, `seat0`, `tty1`, leader 884);
- `timedatectl set-timezone Europe/Berlin` failed with *Interactive
  authentication required*, though the rule allows `subject.local &&
  subject.active`;
- logind's `CanReboot` and `CanPowerOff` both answered `challenge`.

`crates/alo-image/tests/the_person_may_set_the_time_zone_and_the_agent_may_not.rs`
passes while the machine cannot do it. Its own header says it cannot evaluate
polkit; what nothing said is that **the person was in no session for polkit to
find**.

## Why: three measurements and one inference

**Measured 2026-10-30, once, by hand, inside the image itself**
(`localhost/alo-os:dev`, systemd `257.13-1.fc42`) booted under
`podman --systemd=always`. No seat and no virtual terminal exist there, so
these say where a process lands and nothing about the seat. The script was
`C:/dev/session-scope-probe.sh` on the Windows lane machine; it is not in the
repository and nothing runs it again.

| route | what it is | where the process was afterwards |
|---|---|---|
| A | a unit with `User=` and `PAMName=` whose stack has `pam_systemd` — **what `alo-desktop.service` is** | `session-c1.scope`, and its child too; `XDG_SESSION_ID=c1`, `Active=yes` |
| B | a root unit forks a child and calls `CreateSession` naming the **child** as leader — **what a display manager does** | the child in `session-c3.scope`; the parent still in its unit |
| C | a root unit calls `CreateSession` naming **itself** — **what `alo-sessiond` does** | the unit's main process moved into `session-c4.scope` |

So **the ordinary mechanism works on this base by either road A or road B.**
Road A is systemd's documented behaviour for `PAMName=` (*the main unit process
will be migrated to its own session scope unit*), and it is the shape upstream
weston's and cage's own example units use.

**Road C is what is wrong.** On every sign-in `alo-sessiond` opens a session on
`seat0`/`tty1` with **itself** as leader, so a privileged daemon is moved into
the person's session scope and **holds the seat**. The `c1` on the booted
machine — class `user`, `seat0`, `tty1`, a `c` name because it has no audit
session — is that session.

**The inference, not measured on a machine:** the desktop's own `pam_systemd`
then asks logind for a session on the same seat and terminal, logind answers
`SessionBusy`, and `pam_systemd` treats that as success and opens nothing — so
PAM says *success*, the desktop stays in `alo-desktop.service`, and it still
draws, because `libseat` falls back to the person's *display session*, which is
`c1`. polkit asks by process, finds none, and challenges. Every line of the
booted machine's evidence fits this; the journal line that would confirm it is
`pam_systemd(alo-desktop:session): Not creating session` and is on the walk
list below.

## The owner's decision taken literally, and what sinks it

The decision names the compositor as the process that opens the PAM session,
drops to the person's uid, gid and groups, and execs `alo-desktop`. Asked the
question ADR 0115's amendment says should have been asked first — *is this
process even allowed to do that?* — from the files:

1. **Becoming the person needs `CAP_SETUID` and `CAP_SETGID`**, and
   `alo-compositor.service` holds `CapabilityBoundingSet=` empty with
   `NoNewPrivileges=yes`. A child cannot regain what its parent's bounding set
   dropped. So the compositor's unit would have to grant both.
2. **ADR 0119 refused exactly that trade, for a smaller capability, on a
   measurement, and named this process.** *"Grant it to `alo-compositor`" —
   rejected: 113,587 lines, 51 dependencies, a password prompt and
   input-device parsing*; and its consequences: *`alo-sessiond` and
   `alo-compositor` keep their empty bounding sets.* `CAP_SETUID` is a larger
   power than `CAP_CHOWN` — it is the power to become any account on the
   machine, the agent's included. **This is a contradiction with a settled
   ADR, which is why this record stops here rather than building.**
3. `pam_loginuid`, in every stack this image ships, needs `CAP_AUDIT_CONTROL`:
   a third capability, or a stack that attributes less to the person.
4. There is no PAM client in the workspace and `unsafe_code` is forbidden. A
   libpam binding is a new C dependency on the sign-in path. (Avoidable: road
   B can call `CreateSession` directly, which ADR 0024 measured root may do.)
5. **It would fail the same way the desktop does today** unless `alo-sessiond`
   also stops opening its own session on the seat. That change is needed
   whichever road is taken.

Items 1–3 are the reason to ask; 4 and 5 are engineering.

## The same ordinary mechanism, with no new capability (recommended)

**Road A, which the image already half-uses.** pid 1 is the process that
already holds the privilege, and `User=` with `PAMName=` is the ordinary thing
it does with it: open the `pam_systemd` session, move the process into the
session's scope, become the person, exec. The only fault on today's machine is
that `alo-sessiond`'s session got to the seat first.

The questions this record was asked to answer, for this option:

- **Who holds the person's PAM and logind session, and for how long.** The
  desktop's unit, and nothing else: systemd keeps the PAM handle (its
  `(sd-pam)` process) for exactly as long as the unit runs. **One owner.**
  `alo-sessiond` stops calling `CreateSession`, so there is no second session
  for the same person and no root daemon inside their scope.
- **How the display and the seat are handed over.** By logind, which is how
  every display manager's handover works. The desktop's session is created on
  `seat0`/`tty1` as class `user` and becomes the seat's active session; logind
  pauses the sign-in screen's devices and drops its DRM master — the pause #1029
  already recognises — and the compositor exits 0 and is not restarted. The
  compositor does not need to release anything by hand, because logind takes
  it.
- **When the desktop exits.** Signing out is the desktop exiting 0: the unit
  stops, its PAM session closes, and `OnSuccess=alo-compositor.service` puts the
  sign-in screen back. A crash restarts the desktop (`Restart=on-failure`), and
  if it keeps failing, `OnFailure=alo-compositor.service` puts the sign-in screen
  back rather than leaving a console. (There is no *sign out* control yet:
  `alo-leaving` is called by nothing. This makes the desktop's end correct; it
  does not add the button.)
- **What remains of `alo-desktop.service`.** It becomes `alo-desktop@.service`,
  instanced by the person's number with `User=%i`, keeping `PAMName=`,
  `TTYPath=/dev/tty1`, an explicit `XDG_SESSION_CLASS=user`, its runtime
  directory and its empty capability set. `BindsTo=user@1000.service` and
  `WantedBy=user@1000.service` go: the desktop's session is now what starts the
  person's user manager, not something started by it.
- **Capabilities.** None gained, by any unit. `alo-compositor` and
  `alo-sessiond` keep empty bounding sets; asking systemd to start a unit is
  authorised by being root, which is how ADR 0121's helpers are already started.
  A test holds the shipped desktop unit to `User=`, `PAMName=`, `TTYPath=` and
  no `WantedBy=user@…`, and `this_crate_cannot_do_what_its_unit_forbids.rs`
  keeps passing unchanged.
- **What `alo-sessiond` still does.** Everything it does now except one call:
  it checks the caller is the sign-in screen, checks the number is an account,
  makes their undo folder and gives them their name (ADR 0121, 0135), and then
  **starts `alo-desktop@<number>.service`** — a constant and a checked number,
  the shape it already uses twice — in place of opening a session itself.
  *Already signed in* is asked of systemd (is that unit active?) rather than a
  flag that never resets, so a person who signed out can sign in again.
- **The agent gains nothing.** The time-zone rule refuses `alo-agent` by name
  before it looks at sessions. The agent's turns run in `alo-asking@.service`
  and `alo-agentd` in `alo-agentd.service`, both system services in no session,
  so `subject.active` is false for them whatever this changes. Only processes
  the desktop starts — the person's terminal and applications — enter the
  session.

## What would sink the recommended option

- **The seat is still refused.** If logind refuses the desktop's session on
  `tty1` because the greeter's session is there, nothing changes. #719 measured
  that a `greeter` session does not block the person's, but it measured
  `alo-sessiond`'s call, not `pam_systemd`'s.
- **Road A on a seat.** The podman measurement had no seat. Migration into the
  scope is the same code path with or without one, but it is unmeasured on one.
- **A failed desktop that never opens a session** leaves the sign-in screen in
  front with nobody behind it until the restart limit fires `OnFailure=` —
  about ten seconds of a screen that says nothing. Recorded, not solved.
- **`OnSuccess=` at shutdown.** A desktop stopped by a shutdown would ask for
  the sign-in screen; the screen's unit conflicts with `shutdown.target` by
  default, so the job is cancelled. Unmeasured.
- **Two people.** One terminal, `tty1`, is named. A second person's desktop on
  the same seat is ADR 0118's question and this does not answer it.

## What was rejected

- **A new helper holding `CAP_SETUID` and `CAP_SETGID`**, in ADR 0119's shape.
  It would rewrite what pid 1 already does with privilege it already has;
  engines are configured, never written.
- **Keeping `alo-sessiond`'s session but without a seat**, so the desktop's
  PAM session can take the seat. Two sessions for one person, and a privileged
  daemon still moved into their scope on every sign-in (road C).

## The question for the owner

**The literal reading of "do it the ordinary way" gives `alo-compositor` —
113,587 lines, the screen a password is typed into — `CAP_SETUID` and
`CAP_SETGID`, which ADR 0119 refused to that process for a smaller capability.
Do you want:**

- **(a)** that, and ADR 0119 amended to say the compositor may become the person;
  or
- **(b)** the same ordinary mechanism — a `pam_systemd` session, the desktop's
  processes in the session's scope, its environment from PAM — with systemd as
  the process that becomes the person, started by `alo-sessiond` in place of the
  session it opens today, and **no capability gained anywhere**?

This record recommends (b). Either way `alo-sessiond` stops opening a session
of its own on the seat, because that is the measured fault.

## What building it found: `try` lost its box, and gets it back

**Moving the desktop into the session took the person's `try` with it.**
`alo-try` makes its sealed box as a child of its own cgroup, which worked
because `alo-desktop.service` said `Delegate=yes`. A login session's scope is
`logind`'s. Measured inside the image on 2026-10-30, once, by hand, as the
person in a session opened by road A: the scope was `root:root` and `mkdir`
answered *Permission denied* — and the same with a `Delegate=yes` drop-in for
every `session-.scope`, which those scopes do not take.

The person's own manager can give them a delegated scope, measured the same
way: `systemd-run --user --scope -p Delegate=yes` put the process in
`user@<uid>.service/app.slice/run-….scope`, owned by the person, type
`domain`, and a box made and moved into there. So `alo-try`, when its cgroup is
not the person's, runs itself again inside such a scope, once
(`crates/alo-try/src/a_scope_of_its_own.rs`). Only the `try` process moves; the
terminal it was typed in stays in the session. *This makes an existing promise
true — a sealed box, ADR 0064's default — rather than adding one.*

## What was built

- `image/usr/lib/systemd/system/alo-desktop@.service` replaces
  `alo-desktop.service`: `User=%i`, `PAMName=alo-desktop`, `TTYPath=/dev/tty1`,
  `XDG_SESSION_CLASS=user`, no `[Install]`, no `user@` in it, and
  `OnSuccess=`/`OnFailure=alo-compositor.service`.
- `alo-sessiond` no longer calls `CreateSession`; it starts that unit and waits
  until it runs, and asks systemd whether it still runs instead of keeping a
  flag that never cleared.
- `alo-compositor` is unchanged in behaviour: #1029's step-aside stands.

## How it is known to be done

Not by a unit starting. The tests above run on every build; the machine has
not been booted with this change. The next walk on a booted machine checks,
from the person's terminal:

- `cat /proc/self/cgroup` shows a `session-N.scope`;
- `loginctl show-session self -p Active` says `yes`;
- logind's `CanReboot` says `yes`;
- `timedatectl set-timezone Europe/Berlin` works with no password;
- the desktop survives sign-in and is not pushed off (#1029's behaviour);
- signing out — the desktop exiting — brings the sign-in screen back;
- `try true` works from the terminal, inside a `run-….scope` of its own;
- and, on an image from **before** this change, the journal has
  `pam_systemd(alo-desktop:session): Not creating session` — the line that
  confirms the inference above. **The person's account cannot read the system
  journal on the image**, so this is read from the serial console or as root.
