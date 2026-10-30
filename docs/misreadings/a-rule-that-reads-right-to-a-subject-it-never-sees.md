# A rule that reads right, to a subject it never sees

**Found 2026-10-30, the walking lane, on a booted machine.**

## What was concluded

The person may set the time zone and the agent may not:
`crates/alo-image/tests/the_person_may_set_the_time_zone_and_the_agent_may_not.rs`
passes, and `50-alo-time-zone.rules` allows `subject.local && subject.active`.

## What was true

In the person's own terminal on the booted machine:

- `cat /proc/self/cgroup` printed `0::/system.slice/alo-desktop.service`.
- `loginctl show-session self` printed *Caller does not belong to any known
  session*.
- `timedatectl set-timezone Europe/Berlin` printed *Interactive authentication
  required*.
- logind's `CanReboot` and `CanPowerOff` answered `"challenge"`.

polkit decides `local` and `active` from the caller's session, and the desktop
(and everything it starts) is in none. ADR 0115 found this in 2026-10 for
libseat. It was then routed around for the seat, but not for polkit.

## The mechanism

The test reads the rule's text and checks that it says the right thing. Whether
the person's processes ever match `subject.active` depends on a property of a
booted machine, which no file in the repository records. The rule was right,
and the subject it trusts does not exist here.

## The cure

A rule that trusts a session is tested with the session: on the next walk,
`loginctl show-session self -p Active` from the desktop's terminal is the check,
and it belongs beside the rule's test as the thing that test cannot see. The
owner chose the ordinary fix: the sign-in screen starts the desktop inside the
person's login session.
