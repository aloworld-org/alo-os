# Recording permissions and native capture

The owner authorised OS integration and a booted recording walk for alo Studio.
This work is on `app/alo-studio`; market publication remains separate. Booted
acceptance is in progress and is not established by the host checks below.

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
chosen source currently times out before preview. Recording, playback, export,
revocation and lock acceptance remain pending while that failure is diagnosed.

The startup trace located two deadlines behind that failure: first-use native
engine discovery took about 21 seconds in the software-emulated guest, beyond
the publisher's seven-second setup limit. A warm retry then reached the engine
but received a truncated frame after the compositor's one-second write limit.
Setup now allows 30 seconds inside a 40-second cancellable operation. Frame
delivery preserves its offset across short waits and checks source lifetime and
portal ownership while a consumer is blocked. It retains one in-flight frame
beside the newest slot. Two permanent real-socket tests cover a reader starting
after one second and revocation of a reader that never drains. Validation and
the next booted retry are pending. The startup traces are one-off local evidence
under `/var/lib/alo-studio-verification` inside the private guest.

The native SaveFile and actual sandbox export/revocation proof are recorded in
`native-save-dialog-for-sandboxed-recordings.md`. An isolated private branch
image will exercise the installed app, native picker, recording, playback,
export, revocation and lock together. Nothing here claims production image,
kernel, or physical hardware acceptance.
