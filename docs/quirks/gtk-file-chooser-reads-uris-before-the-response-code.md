# GTK reads the file list before checking cancellation

- **Version:** GTK 4.20.4 in Studio's private installed alo OS walk, source
  `f5f0f0ea6d382ee80b7e86c71de134ee6f679a9b`, run 44448153799. A separate local
  consumer experiment used GTK 4.14.5 on WSL Ubuntu with Xvfb and a private bus.
- **Whose:** GTK's response reader and alo's FileChooser reply compatibility.
- **Date:** 2026-11-19 UTC.

## Behaviour

The installed walk completed screen recording, pause/resume and review, then
pressed **Cancel** on the native Save a file sheet. Studio terminated with
SIGSEGV in `libgtk-4.so.1.2000.4+0x38385b`. The controller subsequently reported
`ServiceUnknown`. The session had recorded the proposed destination but had
not saved or uploaded the video. The retained evidence artifact has SHA-256
`37f7b0f1291af0ed6006c75b6285229f9902d01e1dc5d3363054fd6601d088bf`.

In [GTK 4.20.4's response callback](https://github.com/GNOME/gtk/blob/4.20.4/gtk/gtkfilechoosernativeportal.c),
`uris` is an uninitialized pointer. The callback does not check whether
`g_variant_lookup` found that field, and iterates the list before inspecting
the response code. alo previously included `uris` only when a file was
successfully handed over. Cancellation and early refusals sent an empty map.
This is a concrete unsafe consumer path; the installed crash's attribution
still requires the corrected installed walk, rather than treating a source
trace alone as that proof.

A **one-time local experiment**, using a real `Gtk.FileDialog` and a private
portal emitting a unicast cancellation response, hung with the field absent
on GTK 4.14.5. Its native callback remained inside the main loop and the owned
probe was terminated. With an explicit empty string array, the callback
returned cancellation and exited successfully. This was a different GTK
version and a hang, not a reproduction of the installed SIGSEGV. It is not a
recurring CI check.

## What alo OS does about it

`Asked::responded_with` supplies an empty string array (`as`) when a
FileChooser reply has no `uris` field. It preserves successful file addresses,
response codes, the record-before-response order and the requesting bus
recipient. Other portal interfaces keep their existing reply maps.

The recurring integration tests in
`crates/alo-portals/tests/a_file_is_chosen_on_alos_own_sheet.rs` require typed
empty file lists on cancellation and refusal, including GTK-shaped SaveFile
options and failures to hand over a chosen save destination. They also check
successful addresses, records and that cancellation exports nothing. The
existing OpenURI integration tests require an empty map, guarding the boundary
between interfaces. The installed Studio walk presses Cancel, verifies that
no destination was created, reopens the chooser, saves and compares the bytes.
That fresh installed check remains required after this correction.
