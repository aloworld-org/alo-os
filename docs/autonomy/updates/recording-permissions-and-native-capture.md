# Recording permissions and native capture

The owner authorised OS integration and a booted recording walk for alo Studio.
This work is on `app/alo-studio`; market publication remains separate. Booted
acceptance is in progress and is not established by the host checks below.

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
  of a physical microphone.
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
  authentication checks use a real bus and a different kernel PID. Final
  validation of these latest checks is still running.

The native SaveFile and actual sandbox export/revocation proof are recorded in
`native-save-dialog-for-sandboxed-recordings.md`. An isolated private branch
image will exercise the installed app, native picker, recording, playback,
export, revocation and lock together. Nothing here claims production image,
kernel, or physical hardware acceptance.
