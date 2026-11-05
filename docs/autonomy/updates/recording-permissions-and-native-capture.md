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
desktop-lock recording test cannot be claimed. The owner confirmed that desktop
locking belongs in a separate OS change. The owner also confirmed that only the
current software VM is available, so the remaining recording-quality investigation
continues there; physical-hardware performance remains unverified.

Microphone sample loss is reported by GStreamer as a `CORE/CLOCK` warning from
the microphone element. Studio now treats that specific warning like encoder
overload: stop capture, drain the media already queued, and explain that the
device could not keep up. Other warnings and warnings during finalization do
not interrupt saving. This prevents silent continuation after audio loss; it
does not establish recording quality in the software VM.

At `9f111958`, formatting, clippy, and all 14 unit/refusal checks passed,
including microphone sample loss, unrelated warnings, and finalization after a
late warning. GTK/framebuffer checks passed in `/tmp/tmp.6OtO4hE7nf`.
The private PipeWire/Pulse fixture passed in `/tmp/tmp.YtLI0wegW7` with 61
video-only, 61 generated-audio, and 62 microphone preview frames; microphone
tracks started 14.5 ms apart. The native publisher/restricted-descriptor fixture
passed in `/tmp/tmp.PwJvC94XvT` with 115, 116, and 114 frames respectively;
microphone tracks started 7.5 ms apart. Dedicated native CI job `127086557094`
also passed. The updated SDK-built package has SHA-256
`9941e9e732709be5c06811044596e63de7497eb02611dae564d98313d8c7db45`.
These are application/scoped integration results, not a booted quality pass.

A private VM trial with QEMU's `max` CPU model did not establish an improvement.
During that guest's startup, a 60-frame 600 x 438 sample reached its 90-second
timeout; 320 x 240 completed in 47.720 seconds. Startup load differs from the
earlier measurements, so these results do not isolate CPU instructions as the
cause. The trial was reverted to the original CPU configuration. Console
evidence is `D:\\alo-studio-verification\\cpu-evaluation-console.log`; the five
staged OS binaries were backed up and checksum-verified before restoration.

The updated package above is installed normally after restoration, at Flatpak
commit `a37cad8275c79da1d998bab7adfe7c153fa4c5938f3faf9c3c32e12c2762bafd`.
The private test disk had two serial login services sharing one terminal; the
duplicate was masked. The first-boot application installer was also stopped
during capture diagnostics. Neither adjustment changes the application.

An isolated guest transport reproduction exposed a publisher lifecycle failure:
the distribution's PipeWire 1.4.11 GStreamer sink exits with `all buffers have
been removed`. A warmed run delivered seven preview frames, then none after
Resume. The strengthened probe now requires at least two fresh frames after
Resume; the older aggregate-frame check could accept only pre-pause frames.
The same stricter check also exposed the host's older distribution fixture.

The upstream owned-buffer plugin at `ee2c6eb41ebab02c91faeb567cba86ece44dd996`
survived the guest reproduction: 44 preview frames, including 27 after Resume,
and VP9/WebM decoded to EOS. Its trial binary SHA-256 is
`88959dd93b3046e2d16ad5609444cfe7e4e4236e0ce0b5240cbaa758b7aeffe7`.
Evidence is `D:\\alo-studio-verification\\guest-transport-owned-buffer-prototype`.
This used generated frames and a diagnostic PipeWire connection; it is not a
source-permission or demonstration-quality acceptance result.

The integration now packages that engine privately for the publisher, with
upstream's buffer-release race fix `2770143f506a92e242f8a5b8b870929b515b48c0`.
The sanitized publisher launcher supplies its fixed settings explicitly, and the worker refuses an
unexpected filename or version. The distribution's daemon and other programs'
plugins remain separate. Container checks target the image's pinned base and
the stricter resume assertion.

At `1594b42b`, clippy and 16 scoped native tests passed (14 media-server tests,
two private-engine refusal tests), as did the image packaging contract. The
dedicated application CI and broader OS gate both passed at `289589a1`.
The pinned-base container passed all six recording cases: ordinary and native
publisher paths, each with no audio, generated sound, and isolated PulseAudio.
Each continued to deliver fresh frames after Resume.

The updated portal, worker, unit, and private plugin are installed in the VM's
temporary OS overlay. The portal, worker, and plugin SHA-256 values are
`e5b7cb78f8895ad4b69c906b1e2c5c4e080795160571ca3ba7a287a8174d88a7`,
`ff330c21fa00d2f277b607b04e3559130cfe53aeb85dd445f812f733eb35dd69`, and
`783acc20cde1eb97f28a029705bf9ff916912dc920e765cd2c1533d25e681ed2`
respectively; the unit is recorded in the installation evidence.
The publisher cache is owned by `alo:alo` with mode 0700. Installation evidence
is `D:\\alo-studio-verification\\private-capture-engine-install.txt`.

The installed default-quality app then used the real native chooser to select
the demonstration window with microphone enabled. It stopped after detecting
microphone sample loss and saved clip `1793866252-340967369-2`. Independent
decoding reached EOS: 19 video frames at 600 x 436 and 345 audio buffers, over a
9.14-second container. Both tracks start at zero; video ends at 3.885 seconds.
The metadata's 13.439-second duration disagrees with the container. This is
failed quality acceptance, including a duration discrepancy to investigate;
it is not a pause/resume acceptance result. The thumbnail contains the selected
window. Evidence is
`D:\\alo-studio-verification\\private-engine-installed-recording`; WebM SHA-256
`025142f2e23b2e28e3da8725f401cc29b801f0fa074d7f4ca4cde50daa3b27e2`.

At `318eda56`, Studio adds a visible **360p · up to 5 fps** option for slower
devices. The 1080p default remains unchanged. Preflight measures the selected
rate; the recording branch drops excess frames before conversion and scaling.
The media probe checks decoded dimensions and frame-rate ceiling. Formatting,
clippy, all 15 application tests, and GTK/framebuffer checks passed. Framebuffer
evidence is `/tmp/tmp.F6A4ggj5nG`; the new profile decodes at 480 x 360, 5 fps.
The ordinary pinned-engine fixture passed all four cases in
`/tmp/tmp.6qi4dmvOI6`, including the new low-resource PulseAudio case. An initial
native invocation timed out during startup; a fresh run with the current
release worker passed all four cases, with 53–60 fresh frames after Resume.
Its low-resource recording decoded at 576 x 360, 5 fps with coincident first
audio/video timestamps. Preserved native evidence is
`/root/alo-studio-engine-container-evidence/tmp.tUxpVGa2KA`.
These checks run again in `.github/workflows/studio.yml`; VM observations are
one-off measurements. The lower-resource package's real-portal walk remains
required before quality acceptance.

The dedicated application CI passed at `df655460`, `e5810d12`, and `21c303eb`,
including the new isolated microphone fixture. Broader OS acceptance remains open;
there is no full-workspace, production-image, or physical-hardware pass here.

Earlier scoped OS checks passed: 594 permission/portal/protocol/activity/storage
checks, 295 transport/portal/architecture/language checks, 413 client-lifecycle
checks and 12 native-parent wire checks. The exact-file save boundary is
documented in `native-save-dialog-for-sandboxed-recordings.md`.
