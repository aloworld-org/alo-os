# ADR 0139 — The first-start screen sets the time zone itself

**Status:** accepted, 2026-10-28, under the owner's instruction: *find the
honest road to set `/etc/localtime` from a screen that runs as root but with
dropped privileges; if it needs a new privilege, write the ADR, and if that is
ambiguous report back before building.* It needs no new privilege, so this
records the road rather than asking for one.

**Context:** `docs/features.md`, *regional formats and timezones per language*;
[ADR 0137](0137-home-folder-names-come-from-xdg-user-dirs.md) and the
first-start sheets (`docs/design/the-first-start.md`); `crates/alo-formats`,
`crates/alo-shell`; `image/usr/lib/systemd/system/alo-compositor.service`.

## The decision in one line

When a person takes a region on the first-start sheets, `alo-compositor` —
the process drawing those sheets — **replaces `/etc/localtime` itself**, with a
link into the zone database, through `alo_formats::the_machines_zone::set`.
Nothing gains a capability, no helper is added, and no D-Bus service is asked.

## Why it needs nothing new

`alo-compositor` runs as root with `CapabilityBoundingSet=` and
`AmbientCapabilities=` both empty. Without `CAP_DAC_OVERRIDE` it cannot open a
person's `0700` home, but **it owns `/etc` by user id**, and replacing a file
in a directory one owns needs no capability at all. It already writes two
machine files there for the same first-start sheets:
`/etc/alo/keyboards.toml` and `/etc/alo/set-up-in.toml`. `/etc/localtime` is a
third, beside them in the same way.

## The roads considered

1. **The screen writes the link itself** — this decision. No new privilege.
   Covered by tests that set, replace and refuse on a disk a test owns.
2. **`systemd-timedated` over D-Bus** (`SetTimezone`). It authorises a caller
   by `CAP_SYS_TIME` or by polkit. The compositor has neither the capability
   nor a polkit rule written for it, and whether polkit grants a root caller
   with no capabilities by default is not something this repository has
   measured. Giving it either would be a new privilege to set one link it can
   already set.
3. **A narrow helper**, like `alo-giving-a-folder`. A new unit and a new binary
   to do what the caller can already do.

## What is guarded

- **The zone must be one CLDR names**, with no `..`, no empty part and no
  leading `/`, and its file must be a file in the zone database on this
  machine. So nothing but a real zone can be named, and the link cannot point
  anywhere else.
- **Whole or not at all**: the new link is made beside the old one and renamed
  over it. If either step fails, the machine keeps the zone it had, and the
  journal says why.
- **Where a region has several zones** — the United States, Russia, Brazil —
  the person is asked which, on a sheet the design does not draw, from CLDR's
  own zones for that region, named by their exemplar cities in the person's
  language (`alo_formats::zones`). Nothing arrives chosen, and leaving that
  sheet sets no zone.

## What is not measured

- **SELinux.** The link is created by the compositor's domain and labelled as
  its directory; `systemd-timedated` would have labelled it `locale_t`. Whether
  every confined daemon that reads `/etc/localtime` still may has not been
  checked on a booted machine. That check belongs to the next boot walk.
- **Running daemons** read the zone when they next look; nothing tells them it
  changed. On the first start, the person's desktop is started after the
  sheets, so it reads the zone just set.

## Changing the zone later

Settings runs as the person, not as root, so it cannot take this road; and
Settings has no row for a time zone today. So **changing the zone after the
first start is not built**. When it is, it needs a privilege that the person
can use and the agent cannot, and that is a decision for its own record.
