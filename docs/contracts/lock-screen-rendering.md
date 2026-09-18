# Lock screen rendering

The shell consumes `alo_locking::LockScreen`, whose public view contains only
the moment, appearance-selected background, battery reading, locked statement,
and destination-free egress lamp. `LockSurface` holds a locked `Seat`; private
notifications remain there until the existing `Seat::unlocks` composition
releases them after authentication. There is no alternate authenticator or
new session opener.

`Nested::pump_lock` intercepts input through the existing sign-in seat. Enter
opens the existing name/password fields; submission reads current accounts
through the supplied reader. Every attempt forgets both fields. An unreadable
store displays the greeting's existing refusal and keeps the seat locked.
Focus loss forgets unfinished input. The agent key is refused by lock policy
before a compositor is reached. Hosts must route all input through this pump
while locked; ordinary desktop input pumps are not lock-aware.

`LockBackground::prepare` resolves the policy-selected image. Shipped names
follow [the wallpaper contract](shipped-wallpapers.md). Personal PNG/JPEG files
are explicitly selected; rotations use sorted regular image files and the
appearance model's index. Directory enumeration is bounded to 4096 entries,
compressed input to 32 MiB, each image side to 8192 pixels, image/output area
to 16,777,216 pixels and decoder allocation to 128 MiB. Missing, malformed or
oversized images are refused without substituting a different wallpaper.
Transparent pixels and fitting margins are composited onto opaque black.
The five appearance fitting modes are preserved.

The caller prepares the background on output, appearance and rotation changes
and obtains a fresh policy snapshot for each frame. Civil time comes from
Jiff's bundled timezone database using the explicitly selected zone;
`alo-formats` supplies CLDR wording. No network or host-zone guess is used.
Unknown zones and unrepresentable clock values refuse the frame.

`Nested::submit_lock` uploads one opaque texture. The shared scene boundary
returns through the exclusive lock painter before importing any client or
painting private native layers. No client cursor is drawn. Name/password
fields reuse sign-in rasterization in a separate middle region; password
length never changes the drawn band. Lock text has an opaque token-based
contrasting ground; egress retains its mark and count without destinations.
Outputs smaller than 320 by 480 pixels or unable to hold complete text refuse.
On frame-layout failure the nested surface submits an empty opaque
frame; if the graphics backend itself fails, the host must retire the output
and retain the locked session. It must never resume the desktop on an error.

These are nested compositor interfaces, not a booted session manager or
direct-display lock implementation. This change does not install a compositor,
wire physical suspend/resume, or certify a laptop. The shell plan's hardware
acceptance remains open.
