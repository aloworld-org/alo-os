# Recording permissions and native capture

The owner authorised OS integration and a booted recording walk for alo Studio.
This work is on `app/alo-studio`; market publication remains separate. Booted
acceptance is in progress and is not established by the host checks below.

Current booted status: the recording controls respond and a file is saved, but
decode found only one frame for a longer recording. This is not an accepted
recording. The native CI probe now uses the actual descriptor-preparation helper
and requires continuous frames, rather than consuming an ordinary socket. The
selected node also receives PipeWire's explicit link permission, while all
other current and future registry objects remain hidden. This follows the
scoped purpose of `PW_PERM_L` in PipeWire's
[permission header](https://github.com/PipeWire/pipewire/blob/1.4.11/src/pipewire/permission.h).
Validation of that correction is pending. Review visibility after finishing is
also still under investigation.
The recording bar now hides the workspace only after taking focus. Completion
reopens the workspace and keeps the controls until review has regained focus,
avoiding the compositor's persistent minimized state. GTK's application name is
set to `alo Studio` for native dialog titles. Validation of this UI handoff is
pending.

The first booted source request reached the real GTK picker but its unparented
dialog opened behind Studio. The follow-up exports the actual caller window,
adds the compositor's standard XDG parent protocol and admits a new dialog only
when its explicit parent holds focus. Wire checks cover the focused parent,
background parent, revoked export and unknown handle. All 12 parent/focus wire
checks and the full 413-test client-lifecycle suite passed at `530994a9`.
The installed app's chooser now opens in front on the private booted desktop.
Picker labels use display size and packaged application
names with authenticated identifiers rather than compositor device names.

The public ScreenCast frontend identifies the Flatpak caller through the bus,
binds its session to that unique connection, and asks the GTK backend for a
fresh source choice. It makes a session grant on the existing permissions list,
judges that exact grant through the ordinary capability policy, and records the
answer before starting its publisher. Restored and persistent choices are
refused. A session ends on closure, disconnection, revoked permission or a lost
source. Startup removes abandoned application capture grants after obtaining
the public frontend name; ordinary file and agent permissions remain intact.

The signed-in compositor alone offers the capture endpoint. The socket's peer
must be the living process behind the public portal bus name. Output and window
handles follow compositor and mapping lifetimes. Application windows are drawn
into separate targets with only their own descendants and popups. The window
picker currently lists Wayland application windows; native shell windows are
included when sharing a complete screen. Source resize ends a fixed-size stream
and requires a new selection. Lock, seat loss and shutdown invalidate capture
before scanout cleanup, including cleanup refusal.

Frame slots retain only their latest immutable frame. The renderer uses a
nonblocking handoff; socket writers and native media processes run elsewhere.
Separate native-libc workers publish frames and prepare the selected-node
PipeWire descriptor, while the public daemon remains compatible with its static
image build. Each publisher has a persistent supervising thread and a kernel
parent-death signal. ADR 0146 and the rented binding's change ledger document
the two safe upstream API additions used for descriptor preparation.

The desktop indicator reads the actual media graph. A portal-stamped screen
publisher identifies its authenticated recipient even when the consuming media
node omits that identity. Studio subscribes to session closure before starting
preview. A lost source stops video and microphone inputs, then drains queued
media into a final WebM trailer, including when recording was paused.

Checks and their scope:

- Studio's scoped formatting, clippy, eight model/storage checks, actual GUI
  flow, framebuffer recording with generated audio, and PipeWire preview-to-
  recording handoff passed at `7d0782ca`. The additional permanent native probe
  ends sharing while paused and decodes the saved VP9/Opus clip. Local evidence:
  `/tmp/tmp.RW4W0BXY2Y` and `/tmp/tmp.rnib8AdSuQ`. Generated audio is not evidence
  of a physical microphone. The complete scoped gate passed again at
  `338a0544`, including the new caller-window export; local evidence is
  `/tmp/tmp.CIjA2mny2r` and `/tmp/tmp.hf6ThuF1Lg`.
- A one-off private PipeWire check at `b7e2a272` exposed only the selected node,
  hiding an existing unrelated node and another created after handoff.
  `/root/alo-studio-restricted-remote.2E2KYY` holds its local evidence.
- A one-off native worker check used the provider built from `500c8330` and the
  unchanged remote helper. After the descriptor-preparation process exited, a
  real GStreamer client consumed it, encoded VP9/WebM and decoded to EOS.
  `/root/alo-studio-capture-provider.920YB2` contains the video and active graph.
  This exercises generated input, not compositor pixels or a booted desktop.
- Permanent real-bus checks cover session ownership, unsupported modes,
  repeated selection and missing compositor refusal. Frame-state checks cover
  allocation bounds, live source interest and a busy handoff slot. Process
  authentication checks use a real bus and a different kernel PID. At
  `d8fcb765`, all 594 tests and doctests across the seven permission, portal,
  capture-protocol, activity, and storage crates passed. The separate shell
  checks passed 24 capture/state tests, 12 direct-target tests, five direct-loop
  tests, and nine scanout tests. The media-server suite passed all 13 tests,
  including cancellation with an incomplete native response. Scoped clippy
  denied warnings, and workspace formatting was clean. Local logs are
  `/tmp/alo-studio-permission-verification.log` and
  `/tmp/alo-studio-capture-final-clippy.log`; the checks are repository tests,
  while these particular local invocations are one-off verification.
- The dedicated app CI job passed at `530994a9`. The OS CI run
  `41895386015` stopped while building workspace test executables because its
  runner ran out of disk space. The subsequent run `41912137856` reached all
  12,620 tests and identified two additional change-owned failures: the ADR
  lacked its status, and the language snapshot omitted 15 new portal keys.
  Both are corrected; the next CI result is pending. The runner's declared
  permissions-test failure was also present. No full workspace pass is claimed.
- The latest locally built Flatpak bundle is 1,079,112 bytes, SHA-256
  `22b93a168a26a623bdaa5cc5374151c60124e7aba733e41f896fceb719f867e2`.
  Its installed build excludes the verification feature. It has not been
  published to an application source or the live catalogue.

The private booted desktop has the installed GNOME 49 runtime and the bundle
above. Native picker cancellation recorded cancellation and left no publisher
or recording. Screen selection reaches the trusted chooser, but opening the
chosen source now opens, but preview exposed a format-negotiation failure.
Recording, playback, export, revocation and lock acceptance remain pending.

The startup trace located two deadlines behind that failure: first-use native
engine discovery took about 21 seconds in the software-emulated guest, beyond
the publisher's seven-second setup limit. A warm retry then reached the engine
but received a truncated frame after the compositor's one-second write limit.
Setup now allows 30 seconds inside a 40-second cancellable operation. Frame
delivery preserves its offset across short waits and checks source lifetime and
portal ownership while a consumer is blocked. It retains one in-flight frame
beside the newest slot. Both permanent real-socket tests passed at `1f068c4a`:
a reader starting after one second and revocation of a reader that never drains.
All 295 tests and doctests in the scoped media-server, portal, architecture and
language suites passed; formatting and scoped clippy denied warnings. These
checks live in the repository. Logs are `/tmp/alo-studio-delivery-tests.log` and
`/tmp/alo-studio-startup-tests.log`. The startup traces are one-off local evidence
under `/var/lib/alo-studio-verification` inside the private guest.

The booted media trace then showed a missing pixel aspect ratio negotiating as
`1/2147483647`, overflowing preview scaling. The native publisher now declares
square pixels. The first run of the new native-publisher CI check reached
preview but exposed ordinary EOS finalization waiting forever on this provider.
A one-off run using the existing input-stop and queue-drain path passed,
including decoded output (`/tmp/tmp.F9WsWrhhFb`). Finish now uses that same
path, stopping video and microphone before draining queued media.
The expanded check then caught a native crash in `video-input:src` during
shutdown. PipeWire input now copies engine-owned buffers before they enter
asynchronous queues, so stopping a stream cannot invalidate queued pixels. The
encoding queue also has a 128 MiB byte bound in addition to its two-second
limit. The native-publisher check remains pending until that full path passes.
That complete check passed locally and in app CI at `e747b594`, including
preview, pause/resume, finalization and decode. Local evidence is
`/tmp/tmp.Nt7iX8YTly`.
The newer guest media stack still negotiated an invalid pixel aspect ratio
despite the publisher declaration. A one-off guest pipeline reproduced the
invalid ratio and confirmed that a source-side square-pixel constraint delivers
frames at 640×400. Studio now applies that input constraint as well; guest logs
are under `/home/alo/StudioVerification/format-diagnostics`.
The app CI also builds that actual native publisher and passes
its non-16:9 output through Studio's preview, recording, pause/resume and decode
probe, checking that a 1280×800 source previews at 640×400. This is a permanent
CI check, with validation pending. Consent discovery now includes both Flatpak
installations using the existing desktop-entry helper. Preview failures use
accurate plain-language copy and reset the source-selection hint.

The native SaveFile and actual sandbox export/revocation proof are recorded in
`native-save-dialog-for-sandboxed-recordings.md`. An isolated private branch
image will exercise the installed app, native picker, recording, playback,
export, revocation and lock together. Nothing here claims production image,
kernel, or physical hardware acceptance.

The booted guest has not passed recording acceptance. Its first saved clip
decoded to one frame despite an advancing recording timer. A one-off trace of
the actual native publisher on the guest's PipeWire 1.4.11 stack then showed
successive compositor arrivals stamped at zero. The publisher now explicitly
uses GStreamer's system clock for arrival timestamps, independently of whether
its output graph has a consumer. Validation of that change is pending. The
permanent app CI probe exercises the real publisher and restricted descriptor,
requires continuous preview frames, and decodes the resulting recording; the
guest trace itself is a one-off investigation, not an automated acceptance.

The one-off guest transport comparison now passed with the clock change:
30 frames reached EOS over 4.45 seconds, against nine frames and a timeout
with the preceding publisher. Guest evidence is in
`/home/alo/StudioVerification/provider-monotonic-diagnostics`. Full Studio
acceptance is still pending. Studio also measures duration from encoded video
packets and refuses a completed recording with no timed video frames; an
advancing interface clock alone can no longer become saved clip metadata.
The empty-video and one-frame regressions are permanent app unit tests.
