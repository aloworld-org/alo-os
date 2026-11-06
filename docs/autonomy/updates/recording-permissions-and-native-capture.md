# Recording permissions and native capture

The owner authorised alo Studio's OS integration and a booted recording walk.
Work remains on `app/alo-studio`, with a draft review. Market publication,
signed distribution, production-image acceptance, and physical-hardware
acceptance are separate. The complete booted walk is **not yet accepted**.

Current result: native permissions, the installable application and 19 typed
agent actions are implemented. Application source `8555d5bd` passes dedicated
CI: formatting, Clippy, 28 unit tests, the actual GTK recording flow and the
pinned native capture publisher cases. At `00781cbd`, dedicated native CI,
authors validation and the hosted OS gate all pass. The hosted gate required a
retry after dependency installation timed out; its documented runner exceptions
were not changed. Installed window recording completes Start, Pause,
Resume, Finish, naming, library review and exact-file export. Decoded-media
quality is being checked below; it is not inferred from those successful
controls. The review remains a draft with no market publication. Earlier
measurements retain their tested source and are not acceptance results for
later packages.

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

Saved duration is read from the completed media by GStreamer discovery, off the
UI thread with a 15-second deadline. Missing, damaged, empty, or audio-only media
is refused; failure preserves the temporary file. Preview dimensions are bounded
without enlarging small sources.
Recording preserves incoming timing, up to 30 frames per second, and does not
invent duplicate frames for a slow source. A quality check offers an explicit
choice before recording on a slow device.

The latest implementation keeps the selected capture stream connected from
preview through recording. It attaches the encoder and optional audio branch to
the running pipeline with a blocked tee pad, retaining the negotiated source
rate when below 30. Finish stops inputs, then drains queued media into WebM.
Queued raw video is bounded by both frame count and 128 MiB. Microphone recording
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
one-off measurements. Both dedicated recording CI job `127135984023` and the
broader OS gate `127135983754` passed on `318eda56`.

The rebuilt package is 1,089,312 bytes, SHA-256
`87b95fdf9117c89330e0f1004a8142a70c17a036ff70736d2ac69f394741d083`,
with Flatpak commit
`453cba692a7b177d045bb1132db0ff3e0ea21ef69734ee3b17c30f5736c9b0bb`.
It is installed normally in the VM. The lower-resource package's real-portal
walk remains required before quality acceptance.

The installed GTK menus did not open reliably during the VM walk. Quality and
countdown now use visible toggle-button choices, including keyboard mnemonics
for quality. The actual installed app selected 360p/5 fps, no countdown, and the
microphone through those controls, then opened the native window chooser.
Its package SHA-256 was
`6d8026d6c7e4210d5a84676179a8030d056ee7d9c408a79dce297f26665cffa8`,
Flatpak commit
`8120dc911b932f431d97e8e65c3213bc9c0ff06ca0afce19206363f1f4391dee`.

That recording still stopped on microphone sample loss before Pause succeeded.
Clip `1793870158-802110353-2` decodes to 18 video frames at 494 x 360, 5 fps and
216 audio buffers. Video starts at zero; audio starts at 9.5945 seconds. The
container lasts 16.833 seconds; video metadata reports 10.093 seconds. Evidence
is `D:\\alo-studio-verification\\installed-lightweight-audio-stall`, SHA-256
`cd0d92f997842d238b95acd6f3a97f761559c30ff00a38b05b7e7ef580ca192c`.
This is failed quality acceptance, not a microphone pause/resume pass.

The resulting changes increase the microphone's bounded ring to two seconds
with 20 ms reads, and retain up to ten seconds/1 MiB of compressed audio while
video starts. Filling either recording queue requests a controlled finish.
Recording also excludes preview frames queued before the new branch is ready;
opening the microphone must not include earlier preview images in saved video.
Space shortcuts run before focused button actions. The fixture starts
WirePlumber only after PipeWire's socket exists, addressing an observed startup
race rather than retrying a failed recording.

At `ad6147b0`, formatting, clippy, 15 tests, the actual GTK recording flow, and
all eight pinned-base recording cases passed locally. GTK/framebuffer evidence
is `/tmp/tmp.MJWPZzPC1V`; ordinary PipeWire evidence is `/tmp/tmp.WFi4xIIfru`;
native publisher evidence is `/tmp/tmp.Auh5biNyDa`. The low-resource microphone
tracks start 33.5 ms apart in the ordinary fixture and 102.5 ms apart through
the restricted native descriptor. These repeat in dedicated recording CI.
The updated SDK build and another actual VM measurement remain in progress.

The next normally installed package, SHA-256
`8d5b4afb8165777d8e72d92b5d277e6a9365b9628c44fd1236888caa34586e80`,
at Flatpak commit
`cec0977b8c0985cc311ca44493ffb211fc9c690a9a45b6e3e0ef828027458dab`,
completed Start, Pause, Resume, and Finish with microphone enabled. Both tracks
start at zero and no microphone sample-loss warning was logged for that run.
Clip `1793871577-829297835-2` decodes to 27 video frames at 476 x 360, 5 fps and
863 audio buffers over 19.3 seconds. Video ends at 17.767 seconds; metadata
incorrectly reports 24.306 seconds. WebM SHA-256 is
`df587c7d6dae8aaa709441973c1c253c06d279beb513daf068852aa2580ac9d7`.
Evidence is `D:\\alo-studio-verification\\installed-verified-window-microphone`.

The thumbnail establishes that the chooser's default had changed to Studio
itself. This proves that selected window's lifecycle, not the demonstration
window's recording quality. The final walk explicitly selects the demonstration
window before Share. Sparse video and the duration discrepancy prevent a quality
acceptance claim for this run.

At `298342a5`, the application reads completed-file duration with GstDiscoverer
instead of estimating it from encoder packets. Discovery runs on a worker after
capture has closed. It must succeed before the library entry is committed.
Formatting, clippy and 16 tests pass, including damaged/missing media refusal.
Header discovery and decoded-stream duration queries differ in final packet
padding. Each must fit the observed active recording window; the GTK flow
requires its saved metadata to equal discovery exactly.

