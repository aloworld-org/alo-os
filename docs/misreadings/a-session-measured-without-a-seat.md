# A session measured without a seat

**Concluded**, twice. On 2026-11-09, by the lane that let the person use `sudo`
(#1151): *with neither line, a process of the person's answers `root`*, and the
desktop's unit was commented *handed nothing*. On 2026-11-11, by the lane
looking at why `flatpak run` failed in the terminal: *a process started under
the desktop's unit holds no capability at all, so removing those lines cannot
be the cause.* Both were measured in `localhost/alo-os:dev` with `systemd-run`.

**True.** On a booted machine the desktop held `CAP_WAKE_ALARM` in its ambient
set, its children inherited it, and Fedora's `bwrap` refused every `flatpak run`
from the person's terminal. `pam_systemd` raises that capability only for a
session **on a seat** (or for the service `systemd-user`). The measurements ran
with no seat: the first with no PAM session at all, the second with
`PAMName=alo-desktop` but without the unit's `TTYPath=`, because with it PAM
failed in the container. The walk's journal answered it in one field:
`_CAP_EFFECTIVE=800000000` on every line `alo-desktop` logged.

**The mechanism.** A stand-in is chosen because it shares the obvious
properties with the real thing, and it is trusted on the properties it does not
share. `systemd-run -p User=` shares the unit's user and capability lines. It
does not share the session, and a session is where PAM modules act. The line
dropped to make the container work (`TTYPath=`) was the line that made the
difference. The result was a clean, confident negative. This was found because
a booted machine failed, not because any check failed. **Nothing caught it.**

The 2026-11-11 lane was one step from telling the owner that #1151 was cleared.
What stopped it was asking why the walk's machine and the container disagreed.

**The cure.**

- When a measurement in a container disagrees with a booted machine, look for
  what the container dropped before you trust it. Write down any line you
  removed to make a unit start there. It is a candidate cause, not
  scaffolding.
- Before a container, ask the booted machine's own record. The journal stores
  `_CAP_EFFECTIVE`, `_UID` and `_SYSTEMD_CGROUP` for every line a process logs,
  and a disk's journal can be read with `journalctl -D` from a
  read-only mount, without touching the machine.
- For a capability question about a session, reproduce `pam_systemd`'s seat
  branch with the PAM service name `systemd-user`
  (`docs/quirks/pam-systemd-hands-a-seated-session-a-capability-and-bwrap-refuses-it.md`).
