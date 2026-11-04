# Native saving for a sandboxed recording app

The owner authorised alo Studio's OS integration and a booted recording walk on
2026-11-04. This change supplies the native save dialog and exact document
permissions. Capture integration is described in
`recording-permissions-and-native-capture.md`. Market publication is separate.

The frontend identifies the bus caller, opens the rented GTK save chooser, makes
an exact file grant, records the answer, and uses the Documents portal to expose
that file to the application's sandbox. It does not grant a folder. A failed
export rolls back its new grant. Native requests are bounded, cancellable only
by their owner, and ended when their caller disappears or the frontend stops.

Document permission bookkeeping points to the exact grant IDs in the existing
list. A grant that is gone or unread withdraws its document permissions. The
bookkeeping file cannot independently authorise an export.

Permanent private-bus checks cover request ownership, disconnection,
cancellation, an unavailable chooser, and failed document export. These use a
controlled picker fixture and do not prove the real native dialog or FUSE
sandbox export. These checks and the scoped portal, capability, remembering,
changing, and capture transport checks passed at `4c3f13ae`; this is not a full
workspace gate.

A one-off integration probe at `67191b9f` used the real GTK chooser, Documents
FUSE export, process identification, and installed `org.aloworld.Studio` Flatpak
on a private host session. The sandbox copied a 25,512-byte generated video to
the exact selected file. Reopening the exported document for writing succeeded;
after removing its grant through the same locked grants store, reopening from
the sandbox was denied. The exported file and private original remained byte
identical, SHA-256
`7b331c332173c7c8dce2d34c7f5a8da28d99e648bcc234be51e030eb96e00877`.
Evidence is local to `/root/alo-studio-native-save.KWXSbb`. This one-off probe
does not run in CI; the two verification clients live in `alo-portald/examples`.

A person reaches this through Studio's Save action. The image recipe installs
the native backend and document services while retaining alo's public portal
authority. The booted recording walkthrough is tracked in the capture report;
this prerequisite alone does not establish booted-system or microphone hardware
acceptance.