The normally installed duration-fix package has SHA-256
`dc40d06a9531ac1f5ab3eb8ec1080277f3d73f1943b2ce4735ef2d5b58f4accc`,
Flatpak commit
`ef9024952e3a78a2f28afd8151f85eb67a3785088cbd58994b37cae5e0c97514`.
It explicitly selected the demonstration window and completed the microphone
Start/Pause/Resume/Finish flow. Clip `1793873010-622283278-2` has 41 video frames
at 494 x 360, 5 fps and 762 audio buffers. Both tracks start at zero. The header
and metadata both report 17,320 ms; decoded playback reports 17,340 ms.
Its thumbnail contains only the intended demonstration window. WebM SHA-256 is
`b9b00839a925963e0b753775b75fb5441523e5b2f2cda10849318cad8311d2d1`;
evidence is `D:\\alo-studio-verification\\installed-demonstration-window-microphone`.
No microphone sample-loss warning was logged, but decoded timestamps still have
gaps (up to 4.764 seconds between video-frame coverage and 2.1 seconds between
audio-buffer coverage). Functional completion does not establish smooth motion
or uninterrupted audio in this software VM.

An intermittent host `gst_mini_object_unref(NULL)` critical was traced to pad
buffer handling, not discovery. Filtered buffers now return `Handled`, letting
the Rust binding consume each buffer once. `verify-native.sh` makes critical
warnings fatal and deliberately delays branch preparation to exercise stale
preview-frame filtering. At `199acff6`, formatting, clippy, 16 tests, this strict
GTK flow, and all eight pinned-base media cases pass. GTK evidence is
`/tmp/tmp.N8pHi8Hy0H`; ordinary PipeWire evidence is `/tmp/tmp.ZQInBFEzb6`;
native publisher evidence is `/tmp/tmp.wiYFAu8H63`. The dedicated CI job
`127183816629` also passed. The workspace screenshot now shows the actual current
application, including direct quality and countdown controls.

The resulting package is 1,094,736 bytes, SHA-256
`93a4f0e3e4fe58af1ce142ce3efd955b35b3f0030c2a356b2fc02e9e0ba0f6d5`,
Flatpak commit
`ab0047617e27ee70a3c03827559ca0e139c531f17a75269b83cfc0ddbdbcf8ee`.
It is installed normally in the VM. The final explicitly selected animated-window
measurement used a separate GTK fixture with a visible, running spinner.
Clip `1793875272-784685961-2` contains 13 distinct decoded video frames at
452 x 360, 5 fps and 281 audio buffers. Both tracks start at zero. Its header
and library metadata agree at 10,060 ms; decoded playback reports 10,080 ms.
Video ends at 3.439 seconds, while audio ends at 10.06 seconds. The guest reports
111,360 dropped microphone samples, and Studio automatically stops and saves.
Pause did not complete before that stop. This is a failed motion/audio quality
check, despite successful file finalization and accurate duration.

Evidence is `D:\\alo-studio-verification\\installed-animated-window-validation`.
The thumbnail confirms the intended animated demonstration window; the decoded
frames are distinct. WebM SHA-256 is
`1d2fd6cdf1a994dd6471cd1116fca32d9aa0608554f5f3e8d21d15593892507f`.
The post-stop graph has no input streams and no capture workers remain. The
normal SSH service was restored, the temporary verification key removed, and
its private host forwarding rule closed. The application and evidence remain
on the private VM disk; its staged OS overlay is temporary and will not survive
a reboot. Only this software VM was available, as confirmed by the owner.

Both dedicated recording CI (`127183816629`) and the broader OS gate
(`127184154863`) pass on `199acff6`. Remaining release work is animated-source
audio/video continuity in the software VM, production-image and physical-machine
acceptance, then signed application-market publication. Desktop locking remains
the separately authorised OS change.

### Recording startup investigation — 2026-11-05

This makes the existing recording promise true: selected video and microphone
must begin on the same media timeline. A one-time trace of the installed package
confirmed that live microphone state synchronization returned before its first
sample. Video entered encoding at 50.652 seconds of pipeline running time; the
first microphone sample was timestamped 59.040 seconds. Earlier video was then
rejected by the WebM writer. Waiting only for branch state synchronization did
not establish media readiness.

The recording branch now observes the actual first audio buffer and excludes
video preceding both that timestamp and branch readiness. The generated-audio
PipeWire case starts audio 750 ms later to exercise the timing boundary on every
application CI run. This fixture does not establish an installed VM quality pass.

The initial process sampler was too expensive for acceptance measurements in
the software VM; its results are diagnostic only. A later bounded sampler reads
only the selected processes every four seconds. Temporary timing instrumentation
is external to the package and will be removed before installed acceptance.

The later one-time native trace localized microphone backpressure to the raw
`audio-input` queue. It declared itself full with 3 buffers (5,760 bytes) but
1.789 seconds of timestamp span. Other readings contained 13–60 buffers at its
one-second limit. Microphone pushes then blocked for 0.78–1.0 seconds, followed
by 111,360 dropped samples. The compressed queue was not full in this run.
Changing the encoded buffer alone had left this earlier bottleneck in place.

Both audio queues now count retained buffers and bytes, with no timestamp-span
limit. Input retains at most 200 buffers; encoded audio retains at most 500;
each is additionally limited to 1 MiB. Either full queue requests a controlled
finish. A GStreamer regression holds the consumer, sends three small buffers
with a timestamp hole, and verifies they do not fill the queue; sending the
actual count limit must still request a stop. This runs with application tests.

The UI now distinguishes input preparation from recording. The timer and Pause
wait for the first fresh video admitted to the chosen audio timeline, while
Stop remains available. The hidden workspace skips copying preview images, but
the final retained preview remains available for the thumbnail. At `332962c5`, formatting,
clippy, 18 tests, the GTK flow including thumbnail validation, and all eight
PipeWire cases pass. This precedes validation of the audio-queue change.

