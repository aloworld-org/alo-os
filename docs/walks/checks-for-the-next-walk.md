# Checks for the next walk on a booted machine

Things built and tested in the repository whose last half only a booted machine
can show. **None of these has been seen on a machine yet.** A walk that does
one moves it to the walk's own record, with what was seen.

## The time zone

Added 2026-10-28 with ADR 0138.

1. **SELinux accepts the link the first-start screen writes.** Take a region on
   the first-start screens (e.g. Germany). Then:
   - `ls -lZ /etc/localtime` names `/usr/share/zoneinfo/Europe/Berlin`;
   - `ausearch -m avc -ts boot` shows no denial naming `localtime`;
   - `timedatectl` reports `Time zone: Europe/Berlin`.
   The link is labelled as its directory (`etc_t`) and not `locale_t`, which is
   what systemd's own service would give it. If anything is denied, that is the
   finding.
2. **Running services see the new zone.** After the first start, without a
   reboot:
   - the status area's clock shows the zone's time;
   - `journalctl -n 1` timestamps are in it;
   - `date` in a terminal agrees.
   A service started before the zone was set may still show universal time
   until it restarts. Write down which ones do.
3. **The person may change the zone in Settings, and the agent may not.**
   - In Settings, choose another city of the region. `timedatectl` follows it,
     and no password is asked.
   - As the agent's account:
     `sudo -u alo-agent busctl call org.freedesktop.timedate1 /org/freedesktop/timedate1 org.freedesktop.timedate1 SetTimezone sb Europe/Paris false`
     is refused with `NotAuthorized`.

## The first-start screens

Added 2026-10-28.

4. **The overview (frame `09`) and each picker are seen on a screen**, at the
   machine's own size, against the design: Language, Region, Keyboard, the
   which-time-zone sheet for a country with several, and *Test your keyboard*
   typing through a German keyboard while English is read.
5. **The home folders are made in the person's language.** A German person's
   home holds `Dokumente`, `Downloads` and `Bilder`, and
   `~/.config/user-dirs.dirs` names them.
