# A separate native application for recording demos

The owner asked on 2026-11-04 for a complete, user-friendly demo recorder that
people can obtain through the application market. They confirmed screen or
window recording, optional microphone audio, preview, and saving. The chosen
scope adds no webcam, account, upload service, or video editor.

This makes the existing screen-recording promise usable as a separate
application, because capture needs a clear entry point a person can install
and open independently of the operating system. The source is its own Cargo
workspace at `apps/alo-studio`, not a new capability in the agent's registry.

## What the person gets

alo Studio is native Rust with GTK for its interface, the standard desktop
permission picker for the selected source, and GStreamer for capture and
encoding. It previews the selection, checks the chosen size can encode before
recording, counts down, pauses and resumes, then saves a finalized original
into a private library. The person can play it back, rename it, search the
library, save a copy using the system file chooser, or deliberately delete it.

The quality check is advice: a slow result offers another size or an explicit
choice to record anyway. It does not forbid the person's chosen size. This was
changed after the final UI check exposed how variable CPU contention can be.

The microphone is off for every new recording and is never opened by a
preview or quality check. A running recording that fills its bounded encoder
queue stops and saves what it has instead of silently lowering quality. The
format is VP9 with optional Opus in WebM, following ADR 0051. No codec is
implemented here, and the installed Flatpak has no network, home-folder, or
camera permission.

The layout uses quiet surfaces, explicit labels, readable hierarchy, and one
primary action. Review actions remain visible, and video metadata arriving
cannot move the Save button. This is native GTK configuration, not an HTML
surface or a replacement shell renderer.

## Evidence and its limits

The separate app's formatting, clippy with warnings denied, and eight lifecycle
and storage tests passed on WSL Ubuntu 24.04 on the Windows host AGAI01. The
tests exercise unavailable source refusal, cancelled countdown, pause timing,
microphone reset, malformed identities, empty output, and overwrite refusal.

An actual Xvfb framebuffer was encoded, paused, resumed, finalized, and decoded
to EOS as VP9 and synthetic Opus/WebM. No physical microphone was opened. The
native UI was also driven through selection, preflight at explicitly selected
720p, countdown, recording controls, completion, and a persisted library entry.
These checks run again in `.github/workflows/studio.yml`; the diagnostic source
and its environment variable do not exist in an ordinary installed build.

A hand-operated native UI walk also opened a recorded video for playback and
saved a byte-identical copy through the file chooser while retaining the
original. Those export clicks were a one-off check, not yet a CI assertion.
Another one-off UI check cancelled deletion and retained the original, then
confirmed deletion and verified the exported copy remained unchanged.
Native screenshots and a recording of that walk are kept as local evidence.

The Flatpak was built from vendored, locked Rust dependencies inside the GNOME
49 SDK, and AppStream composition and desktop-entry validation succeeded.

An additional one-off `verify-portal` diagnostic reached CreateSession,
SelectSources and Start on isolated Sway 1.9 with xdg-desktop-portal-wlr 0.7.1.
Start refused before supplying a stream: the backend reported no valid
screencopy format with the software pixman renderer. An independent `grim`
screenshot worked. This resembles the backend's documented
[pixman limitation](https://github.com/emersion/xdg-desktop-portal-wlr/issues/289),
but it does not establish a successful portal recording. That diagnostic is
built by CI and requires a functioning capture desktop to run; it is not
reported as a passing recording check on this machine.

## What is still required before market availability

**alo OS does not currently serve `org.freedesktop.portal.ScreenCast`.**
`alo-portals` contains capture policy and `alo-capturing` can consume a stream,
but `alo-portald` does not provide the stream-selection and publishing service.
It also does not serve the file chooser this application's export uses. No
test above is presented as a successful recording of alo OS through its portal.
That OS integration and a booted-machine walk remain required.

**A package is not a marketplace listing.** A reviewed build must be published
to an actual signed Flatpak source. `packaging/market-entry.toml` is the proposed
entry for the existing Flathub source, deliberately kept outside the live
catalogue until the identifier can really be installed. No signing key was
created and no imaginary Flathub release was added to the market.

The complete OS workspace suite, kernel/BPF tests, image build, and physical
hardware acceptance were not run for this independent application. This report
moves no operating-system release or hardware acceptance checkbox.
