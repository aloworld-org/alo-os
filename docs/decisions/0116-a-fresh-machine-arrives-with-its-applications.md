# ADR 0116 — A fresh machine arrives with its applications, rather than downloading them while somebody waits

**Status:** proposed, 2026-10-14, by the third-PC lane, from a measured boot
and from an unverified inference that is marked as such below.

## What a person meets today

A machine written from `05d4871d` was booted and a person made the first
account on it. The sign-in screen says, in its own footer:

> Local setup. No online account required.

While it said that, the machine was downloading applications from flathub, and
`systemd` recorded what that cost:

```
Startup finished in 11.474s (kernel) + 46.326s (initrd)
                  + 43min 53.694s (userspace) = 44min 51.495s
```

**The person is typing their name during those 44 minutes**, and the machine is
too busy to take the keys. This was watched rather than inferred: `d i s a n`
arrived, four PIN digits did not, and a later attempt produced one digit of
four. The account could not be completed until the install finished.

Part of that is the emulator — four TCG-emulated processors against a large
download — and on real hardware the contention is milder. **The product
argument does not depend on the hardware:** a machine that needs the network to
finish arriving is not doing local setup, whatever the footer says.

## What is actually being downloaded

`crates/alo-software/shipped.toml` names **seven** applications. What lands is
far more than seven, because each flatpak declares a runtime and the runtimes
are most of the bytes.

**Measured by the Mac lane** against `https://flathub.org/api/v2/summary/{ref}`
for x86_64 on 2026-10-14:

```
the seven applications      download   218M    installed   578M
the runtimes they require   download  1914M    installed  5021M
                            ------------------------------------
                            download  2132M    installed  5599M
```

**Ninety per cent of the download is runtimes nobody chose** and that nothing in
this repository names.

### Two caveats that belong in the record, not in a footnote

1. **Those are flathub's current branches, not the versions this repository
   pins.** `shipped.toml` pins Firefox 156.0, Dolphin 26.04.3, Ark 26.04.3,
   TextEditor 50.1, Loupe 50.0, Papers 50.2. The figures above describe what
   flathub serves now.
2. **ostree deduplicates shared objects**, so the real download and the real
   disk are both lower than those sums by an amount nobody has measured.

**The structural claim survives both:** seven applications, five runtimes, and
most of the bytes in things nobody selected.

### An inference this record does **not** yet assert

The same measurement suggested two runtime pairs disagree — Dolphin on KDE 6.10
beside Ark on 6.11, Ptyxis on GNOME 50 beside three applications on 51 — which
would be roughly 830M spent fetching second copies.

**That is not yet a fact about this product.** `shipped.toml` pins Dolphin and
Ark to the *same* release, 26.04.3, so a split between them cannot be read off
our own list; it was read off flathub's current branches. And a flatpak's
runtime is declared by that flatpak's build, so it is not ours to change
except by choosing different application versions.

**It was reported to me as a two-line fix and it is not one.** Recording that
distinction is the point of this section: the saving may be real and the
mechanism is not what it appeared to be.

## What settles it

`alo-readings.service` already prints to the console at every boot and needs no
login — which matters, because `serial-getty@ttyS0` is masked in this image
deliberately, and *a diagnostic facility that quietly adds a way in is a worse
bargain than no diagnostics.* It now also prints:

- every ref with its size and installation, applications **and** runtimes;
- what each application names as its runtime, which is what pulls them;
- where flatpak keeps things, and the size under `/var/lib/flatpak`.

So the next boot answers the runtime question with what a machine did, rather
than with what an API said, and keeps answering it afterwards.

## The decision

**A fresh machine arrives with its applications already on the disk.** Nothing
is downloaded between a machine being switched on and a person being able to
use it.

Nobody else downloads applications during setup — macOS, Windows, Android,
Ubuntu, Fedora and SteamOS all put them on the disk first — and for a product
sold on sovereignty the argument is stronger than convenience: a setup that
requires flathub is a setup that requires somebody else's server to be up.

### The mechanism, and the one thing that would sink it

`bootc` separates `/usr`, which is the image, from `/var`, which is machine
state — and flatpak installs into `/var/lib/flatpak`. **That is very likely why
this was done at first boot**, and it is the whole difficulty.

`flatpak-installation(5)` allows a second system-wide installation declared in
`/etc/flatpak/installations.d/*.conf` with a mandatory `Path=`, and that path
can be inside `/usr` and therefore inside the image.

**What is unverified is whether flatpak writes into that directory at
runtime.** If it does, the approach fails against a read-only `/usr`, and the
answer is not in the manual page — it is in a boot. The readings above are
written so that boot answers it.

## Until then

**`alo-shipping.service` sets no scheduling priority at all**, so flatpak's
downloads compete with the compositor on equal terms and a person loses the
race for their own keystrokes. `CPUWeight=`, `IOWeight=` and `Nice=` make the
person win without changing what is installed. That is a mitigation and not
this decision, and it should land whether or not this decision is accepted.

## How this is known to be done

On a machine, from the readings and from `systemd`'s own line: a fresh boot
reaches the sign-in screen with its applications already present,
`the_applications_arrived: yes`, and **no flatpak download between power and
the person.** Userspace startup measured in seconds rather than in
three-quarters of an hour.
