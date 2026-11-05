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

At `21c303eb`, formatting, clippy, 11 lifecycle/storage/duration checks, actual
GTK/Xvfb recording, generated Opus audio, pause/resume, source ending while
paused, finalization, and decoding passed. The private PipeWire fixture now also
opens the production microphone implementation through an isolated PulseAudio
server and virtual monitor. It waits three seconds with the preview running
before attaching audio, and requires actual decoded buffers from both tracks
with startup timestamps within 250 ms. It never selects physical audio hardware.

The ordinary fixture's microphone tracks started 10.5 ms apart. The actual
native publisher and restricted-descriptor fixture passed all three cases:
117 preview frames without audio, 116 with generated audio, and 121 through
PulseAudio, with the latter tracks starting 32.5 ms apart. Native evidence is
`/tmp/tmp.usbt00Qhg4`; ordinary PipeWire evidence is `/tmp/tmp.9Lf5DK5kRc`;
GTK/framebuffer evidence is `/tmp/tmp.xdf9BtSe1k`. The microphone-state regression
also verifies that a delayed switch activation during preflight/countdown is
retained, while recording, pause, and finalization refuse changes.

The private software-emulated guest established these one-off observations:

- A production-feature development binary at `e3352001`, mounted read-only in
  the installed Flatpak sandbox, recorded a selected window, paused/resumed,
  saved it, and played it to completion in GTK review. It decodes to 30 frames
  over 102.613 seconds at 600 x 436; metadata reports 102,612 ms. Its thumbnail
  contains only the selected demonstration window.
- Studio's Save a copy action opened the real native chooser and exported
  `Product demonstration.webm`. The original and 31,891-byte copy are identical:
  SHA-256 `099ab146ed57eb0af447189d286d151bbdc1f8148129a69987275c8dcdac8f36`.
  Settings displayed that exact file grant. Reopening for writing succeeded
  before Take back and was denied afterwards. Both files remained intact.
- At `8d18ba6d`, the real ScreenCast portal probe produced a finalized
  1152 x 720 WebM. Independent decoding counted 14 encoded frames over
  11.039 seconds. The installed SDK package also saved a 1280 x 800 screen
  recording, independently decoded as 14 frames over 17.325 seconds.
  Guest evidence is `verified-screen-stable-format.*` and
  `completed-screen-without-audio.*` under `/home/alo/StudioVerification`.
- A production-feature development binary at `303d0b4f` used the actual portal
  and real PulseAudio microphone path in the installed sandbox. Both streams
  opened, paused, resumed, and paused again. Taking back capture grant 22 in
  Settings while paused removed the grant and both streams. Studio finalized
  clip `1793845599-533344291-2`, with microphone metadata true. The installed
  SDK package subsequently opened it from the recording library.
- That 244,447-byte clip contains VP9 and mono 48 kHz Opus and both tracks
  decode to EOS: 17 video buffers and 17,871 audio buffers over 628.88 seconds.
  SHA-256 `69b6fe999ca9fd227fe902457eb28938d0bc7257f46d92b28626d9cff6b1f704`.
  However, its first decoded video timestamp is 65.536 seconds after audio,
  and the guest logged audio sample loss. **It is not accepted as a
  demonstration-quality recording.** Evidence is `completed-screen.*`,
  `fixed-retry-paused-media.txt`, `fixed-resumed-media.txt`,
  `fixed-before-revoke-media.txt`, and `revoked-media-status.txt`.

The PulseAudio check passes on the host; this does not explain or waive the
software guest's poor cadence and delayed initial video. The guest uses QEMU
software emulation, two virtual CPUs, and a virtual audio input. An isolated
600 x 438 VP9 sample encoded 60 frames in 12.4 seconds. Neither microphone
quality nor 30 fps performance on physical hardware is established.

An isolated diagnostic checkout of `21c303eb`, with the native live-source
override removed and microphone enabled in the real-portal probe, did not resolve
cadence: it delivered 11 preview frames and failed the minimum of 15. It was not
promoted into the application. Its diagnostic decoder handled both tracks with
asynchronous preroll disabled. Evidence is `verified-live-window.*` and the
change note in `/root/alo-studio-live-evaluation.oLz3dt/.qa-changes`.
Host checks use GStreamer 1.24.2/PipeWire 1.0.5; the guest's installed runtime
uses GStreamer 1.26.11/PipeWire 1.4.9 against the OS's PipeWire 1.4.11 server.

The SDK-built `df655460` package was installed normally with SHA-256
`1b6d4b5d23f50867579a6503c2fd7e50ba34007dbdc029b0db7a740cf6a3e2fb`
and Flatpak commit
`2b9db4c3171837e2b037d451cb8afb8d43b92c3de8d3d2937f4a18c45b371b80`.
The final accessibility/test build at `21c303eb` is installed normally in the
guest with SHA-256
`42b9d3124621f426049543ce1d9c033f1200e5b1738c172f21ebc560332357fe`
and Flatpak commit
`6afbd7601858f15deb4dde3dedc0273e7958b9c7e265b5c11bd4ecc1a742df76`.

A further small-window microphone run at `21c303eb` also saved and decoded both
tracks, but did not establish demonstration quality: 600 x 436, 11 decoded video
buffers, 579 audio buffers, and a 19.98-second container. The first audio/video
timestamps coincide, but video is sparse. Its evidence is
`completed-window-microphone.*`; clip ID `1793847916-139378302-2`.

The native desktop does not expose a live lock transition. Lock policy,
exclusive-scene capture refusal, and nested lock components are tested; a booted
desktop-lock recording test cannot be claimed. Desktop locking is treated as a
separate OS change pending the owner's scope clarification.

The dedicated application CI passed at `df655460`, `e5810d12`, and `21c303eb`,
including the new isolated microphone fixture. Broader OS acceptance remains open;
there is no full-workspace, production-image, or physical-hardware pass here.

Earlier scoped OS checks passed: 594 permission/portal/protocol/activity/storage
checks, 295 transport/portal/architecture/language checks, 413 client-lifecycle
checks and 12 native-parent wire checks. The exact-file save boundary is
documented in `native-save-dialog-for-sandboxed-recordings.md`.