At `c4204e56`, formatting, clippy, all 19 application tests, the GTK recording
flow, and all eight PipeWire cases pass. GTK evidence is `/tmp/tmp.H8cjh6tiyC`;
ordinary publisher evidence is `/tmp/tmp.x4AhiEEECu`; native publisher evidence
is `/tmp/tmp.Oy91h5TAfB`. A one-time mutation restoring the one-second timestamp
limit makes the new audio regression fail with only 2 buffers (3,840 bytes).
The actual count-bound refusal occurs at 200 buffers (384,000 bytes). Mutation
evidence is `/root/studio-audio-queue-regression.D7qteq`; the unmodified source
was rebuilt afterwards and its 19 tests pass again.

The optimized package from that source is 1,091,832 bytes, SHA-256
`99b51a89d748db14fe5e713930badd31ba0c8f9de098b3dc60b753f1b9a5c486`,
Flatpak commit
`a26da56b163b9705820737488c50f4f3baef0c4f0a57af9117068657f0dd439e`.
The installed animated-window recording from this package is
`1793893260-395891604-2`, preserved in
`D:\\alo-studio-verification\\installed-bounded-audio-acceptance`.
Audio begins at zero and video at 6 ms. All 664 decoded audio buffers are
continuous, and the library/header agree at 13,260 ms. However, the raw queue
actually reaches its 200-buffer limit and Studio stops before Pause. Video
contains 26 frames, ends at 8.876 seconds, and has an uncovered gap of 819 ms.
Continuous audio and correct duration do not make this a quality pass.

Dedicated CI's first attempt at this source passed formatting, clippy and all
19 tests, then timed out while starting Xvfb, before launching the application.
Its job `127341663295` was rerun; the rerun has reached the native publisher
build after passing the GTK flow. This is not an application test waiver.

### Encoder and recording-controls follow-up

The explicit 360p/5 fps choice now uses Opus complexity zero, retaining the
48 kHz mono format and 96 kbit/s target bitrate. Other profiles retain
complexity ten. Two ten-second audio measurements in the installed runtime
took 5.848/5.889 seconds at complexity ten and 1.997/2.608 seconds at zero.
VP9 row multithreading is enabled for recording and preflight. Reducing VP9's
thread count did not improve the measured workload, so it remains four.
These measurements justify narrower processing changes; they do not establish
native recording quality.

The recording bar now hides the workspace when mapped, even when the selected
demonstration window retains focus. New Recording cannot re-present the busy
workspace during preflight or countdown. The preview keeps draining native
samples and skips copying images while hidden; a valve that stopped forwarding
preview buffers was removed. The saved thumbnail retains the last preview.
The GTK check requires the workspace to stay hidden while controls are open,
and the media fixture checks encoding continues during hidden preview.

Application actions now expose actual enabled states, and Pause exposes the
actual lifecycle state. Absolute pause-state requests pass through the same
pause/resume implementation as the buttons. Start becomes available after the
first visible preview image. These changes permit bounded application commands
during recording without repeatedly traversing the accessibility tree.

The native development-binary run `1793897421-903215198-2` completed Start,
Pause, Resume, and Finish without a recording-queue refusal. It contains 68
video frames (36 distinct images), 1,478 audio buffers, and matching library
and header duration of 37,480 ms. Video begins at 30 ms; audio begins at zero.
However, uncovered gaps reach 7.515 seconds for video and 3.5 seconds for audio;
video ends at 29.255 seconds while audio continues to 37.48 seconds.
This is a functional completion and a failed quality check. It uses a normal
feature-set development binary, not the final optimized installed executable.
Evidence is `D:\\alo-studio-verification\\installed-command-controlled-recording`;
the WebM SHA-256 is
`c91da84e313ed568bb7ffc82bbad95665f5b2519b5350ff9d87f884a97dced5d`.

The verification-only delayed-audio fixture now measures actual first delivery
and shifts its timeline by that observed delay. A one-time mutation removing
the startup gate fails with audio at 752.5 ms and video at zero. The earlier
`audiotestsrc` timestamp-offset setting did not produce the intended delay;
the corrected measurement is documented in
`docs/misreadings/a-delay-setting-did-not-delay-the-fixture.md`.

At application source `b4196118`, formatting, clippy, all 19 tests, the strict
GTK flow, and all eight pinned-base PipeWire cases pass. GTK evidence is
`/tmp/tmp.3CVbN7UfuS`; ordinary publisher evidence is `/tmp/tmp.m89CCdQT4A`;
native publisher evidence is `/tmp/tmp.ebRms6JmIf`. The runner now closes its
own process group, including its private accessibility/D-Bus helpers. Older
helpers belonging only to these Studio fixtures were removed after exhausting
the host's per-user watcher limit. No wider session cleanup was performed.

The optimized bundle from application source `b4196118` has SHA-256
`fc30dd088b14daeac496c2554183ea756b9fc03f70afd9df9e85751ab7b0b106`
and Flatpak commit
`15a0491b0fbdd7bc2071874a7b1b628d30843315282c2c7a2ff454c9cd588c63`.
The bundle is 1,101,864 bytes and is installed normally in the private VM.
No timing preload or development executable is used for this installed run.
Animated-window clip `1793898614-661320853-2` completes Start, absolute Pause,
absolute Resume, and Finish. It decodes to 47 video frames (36 distinct images)
at 452 x 360, up to 5 fps, and 1,473 audio buffers. Library and header duration
both equal 33,720 ms; decoded playback reports 33.74 seconds including packet
padding. Audio begins at zero; video begins at 588 ms, outside the fixture's
250 ms startup allowance. Video's largest uncovered gap is 11.775 seconds and
audio's is 2.16 seconds. Video ends at 25.523 seconds while audio continues to
33.72 seconds. This is a failed startup/continuity quality measurement, despite
functional completion and a playable saved file.

