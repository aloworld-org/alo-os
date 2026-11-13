# Flatpak Firefox reads the machine's settings from an extension nothing installs

- **Version:** Firefox from Flathub (`org.mozilla.firefox`, built and published
  by Mozilla), `156.0` in `crates/alo-software/shipped.toml`. The mechanism
  landed in Firefox 115.
- **Whose:** the engine's packaging, and the platform's (bootc).
- **Behaviour:** the usual Linux places for system-wide Firefox settings —
  `/etc/firefox/policies/policies.json`, or `distribution/` and
  `defaults/pref/` beside the program — do nothing for the Flatpak, because it
  runs in a sandbox that cannot see the host's `/etc/firefox` and its program
  directory is read-only. Mozilla's Flatpak instead declares an extension point,
  `org.mozilla.firefox.systemconfig`, mounted in the sandbox at
  `/app/etc/firefox`. **Nothing is installed with `flatpak install`**: an
  administrator makes the directory under the machine's flatpak installation,
  `/var/lib/flatpak/extension/org.mozilla.firefox.systemconfig/<arch>/<branch>/`,
  and Firefox reads `policies/policies.json` and `defaults/pref/*.js` from it.
- **Where that comes from, read once on 2026-11-13** and not re-checked by
  anything:
  - Mozilla's administrator guide, *Configuring policies*, Flatpak section
    (<https://firefox-admin-docs.mozilla.org/guides/policies-configuration/>):
    the Flatpak "cannot read the host's `/etc/firefox/policies` directory", the
    extension is "named `org.mozilla.firefox.systemconfig` in the sandbox at
    `/app/etc/firefox`", and "nothing needs to be installed with
    `flatpak install`".
  - Mozilla's bug 1814897, fixed in Firefox 115
    (<https://bugzilla.mozilla.org/show_bug.cgi?id=1814897>), which says the
    preferences in `…/org.mozilla.firefox.systemconfig/x86_64/stable/defaults/pref`
    are loaded too. The administrator guide does not mention `defaults/pref`;
    the bug is the source for it.
  - The Flatpak manifest itself was **not** read: Flathub's repository for it
    and Mozilla's packaging directory both answered 404 from here.
- **Our response:** alo ships `defaults/pref/alo-data-sharing-starts-off.js`,
  four `pref(…, false)` lines, rather than a `policies.json`. Two reasons: a
  policy **locks** what it sets, which would take the choice away (law 5); and a
  policy makes Firefox say the browser *is managed by your organisation*, which
  on a personal machine is false (ADR 0004). A default pref is what Firefox's
  own Settings shows unticked and lets the person tick.

  **bootc adds a second turn**: what an image puts in `/var` is copied once, at
  install, and no update changes it. So the file lives in `/usr/share/alo/firefox`
  and `image/usr/lib/tmpfiles.d/alo-firefox-shares-nothing-at-first.conf` makes
  the extension path a link to it at every boot (`L`, not `L+`, so a directory
  the person or an organisation put there is left alone).
  `crates/alo-image/tests/firefox_starts_with_data_sharing_off.rs` holds the
  file, the link and the recipe to each other on every gate run, and the image
  build checks the file's lines.
- **Not yet measured:** that flatpak follows a symlinked extension directory
  into the sandbox, and that Firefox's welcome and Settings then show the four
  boxes unticked. Both need a booted machine with Firefox installed, where
  `about:config` shows each of the four as `false` and not modified. The branch is assumed to be `stable`,
  which is the branch Flathub publishes Firefox on.
- **Date:** 2026-11-13.
