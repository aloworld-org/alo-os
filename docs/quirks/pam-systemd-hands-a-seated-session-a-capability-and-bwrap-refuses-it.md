# pam_systemd hands a seated session a capability, and bwrap refuses it

**Date:** 2026-11-11. **Status:** worked around in `alo-desktop@.service`.

## What happens

Two upstream behaviours, each reasonable alone, meet in the person's terminal.

**`pam_systemd` raises `CAP_WAKE_ALARM` into the ambient set** of a regular
user's session when the session is on a seat, or when the PAM service is
`systemd-user`. systemd v257, `src/login/pam_systemd.c`,
`pick_default_capability_ambient_set`, whose comment gives the reason: *this
allows desktops to install CAP_WAKE_ALARM to implement alarm clock apps without
much fuss.* A unit's `AmbientCapabilities=` (empty) does not undo it: PAM opens
the session after systemd has applied the unit's capabilities. An ambient
capability survives `exec`, so every program started from that session's
process holds it in its permitted and effective sets.

**Fedora's `bwrap` is not setuid, and refuses to run holding any capability.**
bubblewrap 0.10.0, `acquire_privs()`: when the real and effective uid are equal,
the uid is not 0, and the permitted set is not empty, it dies with *Unexpected
capabilities but not setuid, old file caps config?* It checks the permitted set
only.

So in alo's terminal, whose shell is a child of the desktop's process:

```
$ flatpak run com.vscodium.codium
bwrap: Unexpected capabilities but not setuid, old file caps config?
error: ldconfig failed, exit status 256
```

The same application opened from the launcher, which starts it from the
person's own systemd manager. Measured in the walk's journal: `alo-desktop`
logged with `_CAP_EFFECTIVE=800000000` (bit 35, `CAP_WAKE_ALARM`), the launcher's
`flatpak` with `_CAP_EFFECTIVE=0`. The manager holds the capability itself but
does not pass it to its services' ambient sets.

## Three things that do not stop it

- `AmbientCapabilities=` empty in the unit (above).
- `pam_systemd.so default-capability-ambient-set=` with an empty value. The
  parser leaves the setting unset, and unset means the default. Measured in
  `localhost/alo-os:dev`: the session still had `CapAmb: 0000000800000000`.
- Measuring with `systemd-run -p PAMName=...` in a container. A container has
  no seat, so `pam_systemd` does not choose the default and every set reads
  zero. To see the default in a container, use the PAM service name
  `systemd-user`, which takes the same branch.

## What does stop it

Take the capability out of the bounding set: `CapabilityBoundingSet=~CAP_WAKE_ALARM`.
`pam_systemd` cannot raise what the bounding set has dropped. Measured in
`localhost/alo-os:dev` on 2026-11-11 with `PAMName=systemd-user`:
`CapAmb: 0`, `bwrap` ran, and `sudo` still answered `root`. The cost is that a
root shell reached with `sudo` from that session cannot set alarms that wake a
suspended machine.

An empty `CapabilityBoundingSet=` stops it too. That is how the desktop's unit
hid this until 2026-11-09, and it also stops `sudo` (ADR 0150, *The person's
terminal*).