Evidence is `D:\\alo-studio-verification\\installed-optimized-native-recording`;
the WebM SHA-256 is
`2ac341a8fd013bd95749ab736924feb038d0b0cd60d3fe09fdff87110dbfb0c1`.
The native picker log confirms the animated demonstration window was explicitly
selected. During Pause the microphone input node is idle. After Finish the
global graph has no audio input streams and no native capture worker remains.
Pause is disabled with its state reset. One immediate post-save command query
timed out at 15 seconds; the later query returned the expected completed state.
Neither query result changes the failed media-quality result.

### Microphone shutdown and preview follow-up

Native teardown tracing measured 3.552 seconds to disconnect video, followed by
296 ms to stop the microphone. Source `34b2e8a4` therefore stops microphone input
first and suppresses capture timestamps immediately. A verification-only 750 ms
video-disconnect delay checks this order. Restoring the old order fails the
existing duration check; independent decoding confirms 603 ms of extra audio
(video ends at 4.202 seconds, audio at 4.805). The restored implementation's
GTK fixture ends video at 4.021 seconds and audio at 4.012 seconds. This is a
measured shutdown regression fix, not a general continuity fix.

Source `0ababe75` consumes hidden-preview buffers before color conversion. Its
stronger fixture requires encoded duration to advance while the preview is
hidden. The initially installed build did not receive video during native
recording startup; its 5,218-byte partial file remains. Source `db80ae7b` retains
preview flow until the encoder produces fresh video, including after Resume.
Source `83f821ea` retains live PipeWire scheduling and disables cached keepalive
frames. The app still timestamps received frames against its running clock.

The normal-feature development binary at `83f821ea` completes the actual
animated-window Start/Pause/Resume/Finish flow at 360p, up to 5 fps, with
microphone enabled. Clip `1793904786-33079266-2` contains 58 video frames and
1,180 audio buffers. Video begins at 36 ms and ends at 22.302 seconds; audio
runs from zero to 27.24 seconds. The largest uncovered video and audio gaps are
5.146 and 3.64 seconds. Library/header duration agrees at 27,240 ms; decoded
playback includes 20 ms packet padding. This is another failed quality check.
Evidence is `D:\\alo-studio-verification\\installed-live-stream-diagnostic`;
WebM SHA-256 is
`dc2eb0f2de499c36423fb81e71c09d6c80d77b6c78cb4e8f6ea6c47d191276b4`.

Other private comparisons do not establish an accepted remedy:

- A newer PipeWire consumer plugin alone, tested against the earlier `b4196118`
  application, still leaves 11.33 seconds of uncovered video and 3.14 seconds
  of uncovered audio. Its private override is excluded from installed builds.
- A development-only 2 fps variant completes the controls, reducing uncovered
  video to 663 ms, but still loses 2.36 seconds of audio and leaves audio
  extending almost six seconds beyond video. The production quality choices
  are unchanged; this variant is not adopted.
- A private virtual microphone comparison configures an in-guest null sink and
  attempts generated tone, without host microphone access or speaker output.
  No completed file establishes nonzero tone samples in that trial. Rejected
  queued video precedes the first microphone timestamp; the recording branch
  then sees no fresh video. Changing audio timestamping, explicit consumed-pad
  flow status, non-live source properties and dynamic latency recalculation
  does not produce a completed recording in those trials. These changes are
  not adopted. The last partial file and command trace are preserved under
  `D:\\alo-studio-verification\\installed-latency-prototype`.

Dedicated application CI and the OS gate passed at `83f821ea`: native job
`127457444495`, authors job `127457444937`, and gate job `127458323636`.
The isolated checks cover the stronger hidden-preview assertion and delayed
microphone shutdown. They do not substitute for the failed booted quality
measurements or establish production-image or physical-hardware acceptance.

### Rejected optimized candidate

The optimized, ordinary-feature Flatpak built from application source
`83f821ea` was installed normally in the same private software VM. It is
1,104,528 bytes, with SHA-256
`8ee71eb2acf0da308b359e774a68d3e9e2517c99d2c78e52327d4d7d68989c76`
and Flatpak commit
`91f9ca358eb69aec81052e4b5b500e4b9473b95e2548f1acd1eb9677b2c75c31`.
No development executable, timing preload, private consumer plugin, 2 fps
profile, or latency experiment is part of this package. It failed the installed
startup check: the actual source picker accepted the demonstration window, but
Studio never enabled Pause or reached Recording. A controlled Finish refused
the audio-only result with "No video frames were recorded" and preserved
`1793911262-737550604-2.webm.part` (31,269 bytes). The candidate bundle, partial
file, source choice, application commands and journal are preserved under
`D:\\alo-studio-verification\\installed-live-capture-candidate`.

Source `44fff5c7` removes the hidden-preview conversion probe and restores the
capture source settings of the last installed build that completed the controls
(`34b2e8a4`). Hidden preview samples continue to be pulled, without copying images
to the UI. The microphone-first shutdown fix and stronger hidden-preview
duration assertion remain. The earlier optimized startup failure outweighs
isolated-fixture passes; the speculative preview/scheduling changes are not
retained as production fixes.

### Conservative release package

The replacement optimized bundle is built from application source `44fff5c7`.
It is 1,102,904 bytes, SHA-256
`ee5f5caccd85d3e471a29fff5e1e1b61b3062d366b7f5165bd6bbe9b82a08a5a`,
Flatpak commit
`5013659da65e28915bc503e0069c2c49bb2ab8bc3ccf915af4d37a72e194440a`.
The local bundle is `C:\\dev\\alo-studio-artifacts\\alo-studio-0.1.0.flatpak`.
Its dedicated CI passes formatting, clippy, all 19 tests, the real GTK flow and
all eight pinned PipeWire cases (native job `127521272854`).

