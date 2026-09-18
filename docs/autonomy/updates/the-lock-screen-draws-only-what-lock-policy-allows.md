# The lock screen draws only what lock policy allows

Date: 2026-09-18. Owner-assigned shell work, task 8, implemented manually in
`C:/dev/alo-os-claude`. This is the code and nested compositor surface, not
certified-machine acceptance or a booted graphical session.

## What changes

A locked surface draws the selected picture, regional time, battery reading,
locked statement, and the existing destination-free egress lamp. Private
notifications stay in the lock policy's seat. The locked scene takes an
exclusive path before client import, private native layers or cursor drawing.
The opaque RGBA image is uploaded as one texture; a photograph is not expanded
into millions of separate text-painter draw calls.

Enter opens the existing sign-in fields. The shared entry handles editing and
zeroing; the shared raster never exposes password length. Every submission
reads current accounts and delegates to `Seat::unlocks`, which already uses
`Greeting` and the original locked session. Wrong passwords, unknown names,
other accounts, and unreadable account storage keep the session locked and
forget the fields. Focus loss forgets unfinished input. The agent key reaches
lock policy before it could reach any overlay compositor.

The approved Quiet Horizon bytes, already published in c69a044, are installed
at `/usr/share/alo/wallpapers/alo.png` with explicit mode 0644. The recipe test
derives the default filename from `alo-appearance::THE_WALLPAPER`. The owner
explicitly assigned this narrow image handoff here; remaining installer/image
work stays with the third PC. No replacement artwork or default is selected.

Image selection, decoding, fitting, pixel composition, battery drawing, clock
conversion, input state, nested input and GPU submission have separate files.
PNG/JPEG decoding has compressed-size, dimension, pixel and allocation limits;
rotation enumeration is bounded, sorted and uses appearance's chosen index.
All five fitting modes are supported; transparency and margins are opaque.
Missing artwork and invalid geometry refuse instead of silently substituting
an unchosen image. A layout failure blanks the nested output; a backend
failure must leave the host locked and retire the failed output.

Time conversion uses the unmodified Jiff engine with a bundled timezone
database, explicitly selected by the person's timezone. CLDR wording stays
in `alo-formats`. No network lookup or host-timezone guess occurs. The existing
formatting API accepts hour/minute and fixes seconds at zero; finer clock
precision remains that API owner's responsibility.

The existing source-level password test now derives credential carriers
transitively from `TypedPassword`, instead of extending another fixed name
list. Its checks still reject Debug/Display/Clone roads from every carrier.
No authentication policy, rented engine, gate or refusal was weakened.

## Validation and practical limits

Scoped Clippy, lock/input/decoding/raster tests, the sign-in source safety
checks and wallpaper recipe test passed during development. Pixel comparisons
use the approved picture and compare different private notifications, agent
names and egress destinations byte for byte; real egress changes the pixels.
Other checks cover wrong/unknown/other-account credentials, unreadable stores,
held notification release, password-length privacy, daylight-saving conversion,
unknown zones, fitting, transparent/malformed/oversized images and rotation.
The final full-gate results and exact acceptance evidence accompany the
pull request; merging into main requires all nine gates on the exact combined
tree. An earlier full run stopped because the source-safety test forbids
`File::options`, even for a read-only request. The decoder now opens with
explicit OS read-only, nonblocking and close-on-exec flags. The unchanged
Settings source checks, sign-in safety checks, Clippy and four background
tests pass after that repair. New checks verify unchanged image bytes and
refusal of directories, FIFOs and oversized files. Earlier failures remain
in the local gate logs; they are not counted as passing validation.

The development probe is `cargo run -p alo-shell --example lock_screen` under
a Wayland parent. It submitted ten frames through the real nested renderer.
WSL's driver selection emitted Mesa/EGL warnings before successful submission;
that is not evidence of a certified GPU. The final probe additionally exercises
opening the reused credential form and refusing a mismatched background.

No image build, installation, direct-display submission, certified keyboard,
suspend/resume path or physical laptop has been verified here. The image
recipe still does not install a compositor. Hosts must route locked input
through `pump_lock`, update policy snapshots and refresh prepared backgrounds
when output, appearance or rotation changes. No English Enter-key hint was
invented in the drawing crate; the lock vocabulary currently supplies only
the locked statement. The booted session and hardware acceptance remain open.

Contracts: `docs/contracts/lock-screen-rendering.md` and
`docs/contracts/shipped-wallpapers.md`.
Engine API references: [Jiff timestamps](https://docs.rs/jiff/latest/jiff/struct.Timestamp.html)
and [image decoder limits](https://docs.rs/image/latest/image/struct.Limits.html).

Proposed changelog: the nested shell can display an opaque, privacy-limited
lock screen using the approved wallpaper, and unlock through the existing
sign-in composition. The image recipe now carries the default wallpaper.
No new plan task is appended and no hardware/release checkbox is closed.
