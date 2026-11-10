# polkit with no agent to ask answers with a challenge, not an error

- **Version:** polkit 126 (`polkit-126-3.fc42`, the pinned base), read from its
  source on 2026-11-09: `src/polkitbackend/polkitbackendinteractiveauthority.c`
  and `polkitbackendsessionmonitor-systemd.c`.
- **Whose:** polkit's. alo OS configures it and reads its answers.
- **Date:** 2026-11-09, for [ADR 0150](../decisions/0150-administrator-actions-ask-the-persons-password-each-time.md).

## Behaviour

`CheckAuthorization` with *allow user interaction* returns three values:
authorised, challenge, and details. When an action needs an administrator's
password and **no authentication agent is registered** for the subject's
session, polkit does not return an error. It skips the agent and returns the
result it would have challenged with: **not authorised, challenge true**.

When the agent answered *Cancelled*, polkit puts `polkit.dismissed = true` in
the details. Any other failure is not authorised and not a challenge, with
nothing to say why.

And a process in no logind session at all, such as a system service, is
matched to a session only through `sd_uid_get_display` on its owner. That
lookup finds nothing for a process under `system.slice`, so asking about
**the service's own process** finds no agent, even while the person's desktop
has one registered.

## What alo OS does about it

- `alo_administrator::what_polkit_said` reads *not authorised and a challenge*
  as **nothing could ask**, and says that to the person, never *the password
  was wrong*.
- The runner asks polkit about **the leader of the person's own session**,
  which logind names as their `Display` session, and not about the agent
  service's process, so the sheet that is registered there is the one asked.
  It asked about the session itself until 2026-11-09, which polkit 126 aborts
  on (`polkit-aborts-on-a-session-subject.md`).

**Not yet seen on a booted machine.** This was read from source; the next boot
walk is where it is measured.