This installed package completes selected animated-window Start, absolute Pause,
absolute Resume and Finish with microphone enabled. Clip
`1793912088-260602040-2` decodes to 52 video frames (35 distinct images) and
1,734 audio buffers. Video starts at 165 ms, within the fixture's 250 ms startup
allowance, and ends at 35.364 seconds. Audio runs from zero to 39.06 seconds.
The largest uncovered gaps are 19.194 seconds for video and 2.26 seconds for
audio. Library and header both report 39,060 ms; decoded playback includes 20 ms
of packet padding. The emulated microphone contains 34.66 seconds of decoded
PCM samples with peak zero, so this run does not validate spoken audio quality.
The recording is playable and functionally completed, but **fails native
continuity acceptance**.

Evidence is `D:\\alo-studio-verification\\installed-conservative-native-recording`;
WebM SHA-256 is
`536441f92c0da5c7ff3f5f0275d466c57fffcccedd50b2dcb460e19d3a224f98`.
The source choice and each requested/observed application state are retained.
After Finish no native capture worker remains. An immediate action-state query
again times out at 15 seconds; it is not treated as a successful state check.

The later query succeeds: Pause is disabled with false state. The graph has one
idle microphone input during Pause and no audio inputs after Finish. New
Recording resets the workspace and microphone; the ordinary installed app is
left running without an active capture. Original audio input is restored, the
private null sink is unloaded, and diagnostic executables, preloads and consumer
overrides are removed. The separate demonstration fixture is stopped. A fresh
console confirmation at `2026-11-05T20:59:23Z` verifies normal SSH is active and
the temporary authorized key is removed. The artifact server is stopped and
neither temporary host port listens. Completed recordings, partial media, the
VM disk and comparison evidence remain. The staged OS overlay is still
temporary and will not survive a reboot.

Native continuity, production-image acceptance and signed market distribution
remain open. The only available test machine is this software VM; its failed
measurements are not waived. No physical-machine acceptance is claimed.
Desktop locking remains a separate OS change.

Earlier scoped OS checks passed: 594 permission/portal/protocol/activity/storage
checks, 295 transport/portal/architecture/language checks, 413 client-lifecycle
checks and 12 native-parent wire checks. The exact-file save boundary is
documented in `native-save-dialog-for-sandboxed-recordings.md`.

## Follow-up investigation, 6 November 2026

The current task reopens private VM verification. The prior cleanup above is
historical: temporary SSH forwarding, the artifact server, diagnostic binaries
and metadata preloads are currently in use again and must be removed after the
installed-app acceptance run. The ordinary SSH service and original capture
worker have been preserved for restoration. No image or market package has been
published.

Paired timestamps now locate an indefinite buffer acquisition in the capture
publisher while compositor frames continue arriving. In the preserved
`capture-pool-wait` evidence, acquisition 581 begins without returning and input
count advances from 667 to 762. The appsrc queue stays bounded at two frames.
Thread inspection finds the publisher's streaming thread waiting on a condition.
The sink signals that condition without the pool mutex, allowing a notification
between dequeue and wait to be lost. A private timed-recheck trial keeps frames
moving; ADR 0146 authorizes the narrowly checksummed build-copy patch. The
imported upstream source remains unchanged. Plugin version
`1.4.90+ee2c6eb4.2770143f.wait1` adds a 100 ms predicate recheck.

The same GNOME runtime in the available software VM encoded a two-second
640 × 360, 5 fps sample in 4.391 seconds with VP9 and 1.814 seconds with VP8.
Two seconds of isolated microphone encoding took 0.818 seconds at 48 kHz and
0.560 seconds with the selected 16 kHz voice configuration. The lightweight
profile now uses VP8 and the cheaper voice path; preflight and recording share
one configuration. Instrumented development builds still overrun the audio
queue. These comparisons motivate the changes and do not establish continuous
or synchronized media. An optimized, normally installed package and capture
worker are being prepared for the full flow.

Studio now exposes explicit status, recording, settings, library and export
operations through its versioned application interface. The daemon routes the
`studio` adapter through the existing approval and execution record. Read-only
capability discovery refreshes before each model round; Studio controls appear
only under a live application grant and delivery rechecks that grant. Changes
retain single-use approval. Native source and file choosers retain consent.
Targeted Clippy for the agent, daemon, protocol, adapters, shell and permissions
passes. Core turn, protocol and permission logic passed 158, 148 and 25 unit
checks respectively. The real private-bus adapter tests pass, including bounded
status, a fixed integer countdown sent only after approval, missing ownership and
revocation. The two Settings checks pass: choosing an app alone writes nothing,
cancel writes nothing, and choosing a duration persists only that app's grant,
with visible revocation. Live installation inventory, protocol integration and
the first-party agent's driving tests also pass. These are permanent tests in
the source tree, distinct from the manual VM measurements below.

The isolated buffer-wait regression compiles the actual patched upstream pool
implementation. The patched build completes dequeue and preserves nonblocking,
pause and flushing outcomes; the baseline misses the withheld notification and
hits the watchdog. This check runs in the pinned-engine verification container.

### Installed recording with a known audio signal

The normal Flatpak built from application source `9f96fa43` includes the semantic
control API, lightweight VP8/Opus profile and review-player cleanup. Its SHA-256
is `f2125b8249a294f83f9fc705a6142783cfbf63eabd8c61378b8339a103983667`,
Flatpak commit `f3fcdd95593cb8c44422ceee338fb4ca81c40418894adf16f632b3771627554f`.
The fixed capture worker and private `wait1` plugin run without timing preloads.

The emulated HDA input is silent. A virtual monitor initially was silent too;
forcing its test generator onto the system clock supplied an actual signal,
confirmed independently outside and inside the sandbox. The later trial uses
PipeWire's native pipe source and a generated PCM fixture, avoiding a second
GStreamer pipeline competing with the recorder. This is virtual-device evidence,
not physical microphone acceptance. Local compilation was stopped during capture.

