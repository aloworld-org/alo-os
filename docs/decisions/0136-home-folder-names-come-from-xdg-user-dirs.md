# ADR 0136 — A person's home folders are named by xdg-user-dirs, in their language

**Status:** accepted, 2026-10-28, by the owner: *home folder names follow the
person's language, the Linux way.*

**Context:** `docs/features.md` (*be ordinary everywhere it is not the
point*, and the shell in all 24 official EU languages); `crates/alo-their-folders`;
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (what is
rented); `docs/contracts/person-settings.md`, `[reading]`.

## The decision in one line

A person's Documents, Downloads and Pictures folders are made with the names
**`xdg-user-dirs`** gives them in the person's reading language, recorded in
the standard `~/.config/user-dirs.dirs`, and **a folder the record already
names is never renamed** — a rename is offered, never forced.

## What it was before

`alo-their-folders` named the three folders through alo OS's own vocabulary:
three words, `their-folders.documents` and two more, for alo's translators to
translate. No translations ship, so every machine made English names, and the
program did not ask what language the person reads at all. And it rewrote the
record at every sign-in, so a person who changed language would have been given
a second, translated folder beside the first.

## Where the names come from, and why there

`xdg-user-dirs` is the package every Linux desktop uses to name these folders
and write `user-dirs.dirs`. Its catalogue is kept by that project's
translators for some seventy languages, and it is what GNOME, KDE's Dolphin,
browsers and file choosers all expect a German home to look like. Using it
means **alo OS and every other program agree** about where a person's documents
are, on the first day and after every update of that package. Three words of
our own would be a second answer, wrong first in the languages nobody here
reads.

The catalogue is read as data — `/usr/share/locale/<language>/LC_MESSAGES/xdg-user-dirs.mo`,
GNU gettext's format, by `alo_their_folders::catalogue` — rather than by
running `xdg-user-dirs-update`, for two reasons measured on 2026-10-28:

- the image has only `glibc-minimal-langpack`, so the updater, which asks the
  C library for the locale, would run in the C locale and make English names;
- the updater decides for itself when to add folders, and alo OS has decided
  which three a person gets (`crate::folders` says why).

So the package provides **the names**; `alo-their-folders` decides **which
folders and when**. The updater's own user service is masked in the image.

## Which language

The first language in the person's own `settings.toml` `[reading]`; where
that names none, the language the machine was set up in on its first-start
sheet (`/etc/alo/set-up-in.toml`). That is the rule the person's desktop
gives them their reading language by, asked by `alo-their-folders` itself
because both run at sign-in and neither waits for the other.

## What the image has, measured once on 2026-10-28

Measured by hand in `localhost/alo-os:dev` and the pinned Fedora 42 base, and
not re-measured by any build step except the one below:

- `xdg-user-dirs` was **not installed**; it is now, in `image/Containerfile`.
- `%_install_langs` is `all`, so its catalogues are kept: 74 of them in 0.18.
- **22 of the 24 official EU languages have one.** English is the catalogue's
  source and needs none. **Maltese has none**, so a Maltese person's folders
  are named `Documents`, `Downloads` and `Pictures` — which is what
  `xdg-user-dirs` gives a Maltese person on every Linux desktop. That is not a
  fallback of ours, and it is the one gap; the remedy is a Maltese translation
  contributed to `xdg-user-dirs` upstream.

**What runs on every build**: the image's recipe runs the real
`alo-their-folders` against the real catalogue for a person whose settings say
German, and fails unless `Dokumente` is made and recorded. The unit and
acceptance tests run on every gate against catalogues they write themselves.

## A folder that already has a name keeps it

The record is read before anything is made. A folder it names is that folder,
in whatever language it was made in: made again if it went missing, never
renamed, never doubled. A key set to `$HOME` alone — the format's way of
saying *this person has no such folder* — is honoured. Lines this program does
not own are kept as they were.

When the person's language now calls a folder something else, the program says
so on the journal (`Made::named_differently`). **Offering the rename needs a
screen, and that screen is not designed**; until it is, nothing renames
anything, which is the half of the decision that can be kept without one.

## Consequences

- `alo-their-folders` depends on `alo-choosing` and `alo-formats` for the
  language, and its three folder-name words are retired (ADR 0068: a retired
  key needs no declaration).
- Everything else in alo OS reaches these folders by what is on the disk — the
  picker lists the home folder, and a grant is made from what was picked — so
  a German person's `Dokumente` is listed and granted by that name. That is
  held by `crates/alo-their-folders/tests/a_person_gets_their_folders.rs`.
