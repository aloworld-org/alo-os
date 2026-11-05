# Recording permissions and native capture

The owner authorised alo Studio's OS integration and a booted recording walk.
Work remains on `app/alo-studio`, with a draft review. Market publication,
signed distribution, production-image acceptance, and physical-hardware
acceptance are separate. The complete booted walk is **not yet accepted**.

## Implemented boundaries

alo's public portal identifies the Flatpak caller through its bus connection
and kernel process, requests a fresh native source choice, records the answer,
and creates a grant in the existing central permissions store. Session closure,
caller disappearance, revoked permission, source loss, and seat loss withdraw
sharing. Persistent and restored capture choices are refused.

The compositor authenticates the live portal owner. Window capture renders only
the selected root and its own popups. Source identities follow mapping and output
lifetimes. Capture is forbidden for lock, sign-in, and recovery scenes. Pixel
handoff never blocks the rendering thread: workers retain the latest frame and
one in-flight frame, with cancellable partial writes and bounded admission.

The selected-node descriptor exposes the necessary PipeWire factories and the
chosen node, including link permission; unrelated current and future objects
remain hidden. ADR 0146 documents the narrow rented-binding additions. Native
window parenting uses XDG foreign v1 and verifies the actual focused parent.

SaveFile uses the native GTK chooser and Documents portal. The exact selected
file receives a central grant before it is returned to the sandbox. Removing
that grant withdraws its document permissions. The application has no network,
camera, or broad home-directory permission. Verification fixtures are excluded
from ordinary installed builds.

## Recording behavior

Saved duration comes from encoded packets. Empty video cannot become a completed
library entry. Preview dimensions are bounded without enlarging small sources.
Recording preserves incoming timing, up to 30 frames per second, and does not
invent duplicate frames for a slow source. A quality check offers an explicit
choice before recording on a slow device.

The latest implementation keeps the selected capture stream connected from
preview through recording. It attaches the encoder and optional audio branch to
the running pipeline with a blocked tee pad, retaining the negotiated source
rate when below 30. Finish stops inputs, then drains queued media into WebM.
Encoded frames are bounded by both frame count and 128 MiB. Microphone recording
starts off for every new recording. Native dialogs share Studio's application ID.

## Verification and current limits

Permanent host checks passed at `8d18ba6d`: formatting, clippy, ten meaningful
lifecycle/storage/duration tests, actual GTK/Xvfb recording, generated Opus audio,
pause/resume, source ending while paused, finalization, and decoding. Both the
ordinary PipeWire fixture and actual native publisher/restricted-descriptor
fixture pass with and without generated audio. Current native evidence is
`/tmp/tmp.JVDEnhWYo2`; framebuffer and GUI evidence is `/tmp/tmp.2IK7mBXZsS`.
Generated audio does not establish physical microphone quality.

The private software-emulated guest established these one-off observations:

- A production-feature development binary at `e3352001`, mounted read-only in the
  installed Studio Flatpak sandbox, recorded a selected window, paused/resumed,
  saved it, and played it to completion in the review screen. The clip decodes
  to 30 video frames over 102.613 seconds at 600 x 436. Its metadata reports
  102,612 ms. The thumbnail contains only the selected demonstration window.
- Studio's Save a copy action opened the real native chooser and saved
  `Product demonstration.webm`. The original and 31,891-byte export are identical,
  SHA-256 `099ab146ed57eb0af447189d286d151bbdc1f8148129a69987275c8dcdac8f36`.
  Settings displayed that exact file grant. Reopening for writing from the
  sandbox succeeded before Take back and was denied afterwards. Both files
  remained intact. This is an actual booted UI export/revocation observation.
- The installed package at `ce4b97a1` opened screen capture with the microphone
  explicitly enabled. Actual graph activity identified Studio's microphone;
  pause suspended both streams. Taking back the capture grant in Settings
  removed both media streams. That run encoded no video, was correctly refused
  as a completed recording, and retained its temporary file.
- A separate real-portal screen test reproduced missing recording buffers without
  audio, while saving a correct full-screen preview. The trace showed the new
  recording connection waiting for buffers after preview disconnected. The
  continuous-stream implementation above replaces that handoff and isolates
  encoder negotiation from the selected native format. At `8d18ba6d`, the real
  portal screen probe passed: 1152 x 720, 15 preview frames, finalized WebM,
  and VP9 decoded to EOS. Independent host decoding counted 14 encoded frames
  over 11.039 seconds, with opaque alpha. Guest evidence is
  `verified-screen-stable-format.log`
  and its video/preview artifacts under `/home/alo/StudioVerification`.
  A production-feature development run at the same revision still delivered no
  encoded video with the microphone switch enabled. Taking back its permission
  while paused removed grant 20 and all capture streams; empty-video refusal
  worked. That is a failed recording observation, not an accepted combined flow.
  **The installed package's combined screen/microphone flow is being verified.**

The SDK-built `8d18ba6d` package was installed in the guest with SHA-256
`2d9b251308d51e912fea9857bcff85382332c37f3b65d29305c1ea790aee5ef4`
and Flatpak commit
`34b48c9dec6c0cd4e39f482d42f131f21f74dc6d7218fd540b9816ffce259ee0`.

The guest uses QEMU software emulation, two virtual CPUs, and a virtual audio
input. An isolated 600 x 438 VP9 sample encoded 60 frames in 12.4 seconds. These
observations do not establish 30 fps performance or microphone quality on
physical hardware.

The native desktop in this branch does not expose a live lock transition. Lock
policy, exclusive-scene capture refusal, and nested lock components are tested,
but a booted desktop-lock recording test cannot yet be claimed. The dedicated
application CI passed at `8d18ba6d`. The broader OS gate reports failure, while
its job-step response remains incomplete and its log endpoint returns 404;
the cause has not been established. There is no full-workspace or
production-image pass here.

Earlier scoped OS checks passed: 594 permission/portal/protocol/activity/storage
checks, 295 transport/portal/architecture/language checks, 413 client-lifecycle
checks and 12 native-parent wire checks. The exact-file save boundary and earlier
host-only proof are documented in `native-save-dialog-for-sandboxed-recordings.md`.
