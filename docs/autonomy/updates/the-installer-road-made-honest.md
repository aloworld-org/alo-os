# The installer road made honest

**Date:** 2026-10-20
**Workstream:** the installer and the boot environment
**Contributor:** Claude Code, Mac lane, for the owner
**Status:** six changes; one merged, five in flight and green. **Nothing here
has installed anything on a real machine.** What is proved is in the table
below, and what is not is said at the end.

## What this was about

The road from Windows is the product's front door: *download, click, reboot,
sign in.* Six things on it were wrong in the same way — **a check that cannot
fail, or a sentence a person cannot act on.** None of them was a missing
feature. Each was something that already existed and did not work, or worked
and said the wrong thing.

| | what it was | where |
|---|---|---|
| 1 | the boot environment could not join a wireless network | #847, merged |
| 2 | the Windows half never said this machine needs a cable, and shrank the disk first | #848, merged |
| 3 | an environment too big for its area told the person to download again — a loop | #851 |
| 4 | the test that proves the Windows reads work did not compile, and no gate compiled it | #853 |
| 5 | the environment had one sentence for three different network faults | #854 |
| 6 | `dracut` exits 0 for a path it cannot find, recorded where the next person looks | #852 |

And one decision written up for the owner rather than guessed at: **ADR 0128**,
may the installer write a Wi-Fi password to the staging partition (#850, merged
as *proposed*).

## The one finding that matters most

**`dracut` exits 0 for an `install_items` path it cannot find.** Measured in the
pinned base with a deliberately absent path: it prints nothing a build reads,
writes the initramfs without the file, and returns success.

`image/installing/alo-installing.conf` is a list of thirty paths the boot
environment must carry. **Every one of them could have been wrong and nothing in
this repository would have noticed.** Two were: `/etc/containers/containers.conf`
and `storage.conf` had been named since the day the list was written and were
never in any initramfs, because Fedora ships those two defaults in
`/usr/share/containers/` — which the list already names beside them. Every
install that ever worked had worked without them.

It also explains the list's own history. Its header says each path was found
*one refusal at a time, by booting the environment*, and that is because booting
a machine was the only instrument anybody had.

The list is now held to the initramfs it builds: `lsinitrd` after `dracut`,
every path aboard, every module with a `.ko` aboard, and a pattern that matches
nothing refused rather than silently expanding to itself.

## What was measured, and where it will be measured again

**Say which, because a report cannot tell them apart a week later.**

| measurement | once, or forever |
|---|---|
| the pinned base has no supplicant, no Wi-Fi plugin, no `iwlwifi` firmware, no `regulatory.db` | **once**, 2026-10-20, 145,474 paths listed with `/usr/bin/nm-online` as a control |
| `dracut` exits 0 for an absent path | **once**, in the pinned base |
| the initramfs carries every path and module its list names | **forever** — a step in `image/installing/Containerfile`, and it printed its verdict on the candidate build of 2026-10-20 |
| the environment installs the five wireless packages | **forever** — `the_recipe_installs_the_wireless_parts_the_base_lacks` |
| `alo-installer` compiles for Windows, tests included | **forever** — a gate step; 34 s on the hosted runner, and it checked `alo-portals` |
| `reading_this_windows` runs against a real Windows | **forever** — a candidate step. It had existed since 2026-09-15 and nothing had ever run it |
| `ip -json link`'s flags for a pulled cable | **once**, with a veth pair in a container's own network namespace |
| 229 MiB staged against the area's 1008 MiB | **once**, from the published candidate |

## What a real build and a real artefact said

The candidate of 2026-10-20 (`candidate-2026-10-20-6b4dfc5`, built from main
`6b4dfc5b`) went green on both jobs, and the two new verdicts printed:

```
the environment can join a wireless network: a supplicant that runs,
NetworkManager's Wi-Fi plugin, the certified machine's own firmware and
the regulatory database
the initramfs carries every path and every module its list names
```

Then the published zip was downloaded and its initramfs unpacked
independently of the build's own check. Every path is there:
`usr/bin/wpa_supplicant`, `usr/bin/wpa_cli`, `usr/bin/nmcli`,
`usr/bin/dbus-broker`, `etc/dbus-1/system.d/wpa_supplicant.conf`,
`usr/lib/firmware/intel/iwlwifi/iwlwifi-3168-29.ucode.xz`,
`usr/lib/firmware/regulatory.db`, and `iwlwifi.ko`/`iwlmvm.ko` — 3,790 entries
in all.

## Three mistakes of mine, because the method is the point

**A mutation harness that could not report a miss.** It tested whether the
output contained `failed`, and `" 0 failed;"` contains `failed`. It reported
eight mutations caught; **two were misses.** One guard was satisfied by a
comment above the `dnf` line naming a package the line no longer installed — a
mention standing in for an invocation. The other was satisfied by an `exit 1`
three lines up in a different branch.

**A harness that did not recompile.** The replacement wrote the file and ran
`cargo` in the same second, so nothing rebuilt and the verdict was about the old
code. It now sleeps, verifies the mutation landed, **requires `Compiling …` in
the output**, verifies the revert landed, and takes a control run first.

**A belief about dependencies, where a measurement was cheap.** I thought
`alo-saying`'s dependency on every crate made `cargo check -p alo-installer`
cover the workspace for Windows. It reaches **17** of 122 crates and **not**
`alo-portals` — the crate whose Windows break shipped. With `--all-targets`, 87
and that crate included. A guard written the obvious way would have passed on
exactly the tree that broke.

Also: I stole a doc comment twice by inserting before a `pub const` instead of
before its doc. Clippy found it both times. And I read a test suite as green
because `head -20` filled with `ok` lines before the failure — the same family
as never tailing a gate's output.

## What the desktop lane found that changed a design

`nmcli` prints a device's state as a **translated string**: its gettext domain
is `NetworkManager`, the image ships the catalogues, and a person picks their
language at first boot. I was about to parse `unavailable`. It would have worked
on an English machine and matched nothing on a German one, falling through to
the same single sentence while looking like progress. #854 reads the kernel's
own flags instead, which are not translated.

## What is owed, and by whom

- **ADR 0128 is the owner's.** May the installer write a Wi-Fi password to the
  staging partition? Four options and a recommendation are in it. Until that is
  answered the road from Windows is **wired-only**, and #848 is what makes it
  fail where a person can still act.
- **Image 0.0.8 is unsigned.** The recipe labels it, `pinned.toml` declares
  `next = "0.0.8"`, and 0.0.7's digest is pinned — the state for a release built
  and not yet signed. #847 touched only `image/installing/`, so 0.0.8's digest
  is unaffected and still signable.
- **The removable-media road is named and not specified** (ADR 0023). Two items
  of this lane's own list — a local `--source-imgref`, and
  `cosign verify --local-image` — only make sense on it, so they are parked
  rather than built toward an unwritten specification.
- **Nothing here has installed alo OS on a machine.** The environment is
  configured for wireless and unproven on hardware until the certified machine
  associates. Those two things belong in the same sentence and with the same
  weight.
