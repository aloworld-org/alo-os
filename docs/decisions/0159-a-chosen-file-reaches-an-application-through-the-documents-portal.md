# ADR 0159 — A chosen file reaches a sandboxed application through upstream's documents portal

**Status:** accepted, 2026-11-17. The coordinator chose option A on the owner's
decision of the same day (*"Yes, build it"*, alo's own file chooser).
**Date:** 2026-11-17

## What it changes for the person

When a person chooses a file in alo's file chooser, **the application that
asked can open that file, and only that file**, even an application whose
sandbox reaches none of their folders. Before this, alo had no way to hand a
sandboxed application a file it could not already see.

## The problem

A Flatpak application sees only what its sandbox lets it see. A path such as
`/home/anna/march.txt` means nothing to a sandbox that was never given it.
Every Linux desktop solves this the same way: the **documents portal**
(`xdg-document-portal`) is a small FUSE filesystem at
`/run/user/<uid>/doc`, and Flatpak mounts each application's own view of it
inside the sandbox. The file chooser *exports* the chosen file into it for that
one application, and hands the application the exported path.

alo OS had no documents portal. `alo-portald` owns `org.freedesktop.portal.Desktop`
itself (it is the frontend, not a backend behind `xdg-desktop-portal`), and the
image installs no `xdg-desktop-portal` package (`alo-handing`'s *the documents
portal export is decided, not performed*).

## The options

- **A — upstream's documents portal, its frontend masked.** Fedora's
  `xdg-desktop-portal` package carries two programs: the frontend
  (`xdg-desktop-portal`, which would take `org.freedesktop.portal.Desktop`)
  and the documents portal (`xdg-document-portal`, which owns
  `org.freedesktop.portal.Documents`). Install the package, **mask the
  frontend's user unit**, and let `alo-portald` export a chosen file with
  `Documents.AddFull` (to open) and `AddNamedFull` (to save), as upstream's own
  frontend does.
- **B — hand back the real path** only when the application's own package
  already reaches it, and refuse otherwise. Honest, but a sealed application
  could never open a file the person chose.
- **C — write a documents portal in Rust.** A FUSE filesystem with its own
  permission store; the largest by far, and it would reimplement a security
  boundary that already works.

## The decision

**A.** It is a pinned upstream engine, **configured and never patched**, which
`CLAUDE.md` allows (*Engines are configured, never patched*). What is
configured:

1. The package `xdg-desktop-portal` is installed from the base's own
   repositories, with `install_weak_deps=False`.
2. **The frontend is masked**: `systemctl --global mask xdg-desktop-portal.service`.
   Its D-Bus activation file is left as shipped, and it names that unit, so
   even an application asking for `org.freedesktop.portal.Desktop` before
   `alo-portald` is up cannot start it. `alo-portald` keeps the name. The image
   test `crates/alo-image/tests/a_chosen_file_reaches_the_application_that_asked.rs`
   holds the mask, and the build checks it took.
3. The documents portal starts on demand, by D-Bus activation, as upstream
   ships it. The build **starts it** (`--help`), so a documents portal that
   would die at launch fails the build.
4. `alo-portald` asks it exactly as upstream's own file chooser does:
   `AddFull` to open, `AddNamedFull` to save, with upstream's flags and
   permissions (`crates/alo-portald/src/the_documents_portal.rs`).

## What it costs

- One more upstream program on every machine, running as the person, holding a
  FUSE mount. It reaches only files that were exported into it.
- **An export is a grant to one application over one file**, kept by the
  documents portal's own permission store. alo's record names every export,
  and a file the person did not choose is never exported. The refusal is tested.
- The documents portal's store survives a restart, as upstream's does: an
  application given a file may open it again later. That is what *recent
  files* in every application relies on.