Clip `1793963752-989885763-2` records the selected animated window at 452 × 360.
It contains 33 decoded video frames and 8.60 seconds of actual PCM over a
27.585-second header duration. Startup video/audio separation is 156 ms. The
largest uncovered gaps are 1.492 seconds for video and 3.200 seconds for audio;
the decoded audio peak is 2971, proving this recording includes the supplied
signal. The input audio queue reaches its retained-data limit, so Studio seals
the recording before the requested Pause. This trial fails continuity and does
not establish clean pause/resume. The library keeps the completed recording.

Evidence is `D:\\alo-studio-verification\\installed-file-fed-microphone`; WebM
SHA-256 is `5358f1da6ffce6fbc0fe5dee9bc3d56938e5f6fa07bde78979bba68544dc31f6`.
This is a one-off installed measurement and does not run in CI.

Source `df9adc0b` is a further performance trial: its buffer-only preview gate
is installed after recording attachment and waits for an actual encoded frame
before skipping hidden colour conversion. It retains the native capture source
settings from the working startup path and forwards negotiation and EOS. The
earlier preview optimization's installed startup failure remains relevant; this
candidate requires its own installed acceptance and is not yet a quality fix.
Review details now say WebM for either video encoder.

No demo-quality acceptance is claimed. The only available machine remains the
current software VM. Desktop locking remains a separate OS change.

### Native scheduling and the current installed measurement

