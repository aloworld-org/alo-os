# polkit takes its helper's answer only when the helper's real number owns the agent

- **Version:** polkit 126 (`polkit-126-3.fc42.2`, the pinned base), with the
  base's downstream change that makes an agent's owner the user of its
  *subject* (`polkitbackendinteractiveauthority.c`, at
  `authentication_agent_new`: *fix pkexec fails with No session for cookie*).
- **Whose:** polkit's. alo OS runs its helper and does not change it.
- **Date:** 2026-11-09, for [ADR 0150](../decisions/0150-administrator-actions-ask-the-persons-password-each-time.md).

## Behaviour

`polkit-agent-helper-1` checks the password and then tells polkitd itself,
with `AuthenticationAgentResponse2 (uid, cookie, identity)`. The `uid` is not
the caller's bus identity: it is the helper's **real** number, `getuid ()`
(`src/polkit/polkitauthority.c`). polkitd only looks for the cookie among the
agents whose owner is that number
(`get_authentication_session_for_uid_and_cookie`), and an agent's owner is the
person whose session it was registered for.

So the helper must run with the **person's real number and an effective number
of 0** — which is what a setuid helper started from the person's own agent
has. Run as plain root, it sends 0, no agent of root's holds the cookie, and a
right password is answered:

```text
polkit-agent-helper-1: error response to PolicyKit daemon:
  GDBus.Error:org.freedesktop.PolicyKit1.Error.Failed: No session for cookie
```

**Measured once, by hand, on 2026-11-09**, in `localhost/alo-os:dev` run as a
container: a session agent shaped like the desktop's (registered for its own
logind session, checking through `alo-checking-a-password`), the runner's own
`the_person_said_yes` from the branch, and polkitd 126.

| `alo-checking-a-password` | right password | wrong password |
|---|---|---|
| ran the helper as root (`main` at `90babddc`) | *No session for cookie*; not accepted | `pam_authenticate failed`; not accepted |
| sets its real number to the caller's first | **accepted** | `pam_authenticate failed`; not accepted |

polkitd kept running through every question. These will not run again unless
somebody repeats them; nothing in CI has a polkitd.

## What alo OS does about it

`alo-checking-a-password` sets its real number to the caller's — the number
`SO_PEERCRED` gave it, never one the caller says — keeping 0 as its effective
one, and then becomes the helper. Its unit holds `CAP_SETUID` for that call
(`image/usr/lib/systemd/system/alo-checking-a-password@.service`).
