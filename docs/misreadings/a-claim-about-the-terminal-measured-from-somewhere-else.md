# A claim about the person's terminal, measured from somewhere that was not the terminal

**Found 2026-11-09, on a boot walk of an image built from `main` at
`90babddc`. It passed every check this repository has; a person typing `sudo`
is what caught it.**

## What was concluded

[ADR 0150](../decisions/0150-administrator-actions-ask-the-persons-password-each-time.md),
and the pull request that built it (#1139), told the person:

> Every sign-in hands it to the system's list, so it stays the same
> everywhere, including for `sudo` in their own terminal, which now works with
> that password too.

## What was true

On the booted machine, in the person's own terminal:

```text
sudo: The "no new privileges" flag is set, which prevents sudo from running as root.
```

alo's terminal is drawn inside the desktop, and the person's shell is a child
of the desktop's process. `alo-desktop@.service` set `NoNewPrivileges=yes` and
an empty `CapabilityBoundingSet=`, and a process inherits both. Measured in
`localhost/alo-os:dev` the same day, a process of the person's under each:

```text
NoNewPrivileges=yes       sudo: The "no new privileges" flag is set
CapabilityBoundingSet=    sudo: unable to change to root gid
neither                   root
```

**Every fact the claim was built from was true and measured**: the person was
in `wheel`, the password was written to `/etc/shadow` as a `$y$` hash, and
`sudo` reads both. Each was measured by hand in the dev image run as a
container, from a shell no unit had limited (ADR 0150, *Measured once*).

## The mechanism

**Whether a program works depends on two things: what it is allowed (the
credentials) and the process it runs in (what that process inherited).** The
claim measured the first and assumed the second. Every measurement was made
from a shell nobody had limited, so the second could not show up in any of
them — and the place where it is decided, the unit of whatever starts the
terminal, is a file about the *desktop*, which a change about passwords had no
reason to open.

It is invisible for the same reason it is wrong: **a limit set on a service is
also set on every child it starts, and which children a service starts is not
written in its unit.** `alo-desktop@.service`'s comment said *it holds nothing*,
true of the desktop and silent about the shell.

## The cure

1. **A claim about what a person can do from a surface is measured from a
   process started the way that surface starts one.** For the terminal, that is
   a child of `alo-desktop`, under its unit. `grep NoNewPrivs /proc/<pid>/status`
   on the shell is the one-line check; a shell from `podman exec` or `ssh`
   answers a different question.
2. **A limit added to a unit is read against every program it starts**, and the
   desktop starts the person's shell. That is now a test:
   `crates/alo-sessiond/tests/the_units_this_starts_are_on_the_image.rs`,
   `the_persons_terminal_may_use_sudo_and_the_agent_may_not`, which also holds
   the agent's own units to `NoNewPrivileges=yes`.