The native PulseAudio service reports RTKit calls through
`org.freedesktop.portal.Realtime` failing with `UnknownInterface`. PipeWire
1.4.11 chooses that route when the portal name exists, without checking whether
the Realtime interface exists. alo's portal owns the name but does not implement
that interface. The native PipeWire and PulseAudio services now explicitly use
the installed system RTKit broker; sandboxed clients retain their existing
configuration and gain no scheduling authority. Both data-loop threads were
observed running at round-robin priority 20 after the configuration was applied.
This follows the rented engine's [RT module configuration](https://docs.pipewire.org/page_module_rt.html)
and its [1.4.11 implementation](https://github.com/PipeWire/pipewire/blob/1.4.11/src/modules/module-rt.c).
The image installs RTKit explicitly and two permanent image checks hold the
native configuration and package dependency.

The ordinary Flatpak from application source `df9adc0b` has SHA-256
`40b7d38e443dcd91d54cc34aee315e8b0adbf20e3299aa28391fac9420afc3d6`
and commit `88886a201f38461d28b44d4d4761eda2ed284bdef2af38219635041412e20545`.
Its first hidden-preview trial completed Pause, Resume and Finish, but clip
`1793964891-9152358-2` still contains only 42 video frames and 10.64 seconds of
PCM over a 30.840-second header duration. Largest uncovered video/audio gaps
are 2.310/2.900 seconds. The earlier file-fed source also logged underruns and
broken pipes, so it is unsuitable as the final audio fixture.

A native, clocked GStreamer tone and a null-sink monitor replace that source.
Independent native and sandboxed probes each deliver three seconds of PCM over
approximately three seconds between first and last delivery. Their total
elapsed times include startup and must not be interpreted as missing audio.
The native scheduling trial still stops early on the audio-queue limit: clip
`1793966970-725682634-2` has 31 video frames and 8.66 seconds of PCM over
21.140 seconds, with uncovered gaps of 1.793/3.620 seconds.

The same disk was then rebooted with two virtual CPUs, `-cpu max` and a 512 MiB
TCG translation cache. Six optimized native binaries from OS source `260f51e2`
were staged, including the current desktop, agent service/client and capture
services. This changes both the CPU profile and native build, so it is not an
isolated CPU comparison. The native dialog service initially timed out during
the default-application installation. The installation's verified cgroup was
temporarily frozen; after startup completed, the ordinary dialog service ran
successfully again. No permanent dialog-service override was installed.

With compilation stopped, the normal installed app and ordinary native
services recorded clip `1793972152-678118873-2`. It contains seven changing
video frames and 4.14 seconds of actual PCM over 6.180 seconds. Uncovered
video/audio gaps reach 1.313/2.040 seconds. The supplied audio signal is present
(decoded peak 1937). The audio-input queue reaches its 200-buffer retained-data
limit and Studio saves the partial recording before the requested Pause. This
quiet measurement also fails continuity; it does not establish synchronized
audio or clean installed pause/resume. Evidence is
`D:\\alo-studio-verification\\installed-quiet-current-system-recording`, with WebM
SHA-256 `e9a5c8043b34819c75e61657fbb6d6f720d39759f5d186661ecc84c77eefdd7b`.
These installed measurements were performed once and do not run in CI.

### Application permission on the running OS

The agent's real OS identity initially receives no Studio actions and its
status request is refused. In the visible Settings application picker, Studio
was selected and then a one-hour duration was chosen. Discovery then offers all
19 Studio actions, and `studio.status` returns the installed app's bounded
status through the running daemon. A proposed `studio.countdown_three` stays
pending until the person-side protocol connection approves that exact proposal.
The approval returns `application-controlled`, and a subsequent status read
confirms the countdown changed from zero to three seconds. This exercises the
real daemon, distinct person/agent identities and installed application; it does
not claim a model conversation or a visible approval-card walk.

Taking back the newly created application permission through Settings removes
all Studio verbs from discovery immediately and a new status request is
refused. The earlier exact-file export permission is retained. This is a
one-off walk on the running OS. Permanent adapter, Settings, grant, source and
vocabulary checks also pass, including the refusal directing application
permission to Settings and a session with no personal settings folder.

In the current source, the Settings page is called **Access for alo**, covering folders,
applications and explicitly shared files. Application-control refusals point
to that Settings choice rather than telling someone to pick a folder. Existing
adapter declarations and grant checks remain intact.

### Encoder coordination comparison

A metadata-only diagnostic of the normally installed app records timestamps
and wall/thread CPU time at capture, conversion, VP8 output and both audio
queues. It finds long waits around the encoding branch and audio backpressure,
with no repeat of the earlier indefinite publisher acquisition. Timing preloads
are diagnostic only and are removed for acceptance measurements.

In the same available VM, four alternating tests of a 30-frame, 452 × 360,
5 fps sample complete in 6.037/6.668 seconds with two VP8 threads and
3.580/3.840 seconds with one. The Lightweight profile now uses one encoder
thread in both preflight and recording. This is a one-off comparison, not a CI
performance threshold.

The normal package from application source `12ba1d99` has SHA-256
`03c4d3fe33576170eac345c31799bb89bcdc126d048f8d4200c35371c9bcf7fc`
and commit `63ffcf2a07f3dc412ff18dcb749660519503516d7cd826d5ecfe53dc2829772b`.
Its installed trial `1793975857-828822970-2` still stops at the audio-input
limit before Pause. It has 26 video frames (25 consecutive changes) and 8.54
seconds of PCM over a 14.525-second header duration. Largest uncovered gaps are
1.767 seconds for video and 2.480 seconds for audio; decoded audio peak is 1828.
The exported publisher uses asynchronous processing, and the recorded worker
thread classes are retained with the evidence. This trial fails continuity.
Evidence is `D:\\alo-studio-verification\\installed-single-thread-recording`;
WebM SHA-256 is `f96c60a8c9cf617b42f48968e855e7dec5c498be3f38c21ff9df2a43cf5a2811`.

### Four-CPU timing investigation

The same disk now runs with four virtual CPUs, `-cpu max` and the same TCG
translation cache. The host has four available CPUs and another existing VM;
that VM was not changed. These are software-emulation measurements, not an
isolated-host benchmark. The two-CPU command and firmware state are preserved.
The first-boot application installation was frozen again during recording.

| Installed application | Recording | Video frames | Video end | PCM duration | Largest uncovered video gap |
|---|---|---:|---:|---:|---:|
| `12ba1d99` | `1793977158-344973477-2` | 120 | 24.378 s | 23.280 s | 0.995 s at Resume |
| `6ad69d1f` | `1793978952-239210547-2` | 120 | 24.058 s | 23.620 s | 0.503 s during recording |
| `37111f19` | `1793980394-145942011-2` | 91 | 22.318 s | 22.240 s | 0.525 s |

Each completes the installed control flow and native exact-file export. The
first two have no uncovered audio timestamp holes. The third contains 179 ms
of overlapping audio at Resume; its PCM duration must not be confused with its
22.061-second final audio timestamp. None establishes final synchronization.
Exports `Product demonstration 2.webm`, `Product demonstration 3.webm` and
`Product demonstration 4.webm` match their private originals exactly. Their
SHA-256 values are respectively
`06babb921f9a08c069df05678b21dd0b905e7d6e735f6f87d1593227fa351936`,
`76c9cb9780104e39b86c455551cb0299c44e7c6645fb292973ea6005ba556555` and
`6223d12c8756c1f8c6436e2fafb3f32d19167a1cfe3eb4aed098f84abbe048ef`.

GStreamer samples pipeline running time again after pausing its children.
Studio had already stopped admitting video, so a slow native pause added a
gap to the saved timeline. A real delayed-child regression reproduces an added
300 ms before the fix. Saving the request boundary excludes that stop time;
the installed `6ad69d1f` trial removes the large Resume gap. Its remaining
audio/video end difference is 438 ms. These video ends include the final
frame's nominal 200 ms display duration at 5 fps. A difference in track ends
alone does not establish an offset between recorded sound and visible events.

An independent microphone probe supplies 48 seconds of uninterrupted PCM over
48.755 seconds between its first and last delivery. Its 51.615-second total
also includes 2.835 seconds of startup. Microphone recording now selects the
microphone's clock before the first sample and starts a fresh recording
timeline, while retaining the permission-scoped capture stream. Old preview
buffers from the previous clock epoch are rejected. Silent recordings retain
the system clock. Permanent checks cover clock selection, failed-clock preview
recovery and stale-buffer rejection.

The first microphone-clock run exposes the Resume overlap above. The pause
order now stops that microphone before saving its clock boundary. A real
delayed GStreamer child verifies this order. The subsequent timed-source run,
`1793981493-675597584-2`, has 21.540 seconds of PCM with neither timestamp
holes nor overlap, but video ends at 24.346 seconds with a 1.130-second
uncovered gap. Only six audio pulse starts and one visual transition are
measurable. It fails alignment verification and is retained as a failed trial,
not an acceptance pass. Changing the reference source to a system clock alone
does not establish alignment: its native clock advances while its covered
window retains an old painted clock label. Its export
`Product demonstration 5.webm` matches SHA-256
`a4e61c12a210af4658d7dbae49347154f5d8d8c0d1d4a88994a4eae51fa051f9`.

The current normal package, built from `8555d5bd`, has SHA-256
`6b576f484fd51998ba423f6239a3a99c4e4b2c8625dad73b7782d951183d7c06`
and Flatpak commit
`f669255934ee94d1ecb3387d594a9595a95c6d8514f2598f3e06154c68c96ecc`.
It contains no verification feature, development command or timing preload.
The earlier native OS binaries staged for this boot are from `260f51e2`; this is a
temporary `/usr` overlay, not production-image acceptance. Later source wording
for **Access for alo** is not claimed as installed in that older desktop binary.

The repeated timed-source run `1793981932-584295404-2` stops at the same
200-buffer audio-input limit before Finish can be exercised. It has 90 video
frames over 23.755 seconds, 23.540 seconds of PCM with no timestamp holes or
overlap, and a 1.597-second uncovered video gap. Only two audio starts and no
visual transitions are measurable. SHA-256 is
`6ffb6b07de97523dc01b872cec1905d2b309780f24779a3f04b94a8a8a176ead`.

Screen-only clip `1793982170-485460354-2` completes Start, Pause, Resume,
Finish, review and native export with microphone disabled throughout. It
contains 54 video frames, all consecutive frames changing, over 21.458 seconds
at 576 × 360. Its largest uncovered video gap is 0.619 seconds. This is still
below the selected 5 fps ceiling and does not establish smooth video. The
export `Product demonstration 6.webm` matches its original with SHA-256
`b5c46d5aa0f19b03def548c398b6a4a343043de77f911b019c5daf9c92b88ffe`.

### Covered application repainting

Inspecting decoded pixels confirms that the timed source retains its old
painted clock value while native logs show its clock advancing. The separate
window-capture renderer does not complete Wayland frame callbacks; normal
desktop presentation completes only its own submitted surfaces. A covered
captured client can therefore stop repainting even though repeated old pixels
continue through capture and encoding.

The source now completes callbacks for the selected mapped surface tree and
its own popup trees after a successful private capture. Failure and unrelated
windows receive none. This is an additive default `FrameTarget` method,
preserving existing implementations. A real Wayland client regression covers
failure, mapped parent/child completion, an unmapped child, duplicate
completion and an unrelated application. The regression and strict library/test
Clippy checks pass at `00781cbd`. Optimized desktop, agent service and agent
client binaries from that source are now installed in the running VM; their
checksums are recorded in `capture-callback-system-identity.txt`. The updated
agent correctly refuses an ungranted Studio read and points to application
access in Settings. The Settings wording is therefore present in this new
desktop build.

The installed application remains the normal `8555d5bd` package. On this
software VM, its verification launch now also uses `GDK_DISABLE=gl,vulkan`
with the existing Cairo renderer, avoiding unsuccessful GPU initialization.
GTK documents these as debugging controls; they are not shipped as an end-user
configuration. A one-off Review/status check completes in 16 seconds, compared
with earlier repeated 30-second timeouts. This is not a general UI latency
guarantee. See [GTK's runtime controls](https://docs.gtk.org/gtk4/running.html).

The private VM was paused during the optimized build and resumed before
testing. Its wall clock was restored afterward. The first-boot installation
was resumed during build work and frozen again for measurement. An independent
virtual-audio probe during host load supplies silent PCM and reports timestamp
gaps; it cannot be used as a valid synchronization reference. The VM is paused
again while unrelated queued builds complete, reducing contention. The final
installed recording repeat is queued on the shared build lock and first
requires a healthy reference signal. Its result is not yet claimed.

The first reserved window opens at 19:21 UTC with no host compiler active.
The reference generator is still using the system clock. Its independent
native monitor probe supplies 10 seconds of PCM with no timestamp gaps and
three detected pulses. Restarting the native audio graph and the generator
again supplies 10 seconds with no timestamp gaps, but only two pulses. Both
source probes reach EOS but fall below the reference's minimum of four pulses, so
this attempt does not start a Studio recording. The complete output is retained
in `installed-callback-clock-verification/control-flow.txt`. These are source
health measurements outside Studio, not installed recording acceptance.

The external reference generator is now rebuilt to use its audio sink's clock
rather than force the system clock. This is a verification-fixture change,
not a Studio package change, and its effect is still unverified. GStreamer's
[audio sink clock contract](https://gstreamer.freedesktop.org/documentation/audio/gstaudiobasesink.html)
tracks the sink's data rate; resolving this fixture's missing pulses remains a
hypothesis. The next
reserved run will also complete the installed control and export flow if the
pulse reference remains insufficient, while retaining that failure separately
from synchronization acceptance.

### Status queries blocking frame delivery

The next reserved run has no host compiler active and independently receives
10 seconds of uninterrupted PCM with seven pulses. The reference still selects
`GstSystemClock`; removing the forced clock did not select its audio sink's
clock, so the improved pulse count cannot be attributed to that hypothesis.
The installed recording nevertheless stops before Pause on the 200-buffer
audio-input limit. Clip `1793995052-176373741-2` contains 35 video frames over
13.801 seconds and 9.120 seconds of PCM. Largest uncovered gaps are 1.494 seconds
for video and 2.100 seconds for audio. No flash transitions are measurable.
SHA-256 is `47a9261f3e0d2e42c48e1104f78fcde9b98dfee7deabb1891707bcbc566b59e6`.
Evidence is retained in `installed-callback-audio-clock-verification`.

A subsequent diagnostic client trace shows Wayland frame replies arriving many
seconds after requests while its local clock advances. This trace runs under
host load and is not a cadence benchmark. A live snapshot of the normal
compositor shows its main thread in an indefinite `poll` on subprocess output
pipes. A pipe-owner lookup identifies `pw-dump`, and the source confirms that
the status area's volume query runs synchronously on the compositor thread at
its once-per-second refresh. The media-activity indicator already has a worker;
that does not cover this separate speaker-volume query.

The new change moves repeated hardware status reads to one outstanding worker
with a bounded handoff. The local clock remains responsive to time and region
changes. Hardware readings that fail, time out or arrive stale become unknown;
late results cannot overwrite a new clock format. The startup query remains
before the desktop takes the display. Regressions cover a blocked reader,
bounded outstanding work, a late result, failure and worker disappearance.
The hosted OS gate at `c65625cd` passes within its existing runner limitations:
12,659 tests pass, one fails exactly as already declared for that runner, and
61 are skipped. Studio's separate native recording job and the authors check
also pass. No exception was changed. The runtime status-reading code is from
`95dd2dce`; the following commit only bounds native dependency installation.
A hosted optimized build at `8cd35fc7` now passes all 55 desktop tests, including
the three blocked/late/unavailable-reader regressions. Strict optimized Clippy
checks pass for the desktop's full target set and the shell library. The
[desktop verification job](https://github.com/aloworld-org/alo-os/actions/runs/42298855519/job/128171075380)
provides the binary, source revision and SHA-256 for the installed repeat.
The downloaded archive and executable checksums are verified; the executable is
`60c25a08cc9bffdce3253b99ecce1f1912991d3395a40fa43eedeb6e0d264224`.
The local recording reservation is still queued behind validation of separate
cursor branches; it will use that tested binary without rebuilding it.
No additional media acceptance is claimed.
