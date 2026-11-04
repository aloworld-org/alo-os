# Native saving for a sandboxed recording app

The owner authorised alo Studio's OS integration and a booted recording walk on
2026-11-04. This change supplies the save-dialog prerequisite while capture
integration remains in progress. Market publication is separate.

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
sandbox export. Validation is pending. A booted alo recording and microphone
hardware walk have not been completed by this prerequisite.
