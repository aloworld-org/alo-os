# polkit 126 aborts when it is asked about a session

- **Version:** polkit 126 (`polkit-126-3.fc42.2`, the pinned base), with its
  JavaScript rules engine built on duktape.
- **Whose:** polkit's. alo OS configures it and asks it questions.
- **Date:** 2026-11-09, found on a boot walk of an image built from `main` at
  `90babddc`, for [ADR 0150](../decisions/0150-administrator-actions-ask-the-persons-password-each-time.md).

## Behaviour

`CheckAuthorization` accepts three kinds of subject on the bus: a process
(`unix-process`), a bus name (`system-bus-name`) and a login session
(`unix-session`). The third is parsed, checked for its owner, and then handed
to the rules engine, which **only knows the first two**:

```c
/* src/polkitbackend/polkitbackendduktapeauthority.c, push_subject () */
if (POLKIT_IS_UNIX_PROCESS (subject))
    process = subject;
else if (POLKIT_IS_SYSTEM_BUS_NAME (subject))
    process = polkit_system_bus_name_get_process_sync (...);
else
    g_assert_not_reached ();
```

So the whole daemon aborts with `SIGABRT`. Every question anybody is waiting
on is lost with it, and systemd restarts polkit on the next request.

**What the booted machine showed** (`w19-serial.log`, t = 2999–3013):
`polkitd` killed by signal 6 two seconds after `alo-running-as-administrator`
started, its core's stack running from GLib's `call_in_idle_cb` (where GDBus
dispatches a method call) through five frames of the polkitd binary to
`g_assertion_message_expr`; and then *not run: polkit could not be asked*.

**Measured once, by hand, on 2026-11-09**, in `localhost/alo-os:dev` run as a
container with systemd, with a person `tester` in `wheel` given a logind
session (`c1`, leader pid 562) by a unit with `PAMName=login`:

```text
busctl call ... CheckAuthorization '(sa{sv})sa{ss}us' \
  unix-session 1 session-id s c1 org.aloworld.alo.run-as-administrator 1 command whoami 0 ''
  -> Call failed: Remote peer disconnected
     polkit.service: Main process exited, code=killed, status=6/ABRT

busctl call ... CheckAuthorization '(sa{sv})sa{ss}us' \
  unix-process 3 pid u 562 start-time t 23100667 uid i 1001 \
  org.aloworld.alo.run-as-administrator 1 command whoami 0 ''
  -> (bba{ss}) false true 1 "command" "whoami"     polkit still running
```

The second is the answer polkit gives when the action needs a password and
nobody was asked: a challenge.

## What alo OS does about it

The runner never asks about a session. It asks about the session's **leader**
— the process logind started it with — by number, start time and login
(`crates/alo-administrator/src/the_process_polkit_is_asked_about.rs`). polkit
finds an authentication agent by the session its subject is in, so the
desktop's password sheet, registered for that session, is the one asked.

The **agent's registration** still names the session (`unix-session`):
`RegisterAuthenticationAgent` does not pass its subject to the rules engine,
and the walk's journal shows polkit taking it (*the password sheet is polkit's
agent for session 3*).

Nothing here can see a polkit that aborts; the next boot walk is where the
sheet appearing is measured.
