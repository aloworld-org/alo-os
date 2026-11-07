# Recording permissions and native capture

The owner authorised alo Studio's OS integration and a booted recording walk.
Work remains on `app/alo-studio`, with a draft review. Market publication,
signed distribution, production-image acceptance, and physical-hardware
acceptance are separate. The complete booted walk is **not yet accepted**.

Current result: native permissions, the normally installed application and 19
typed agent actions are implemented. The normal GNOME 49 SDK Flatpak source is
`06213b2e`, built without verification features; its bundle SHA-256 is
`ac0ba3a3aa33af1895fbefd9ed21d0c7b2571128da51a2008b018e5b0697e0ba`.
All five hosted checks pass for that source. Thirty-three application tests and
the real native integration verify input ownership, lifecycle and pause cutoff.
The installed desktop is `5943a4d6`, with direct reusable software readback.

The previous installed desktop was `0a763631`, with all five hosted checks passing.
Normal application inputs are unchanged from the installed `06213b2e` package,
verified by Git object identity for source, assets, dependencies and Flatpak
recipe. Both installed flows complete source selection, stable five-second pause,
correct microphone states, resume, finish, review, naming, library and exact native
export. Screen measures 4.153 fps with a 615 ms uncovered gap; window measures
1.985 fps with a 1.806-second gap. Window PCM is continuous for 37.700 seconds;
track endings differ by 111 ms. Two marker matches do not establish sync. Both
saved videos have increasing timestamps and expected tracks, but fail unchanged
media acceptance. All media, partials and compiler resumption proof are retained.

An instrumented stage trace finds the publisher already receives about two
frames per second before encoding, with an empty publisher queue and short pool
waits. A recording-time activity sample then identifies kernel work as the
largest compositor cost; minor-fault sampling attributes about 95% of sampled
faults to full-frame export and readback. The next candidate exports directly
into initialized owned bytes and reuses them only after all readers release
ownership. It preserves Smithay's static renderer target and uses Pixman's safe
copy API for the temporary destination. Optimized regressions, strict desktop
checks, native integration and packaging pass. Installed default-configuration
recordings still fail: screen measures 4.573 fps with a 691 ms uncovered gap;
window measures 1.727 fps with a 2.513-second gap and a 10.103-second track-end
difference. Five-second pause clocks and microphone pause states pass. Screen
export matches exactly; the window's native save control times out, while its
completed library recording is preserved. One marker match does not establish
audio/video synchronization. The competing compiler processes are resumed.
The previously rejected renderer-binding trial stays
reverted. No engine source, allocator defaults, media timing or acceptance limit
is changed by the new candidate.

A temporary compositor-only allocator trial produces a passing screen recording
(4.819 fps, 373 ms maximum uncovered gap), but its microphone/window recording
still fails (2.861 fps, 1.033-second gap and unhealthy reference after idle).
These are trial-configuration results, not default-system acceptance. The exact
settings, media and restoration are preserved. Allocator defaults are restored;
no trial configuration is proposed for the image.

The agent permission walk separately verifies live discovery, approved execution,
immediate revocation and refusal. Revoked Studio access removes its actions from
discovery and refuses the read. Other applications must expose their own adapters.
The review remains a draft, with no market publication. Desktop locking remains
a separate OS change. No acceptance threshold, media timestamp or duration is
changed by the current candidate.

The readback candidate's executable SHA-256 is
`be66100c0ea778c5ad71a66cd26602e230fad5a3c619c28f0b3df468fc4d3b73`.
Its normal application inputs remain unchanged from installed package `06213b2e`.
Default-configuration media and controls are retained under
`installed-reused-readback-screen`, `installed-reused-readback-window`, and
`installed-reused-readback-validation` on the verification drive. Diagnostic
minor-fault and call-chain samples are separate from those acceptance recordings.
Live ownership metadata confirms two output pixel allocations are actually reused
and unobserved frames have no reader preventing reuse. Remaining sampled faults
occur while writing those retained allocations and clearing the render target;
they do not by themselves prove new allocation. Process-creation investigation
remains in progress. A failed debugger connection temporarily stops the desktop;
the owned debugger is removed and `TracerPid: 0` with a running desktop is verified.
No debugger or sampling hook is present during the acceptance recordings.

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
remain hidden. ADR 0149 documents the narrow rented-binding additions. Native
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
moving; ADR 0149 authorizes the narrowly checksummed build-copy patch. The
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

### Installed status-reading repeat and remaining rendering cost

The reservation subsequently runs with no host compiler active. Integrated
source `9772d083` passes native Studio CI, all 55 optimized desktop tests,
strict desktop/shell checks, authors validation and the hosted OS gate. That
gate runs 12,671 tests: 12,670 pass, one matches its existing declared runner
limitation, and 61 are skipped. No exception changes. The normal Flatpak is
rebuilt from this source, without its verification feature; its SHA-256 is
`70b3ff37918ca858294eac9119fe694aa82f1ad755634edb47e862a26bfc284d`.
The native desktop's SHA-256 is
`650bda68803a2d024d73d95006cf0ae891b8f67315b953d3453e66a4c4abd16f`.
The title field now accepts the same 200 characters as storage and automation.
Both following installed flows verify a 200-character title and restore their
professional recording names before completing export.

Window/microphone clip `1794007539-309622476-2` completes recording,
Pause, Resume, Finish, review, naming, library navigation and native exact-file
export. Its 193 video buffers end at 39.558 seconds, with a largest uncovered
gap of 1.579 seconds. Its 37.300 seconds of decoded 16 kHz mono PCM have no
detected holes or overlap. The visual reference supplies only one measurable
flash against 21 audio onsets; synchronization is not established. The export
`Product demonstration 7.webm` matches its private original, SHA-256
`268a397c63e65c0701e6448c3979238e6ce1a4324da5a95423e95bf77b19688a`.
Evidence is in `installed-status-readings-recording`.

Screen clip `1794008193-58456063-2` completes the same installed flow with
microphone disabled throughout. It has 54 video buffers ending at 21.520
seconds, with a largest uncovered gap of 1.413 seconds. The export
`Product demonstration 8.webm` matches its original, SHA-256
`ecb7849d3cdcc153937d4105cbb45a84b7d791fc4788f82adf92d196cee11d24`.
Evidence is in `installed-status-readings-screen-recording`. Video gaps with
microphone off show that the remaining problem is not confined to audio clock
selection. Neither complete control flow establishes continuous video.

An isolated packed-pixel conversion experiment does not show a reliable
microbenchmark improvement. Its installed desktop, `70830d88`, passes the
desktop and native CI jobs but its window recording stops before Pause on the
audio-input bound. Clip `1794009901-961346376-2` has 65 video buffers ending at
26.319 seconds, a 1.051-second uncovered video gap, and 22.380 seconds of PCM
with no detected holes. It has only 19 distinct decoded video hashes and no
measurable visual flash. Evidence is in `installed-packed-readback-window`;
SHA-256 is
`086b9188b33edeb495a401e9f137c06291ea518a1b932c4b3755eb258b14a773`.
This experiment is removed rather than credited as a performance fix.

The optimized `70830d88` executable retains function names without adding
instrumentation. A separate 16-sample live compositor profile, taken outside
recording acceptance, places nine samples in `dock_surface::surface` through
`dock_raster::picture`. The source rebuilds the rounded Dock panel's raster on
each frame despite unchanged inputs. The previous stripped snapshot's channel
conversion instruction was only one sample and did not establish a dominant
hotspot. The raw named profile is retained as `compositor-samples.txt`.

The new panel cache compares all visual inputs, including borrowed shapes,
appearance, display clipping, scale, position and focus. It retains at most
eight images and 16 MiB of owned vector storage per rendering thread. Oversized
panels still draw without being retained. Regressions compare reused and fresh
pixels, verify caller mutations cannot poison the cache, exercise visual
changes, and check eviction and oversized images. The change is integrated
with main `251dd212`; both unstable and staging Wayland client features are
retained in the dependency conflict resolution. At `ad8feea0`, all 55 desktop
tests, five readback tests, 15 panel/cache tests and strict desktop/shell
checks pass. The hosted workspace run has 12,700 passes and five failures:
one existing declared runner limitation and four desktop source-inventory
failures caused by the newly added cache file. The inventory is subsequently
extended so the same authority and appearance audits cover the cache. No
exception or audit rule is removed.

The normally installed application's saved-video playback is also exercised
with the earlier complete window/microphone clip. A real pointer click starts
the Review player's native Play overlay. Subsequent screenshots show its
recorded source clock advancing from 97.57 to 130.87 seconds. A fresh native
audio-output listing identifies `org.aloworld.Studio`, active 16 kHz mono PCM,
unmuted and uncorked. This establishes Review playback and virtual output
routing; it does not establish physical audible output or synchronized
recording. Screenshots and the fresh audio listing are retained in
`installed-review-playback`.

### Scanout memory and the synchronization reference

The installed panel-cache desktop has SHA-256
`0ee9b4ceb17fe13517eda4055254358a29dfe9fe56b9f39c05a0011f362f1cb7`.
Window clip `1794012104-874814924-2` completes recording, Pause, Resume and
Finish; two paused snapshots five seconds apart have identical elapsed time
and an inactive microphone. Its native export `Product demonstration 9.webm`
matches the original with SHA-256
`14025f58e9fb1c010e3cff091a4f82f443198b6e2e9d9d8b1c9354e2db779e08`.
It contains 138 video buffers ending at 28.063 seconds and 27.660 seconds of
PCM with no detected holes. The largest uncovered video gap is 1.093 seconds.
Synchronization fails, and the independent reference itself has a 2.360-second
timestamp discontinuity. This run cannot establish synchronized recording.
The repeated 200-character stress check times out after export; that check
already passed on both earlier complete flows using this same Studio binary.
Native-only repeats retain ordinary naming/export checks without repeating
that unchanged application stress case.

Screen-only clip `1794012667-656078606-2` has 51 video buffers ending at 23.073
seconds and a largest uncovered gap of 0.734 seconds. Discovery confirms one
video track and no audio track. Its five-second paused-state check also passes.
The native Save dialog is present but initially behind the source window and
larger than the VM display. An explicit native Save action subsequently
completes export. `Product demonstration 10.webm` matches the private original,
SHA-256
`99b1cfe85e0de43ad98ffb619f13405692423c1e8720f24448eff40af4cb2d98`.
Document grant `26cbec81` gives only Studio read/write access to that file.
The initial accessibility script times out, so the later native choice,
matching copy and fresh document-grant proof are preserved separately rather
than represented as a successful uninterrupted harness run.

A subsequent named profile has no Dock rasterization samples. Seven of its
16 samples are in scanout allocation/initialization or readback, including
full-frame clearing. The raw result is
`installed-panel-cache-window/compositor-samples-004843.txt`. This is a coarse
profile taken outside acceptance, not a measured rendering-speed ratio.

The next change constructs initialized four-byte pixels directly and flattens
their storage, avoiding a preliminary zero pass. An isolated VM comparison
preserves identical pixels in four alternating rounds: twelve 1280 × 800
frames take 0.444–0.591 seconds with zero-first conversion and 0.217–0.297
seconds with direct construction. This does not establish whole-pipeline
performance. The scanout transaction also initializes supplied pixels before
framebuffer registration in one mapping, clears only destination padding and
tail bytes during upload, and retains failure rollback and old-scene ownership.
Idle iterations yield briefly instead of spinning between frames.

At `745b67e3`, the 55 optimized desktop tests, five readback tests, 15
panel/cache tests, 64 allocation/scanout/lifetime tests and four desktop source
audits pass. Strict desktop and shell checks pass, as do native application CI
and authors validation. The source incorporates main `da1192f0`; the image
merge retains both native capture packaging and the generated cursor theme.
The verified native executable has SHA-256
`669dddf8c05cfd7b721c3f4470c76af78a11e00b4dae8bb3aa33abda7756d0c8`.
Its [optimized verification job](https://github.com/aloworld-org/alo-os/actions/runs/42323439985/job/128253126428)
provides the binary and source/checksum manifest. Installed media acceptance
for this candidate is still pending.

The full `745b67e3` gate runs 12,715 tests: 12,711 pass, four fail and 61 are
skipped. Besides the existing declared runner limitation, two failures expose
the recording ADR's number colliding with main's new watching ADR; the third
expects the cursor package's Cargo build to start its own image instruction.
The recording decision and its citations move to ADR 0149, while watching
retains ADR 0146. The image keeps both the private plugin build and cursor
generation, with Cargo in a separate `RUN` instruction. No executable capture
logic, audit rule or runner exception changes. These integration corrections
are awaiting the repeated hosted gate.

Reference validation is strengthened to check timestamp span, uncovered gaps,
overlap, wall-time delivery and a bounded pulse count. The original fixture
gates its tick waveform by buffer timestamp although tick generation has its
own phase; late buffers can therefore separate its sound and visual markers.
The prepared replacement generates PCM from each sample's timestamp, clears
the silence generator's GAP flag after replacing its payload, and computes
visual phase when actually drawing. The independent reference must pass
before synchronization is accepted. No saved recording is repaired or padded
to manufacture a pass, and production queue limits remain unchanged.

### Installed scanout-memory repeat and glyph rendering

The verified `745b67e3` desktop is installed with no host compiler active.
The background application installer is stopped cleanly for measurement and
scheduled to restart during cleanup. Window/microphone clip
`1794017448-201333090-2` completes recording, a five-second pause with exactly
stable elapsed time and microphone off, Resume, and Finish. Its 152 decoded
video buffers average 3.782 fps at the selected 5 fps and have a 0.807-second
uncovered gap. Its 41.400 seconds of PCM have a largest gap of 20 milliseconds;
the track endings differ by 144 milliseconds. The independent audio reference
fails delivery timing even after one audio-graph restart, and only two visual
flashes can be matched. Synchronization is not accepted. SHA-256 is
`e68706a468f65190410133ed93a36aa30366812d3ce9c43a8066ae9706debc2b`.

Screen clip `1794017786-118732295-2` completes recording, the same stable
pause, Resume, and Finish. It contains exactly one video track and no audio
track. Its 40 buffers average 1.387 fps and have a 3.964-second uncovered gap;
SHA-256 is
`7a3146239dfa5a700c3824e6bfa60b3fc2191a83163a35d8c9eac8275d23b4f0`.
The earlier native window export remains pending after its accessibility
timeout, so the screen flow's export is correctly refused as busy. Neither
export is credited as complete. A screenshot shows the native Save dialog
extending beyond the VM display. Exact saved-media and control evidence are
in `installed-scanout-memory-window` and `installed-scanout-memory-screen`.
Later native choices complete both exports. `Product demonstration 11.webm`
matches the window recording and document grant `41225a8a` gives only Studio
read/write access. `Product demonstration 12.webm` matches the screen recording
with grant `b80d236b`. These proofs remain separate from the initial failed
accessibility flow. The VM's remembered chooser size is 1203 × 902 pixels,
larger than its display; a temporary 760 × 500 setting is used for subsequent
checks, with its original value saved for cleanup.

Separate 16-sample profiles outside recording acceptance show text shaping,
glyph rasterization and native painting among the remaining costs. A deeper
stack identifies `painted_text::inked` through both Dock labels and status-row
sentences. That function creates a new `SwashCache` for every draw, discarding
the same glyph images between frames. The next change retains glyph images
per renderer thread, bounded by 4,096 entries, 16 MiB of image allocations and
32 face references. It compares actual font owners before reuse because font
IDs alone are local to a database. Regressions check exact fresh/reused output,
colliding IDs from different databases, and release of excessive pixel storage
and missing-glyph entries. CI and installed media verification of this change
are pending; no performance acceptance is inferred from the sampled stacks.

### Installed glyph-cache result and software-native readback

Source `59f8166b` passes dedicated native and optimized desktop CI, strict
checks, authors validation and the hosted OS gate. The gate runs 12,719 tests:
12,718 pass, one matches the existing declared runner limitation, and 61 are
skipped. The four glyph-cache regressions and three existing text-raster tests
pass alongside the existing desktop, readback, panel, resource/lifetime and
source-audit checks. No audit or runner exception changes. The installed
desktop's SHA-256 is
`1d7e4f3f46a15ba1c7294a822be2fbd3feadc3aff4c4ce68e5a487d0f1748c6b`.

The independent reference passes after one audio-graph restart: 160,000
samples, ten seconds of timestamp span, no uncovered gaps or overlap, a
9.606-second delivery span and five pulses. Window/microphone clip
`1794019648-181544836-2` completes recording, the stable five-second pause,
Resume and Finish. Its 192 video buffers average 4.205 fps with a 0.666-second
uncovered gap. Its 46.040 seconds of PCM have no detected holes or overlap;
track endings differ by 306 milliseconds. Only two visual flashes match, with
a median offset of -0.7405 seconds. Video continuity and synchronization fail.
SHA-256 is
`611f754dcdafacd855ad6cb47363063ffcc37442d0b5b717930adb9c53c6680d`.
The initial export's accessibility flow times out; its later native Save
choice is checked separately rather than credited to that first flow.

Screen clip `1794020124-591763283-2` completes the full control, naming,
library and native export flow without a retry. It has exactly one video
track and no audio track. Its 41 buffers average 1.651 fps and have a
2.741-second uncovered gap, so media continuity still fails.
`Product demonstration 14.webm` matches the original with SHA-256
`71a3190468b5d62820a5c96969b1b02ffe6c4f7d3e59ca7b1cda49cd1bfa1d34`;
document grant `8268a5bc` gives only Studio read/write access. Evidence is in
`installed-glyph-cache-window` and `installed-glyph-cache-screen`.

A subsequent one-time profile outside recording acceptance has ten of sixteen
samples in scanout allocation/upload or readback conversion, and none in text
shaping or glyph rasterization. It is preserved as
`installed-glyph-cache-screen/compositor-samples-025956.txt`; it is not a
recurring CI measurement or a measured speed ratio. The software renderer
currently draws RGBA, exports it, and swaps every pixel to BGR for scanout.
The prepared change instead draws and exports XRGB, retaining its BGR channels
and clearing the unused byte with a word mask. GLES conversion is unchanged.

A separate four-round VM comparison checks identical output for twelve
1280 × 800 frames. Current conversion takes 0.236–0.408 seconds, packed RGBA
conversion 0.272–0.376 seconds, and native BGR copying 0.125–0.148 seconds.
Only the native-format result improves every round. This is not a pipeline
acceptance measurement. The new byte-layout regression covers asymmetric
channels, odd dimensions and malformed lengths; the complete existing software
renderer pixel checks are added to dedicated optimized CI. Installed media
verification of this change remains pending.

The transport loop also writes its retained full-resolution frame again on
every iteration, followed by a 33-millisecond sleep, even if the compositor
has published nothing new. Delivery now tracks a publication generation and
writes each captured frame once per consumer. Fresh captures with identical
pixels still advance the generation. The worker retains only its current
in-flight image beside the newest slot and continues checking source lifetime
and authority while idle. Real socket regressions cover idle delivery,
fresh identical captures and revocation with no new frame; further checks
cover dropped intermediate images, generation wrap and the existing blocked
writer and exact service-owner checks. CI and installed verification of this
additional change are pending; no synchronization result is inferred from it.

### Fresh-publication verification and stage tracing

Source `605d72a3` passes native, optimized desktop, strict, authors and OS-gate
CI. The installed desktop SHA-256 is
`12595953a3019bc11bb85eb0f6b6689e531e425455abd30d79453ec946a57994`.
These checks do not establish recording quality. The normally installed Studio
package remains source `9772d083`, with no verification feature enabled.

The native-format repeat at `83d2f4a` fails media acceptance. Window clip
`1794022241-861431505-2` stops before Pause when its audio queue reaches 200
buffers. Its saved audio is effectively silent despite a passing independent
precheck. Screen clip `1794022677-814956935-2` completes the full flow and exact
native export, but averages 1.855 fps with a 1.697-second uncovered gap.
Evidence is in `installed-native-format-window` and `installed-native-format-screen`.

At `605d72a3`, window/microphone clip `1794023270-289522646-2` completes Start,
the stable five-second pause, Resume, Finish, naming, library and exact native
export. Its 22 frames average 0.475 fps, with a 4.384-second uncovered gap.
Its 48.580 seconds of PCM have continuous timestamps but are effectively silent;
the track endings differ by 1.057 seconds. Synchronization is not accepted.
`Product demonstration 16.webm` matches SHA-256
`7f607ac9e605d727ef22541a8509b07fe0e5f945bf8a9da71ead3468ac943a48`;
document grant `64f823fb` gives only Studio read/write access.

Screen clip `1794023971-76992512-2` also completes the full installed flow and
native export. It has exactly one video track and no audio track. Its 21 frames
average 0.919 fps and have a 1.439-second uncovered gap, failing continuity.
`Product demonstration 17.webm` matches SHA-256
`f64a473a86656d66c9fb64118cd75b237b0f1d0279165255f040dabf6d07ee2f`;
document grant `d97d1869` is scoped to Studio. Evidence and failed acceptance
results are in `installed-fresh-frame-window` and `installed-fresh-frame-screen`.
The generation change exposes actual publication cadence; it is not credited
as a verified performance improvement.

A separate one-time trace, with explicitly disclosed timing preloads on both
publisher and consumer, is preserved in `diagnostic-fresh-frame-screen-0420`.
It is diagnostic evidence, not media acceptance. Publisher input advances from
frame 1 to frame 60 across 81.604 seconds, with its reported queue empty. All 34
observed PipeWire pool acquisitions finish within 33.5 milliseconds. Eleven
observed slow record-entry pushes through preprocessing average 773.7 ms wall
time and 485.5 ms thread CPU time; encoder output pushes are short. This locates
sparse ingress before publication and substantial conversion/scaling work
before encoding. The provider executable is restored and Studio relaunched
without timing preloads after collection.

A separate live process sample observes concurrent status and media-activity
`pw-dump` queries consuming substantial CPU. Both readers immediately restart
after an expensive completion when the start-based interval has elapsed.
The next candidate adds an interval after completion and a two-second recovery
interval after late or failed reads. A late quiet media result is refused rather
than clearing the indicator. Existing freshness limits and one-outstanding-reader
bounds remain. Regressions cover late quiet answers and recovery intervals for
both readers. CI and an installed repeat of this candidate remain pending.

An isolated, one-time comparison inside Studio's normal Flatpak runtime processes
eight 1280 by 800 BGRx frames into 576 by 360 I420 frames, in alternating order.
Conversion before scaling takes 4.476 and 4.162 seconds; scaling before
conversion takes 1.651 and 0.891 seconds. The combined converter/scaler takes
1.574 and 1.040 seconds. All cases reach EOS. Disabling ORC's optimized code is
slower in this comparison, so no such override is retained. This synthetic
comparison is not recording acceptance. The next application candidate moves
scaling before conversion in both recording and preview, retaining the chosen
size, cadence, format, permission source and frame-dropping policy. The existing
real-framebuffer and pinned-publisher CI flows exercise downscaled recording,
pause/resume, finalization and decoded output; a normal installed package repeat
is still required.

### Native microphone clock lifetime

Normal package `54eb8544` builds with all 28 local application tests passing,
and native CI passes its framebuffer and pinned-publisher recording checks.
Its bundle SHA-256 is
`e1f56b8866e66e8015d6f7a31f4b80836fe52385d0760b77aa9ed1e70601d354`.
The isolated installed application repeat retains desktop `605d72a3`.

A persistent reader of only the owned virtual reference monitor keeps reference
audio generation alive after the short health probe disconnects. The first
reference check passes; the subsequent idle-period check has continuous PCM but
a 10.547-second delivery span, outside the existing 10.5-second limit. That run
does not establish synchronization acceptance. No reference limit is changed.

Window clip `1794026261-466303989-2` begins recording, then its audio queue fills.
The subsequent Pause request and status read time out. Studio aborts in native
PulseAudio code, leaving a 1,646-byte partial rather than a completed recording.
The partial is preserved with SHA-256
`147161f51ca29b135a37b82ad15ffc97f491c22ed8da2bedbd2c26998d9b13da`.
The core, application journal and source journal are preserved in
`installed-preprocessing-window`; no full-flow or export success is credited.

The core's failing thread reads an audio clock through the PulseAudio plugin
while shutdown clears its mainloop. The matching
[GStreamer 1.26.11 source](https://github.com/GStreamer/gstreamer/blob/1.26.11/subprojects/gst-plugins-good/ext/pulse/pulsesrc.c)
confirms that manually moving the microphone to Null destroys this context.
The application currently does so before the other pipeline threads have
finished using its selected clock. The candidate instead corks the microphone
in Paused, locks it against parent resume, and retains its clock provider while
queued media drains. Normal parent shutdown then unlocks and closes the inputs
in pipeline order. Verification probes check stopped input states and the live
native clock provider during real microphone finalization. CI and another
normally installed repeat of this correction are pending.

Package `eb13d7f5` builds normally with all 28 local tests passing; its bundle
SHA-256 is `2b971ab4c4cee853ff305e7e2bf24fe006ac61342d16146eae3f12254c1d5cf0`.
Native CI passes, including the new provider-lifetime checks with real PulseAudio
inputs. Optimized desktop and strict checks also pass. The installed repeat uses
desktop `54eb8544`, SHA-256
`6bc42103d203e80e3c8339160ea362292e8d209536c4c2ea698f113a33a15f96`.

That repeat reaches a controlled early finish after audio-queue saturation and
saves clip `1794027671-18085922-2`, SHA-256
`bcc3f7834ca8f8c2a16076bf6d26d2c0c67b67300cb8f149a137e3e528932de6`.
It does not reproduce the native abort. The file contains five video frames
ending at 14.872 seconds, with a 3.984-second uncovered gap, and 10.880 seconds
of PCM with a 2.120-second uncovered gap. The independent audio reference also
fails its timing check. Neither continuity nor synchronization is accepted.
The normal control script times out and receives no successful pause/resume or
export proof. Evidence is in `installed-clock-lifetime-window`.

An explicitly diagnostic stack, captured after those control timeouts and saved
as `application-hang-stack.txt`, instead finds the GTK main thread inside
synchronous PNG loading from the recording library. Its media threads have
already ended. Library rows currently decode every saved preview synchronously,
including while that page is hidden. The next candidate opens and scales these
images asynchronously only when the library is mapped, cancels work when hidden,
and ignores stale completions. The native UI regression adds 128 large image
previews beside copies of an already verified recording and requires prompt
status responses while the library is opened and dismissed. This is a UI
responsiveness check, not a video-cadence benchmark. CI and installed verification
of this additional correction are pending.

The first library regression fails its unchanged two-second status deadline on
the hosted native runner. Async stream loading alone still performs image decode
work on the UI context. The correction runs the complete scaled decoder in GIO's
worker pool and returns only owned RGBA pixels to GTK. Hidden or replaced rows
cancel their queued results; the same 128-preview responsiveness check is retained.

A later CI run stops before the library regression because its fast recording
can hide the preview before the poll saves a thumbnail. The application now
preserves the actual selected-source preview before starting encoding. The
existing requirement for a saved real preview image remains unchanged. The
library check also records an unstressed status baseline before adding its image
fixtures, and thumbnail completions run below control callbacks in priority.

### Bounded software scanout reuse

A further one-time compositor profile at desktop `54eb8544` is preserved in
`native-render-investigation-current54/compositor-samples-051328.txt`. It still
contains readback conversion and repeated scanout allocation/upload among the
remaining costs. Screen selection completes during the early samples; later
samples have screen preview active. No recording or microphone is active during
this explicitly diagnostic profile, and the debugger is detached afterwards.

The next candidate lets only the software desktop retain one detached scanout
beside its active one. It retains at most two live buffers per fixed output and
alternates them after successful blocking commits. An active allocation cannot
be uploaded through the reuse path. Each reused frame is fully uploaded and
tested before activation; captured pixels are still published only after
successful presentation. Other presentation paths keep immediate retirement.
Output shutdown disables active scanout, releases the known-unbound spare and
retains all cleanup errors. Failed disable quarantines active storage; the
independently detached spare can still be released.

Seven added regressions cover target wiring, bounded rotation, pixels and binding
identities, reused upload/test/enable refusal with clean retry, cleanup-failure
latching, active-write refusal, failed-disable quarantine and aggregate shutdown
errors without retry. Existing immediate-retirement resource tests are retained.
CI and installed media verification of this candidate are pending; no whole-flow
performance improvement is inferred from the sampled stacks.

### Embedded playback dependency and library responsiveness

The hosted UI check's unstressed status baseline also timed out before the 128
preview fixtures were created. A one-time GDB stack at `febfd0f8` finds the main
thread joining GTK's playback thread from `show_library` while that thread waits
on a direct KMS video sink. The native test environment lacked `glsinkbin`.
GTK's [embedded renderer](https://github.com/GNOME/gtk/blob/gtk-4-14/modules/media/gtkgstpaintable.c)
returns no sink when a GL context exists but that element cannot be created,
allowing the playback engine to select a different sink. The normal GNOME 49
runtime does contain the OpenGL plugin (GStreamer 1.26.11), checked once locally.

The native workflow now installs `gstreamer1.0-gl`, and Studio checks for
`glsinkbin` with its other required media components. At `7dea46f2`, the hosted
framebuffer/UI step passes, including both the initial baseline and the unchanged
two-second responses while opening/dismissing 128 large library previews. The
pinned publisher workflow also passes, including real PulseAudio inputs and
stopped-clock-provider checks. Complete desktop/OS gates are still running. This test runs
in the native workflow on subsequent relevant changes; it does not establish
installed recording cadence or synchronization.

### Installed repeat after scanout reuse

Desktop `1f36f9ef` passes the native workflow, optimized desktop regressions,
strict checks and the full OS gate. Its executable SHA-256 is
`0fd1253fb72ee8129a7dcae0bfbf38dad4087c6977570f28d8a3d41e0a73df4f`.
The installed normal application is package `7dea46f2`, bundle SHA-256
`adcd8bef984e9810de175cfa735dcd77f9cda2a0ed373854c8653533b5777793`;
its application source is identical to the application tested at `1f36f9ef`.

Both installed flows now complete capture, stable five-second pause, resume,
finish, naming, library navigation and native exact-file export. Window clip
`1794032538-326619305-2` has SHA-256
`71db8ad192115dc2b64cb6465a767ca6567f1d0e8f79606f0ad94ec5aa04b7e1`;
its export is `Product demonstration 18.webm`, document grant `27389a9f`.
It has 19 distinct video frames, 0.536 measured fps and a 4.155-second uncovered
video gap. Microphone PCM is audible and has a maximum 20 ms uncovered gap, but
track endings differ by 867 ms. Only four visual markers match, insufficient to
establish synchronization. Evidence and the failed unchanged acceptance report
are in `installed-scanout-reuse-window-repeat`.

Screen-only clip `1794032819-682408489-2`, SHA-256
`d8b0cac0b67656687a808d6c62ce12c050456aa80b9c668629fd37a330d676be`,
contains exactly one video track and no audio. Its export is
`Product demonstration 19.webm`, document grant `6dd0e122`. It has 25 distinct
frames, 1.132 measured fps and a 1.626-second uncovered gap. Its control flow
passes and media acceptance fails in `installed-scanout-reuse-screen`.

The window fixture uses a non-live audio generator paced by its playback clock;
the visual marker reads playback position. A one-time independent check passes
when its owned virtual audio graph is temporarily forced to its native 16 kHz
rate. The full-flow reference checks still exceed the existing delivery-time
limit, so they do not establish synchronization acceptance. The forced rate is
restored to zero after the run, recorded in the evidence; no production audio
setting or acceptance threshold changes.

Further one-time diagnostics are preserved in
`native-render-investigation-scanout1f`. GDB samples find software readback copies
and repeated native sentence shaping. Breakpoint phase timings include material
debugger overhead and are not treated as production durations. A local syscall
trace finds 26 atomic calls in about 20 seconds, 226 ms total, 26 ms maximum;
these do not account for the observed second-long frame periods. Kernel uprobe
registration is rejected by the guest; its private instance and any owned events
are removed. All debuggers detach and the temporary forwarding port is closed.

An alternating synthetic copy benchmark is performed once in the VM with
recording and preview released. Sixteen masked copies take 0.197–0.442 seconds;
bulk copies take 0.103–0.146 seconds. This saves milliseconds per frame and does
not, by itself, explain the recording gaps. No pixel-format contract changes
are made on that evidence.

The next candidate caches fully shaped native sentences by full text, wrapping
width, font metrics, background and ink. It retains at most 128 entries and
4 MiB of pixel/key allocation, skips text over 4 KiB and outputs over 256 KiB,
and returns independent owned pixels. Its font-match allocation identifies the
font database: cosmic-text replaces that owner after `db_mut`, and a new font
system has its own owner. Locale also invalidates the cache. Rich text and
terminal runs keep their existing renderer. Added tests cover actual pixel
equivalence, dimensions, scripts, font invalidation, independent returned pixels,
eviction and retention limits. Native, optimized desktop, strict and full OS
checks pass at `c7400bc8`.

### Installed sentence-cache repeat

The one-time repeat stages desktop `c7400bc8`, executable SHA-256
`81d638c528c304330bf04bd85db1c141dff2f766d0447ceaedbb9bcd3e658aac`,
with the unchanged normal application package `7dea46f2`. The shared build lock
is held and no host compiler runs during the measurement. An initial screen
attempt fails because the runner did not relaunch Studio after replacing the
desktop. It produces no recording and is preserved separately; the retry adds
an explicit bounded application-readiness check.

Window attempt `1794038599-415841006-2` does not reach active recording. Its
journal reports 34,240 dropped microphone samples, then no recorded video frames.
The 759-byte partial is preserved with SHA-256
`2aa30db4bf9404bd1905d5356d9446cf8c1f521c7ed8a96e8c6277156f0f96b1`.
There is no completed metadata or video to decode, and no pause/resume, export
or media acceptance is credited. Evidence is in `installed-sentence-cache-window`.

The independent reference also fails before this attempt: its first 10 seconds
of PCM arrive over 11.424 seconds. After one audio-graph restart, the reference
has a 2.120-second uncovered gap; the check after idle has a 2.080-second gap.
These failures prevent synchronization acceptance independently of Studio.
No thresholds or production audio settings change. Cleanup restores the original
microphone, removes the virtual audio module and resets the owned forced rate.
A proof-file ownership error in the private cleanup helper is corrected and
cleanup is repeated successfully.

The screen-only retry completes recording, stable five-second pause, resume,
finish, naming, library review and exact native export. Clip
`1794039162-193924091-2`, SHA-256
`fa7ab858c6a9a4b76314ad4dad54895a75003ae4cf3407ccd72bf8d12fdd0975`,
contains 24 distinct frames over 25.482 seconds, with exactly one video track
and no audio. The export is `Product demonstration 20.webm`, document grant
`234be`. Independent decoding measures 0.910 fps against the selected 5 fps
and a 2.197-second maximum uncovered gap. The unchanged formal acceptance
fails both cadence and continuity. Evidence is in
`installed-sentence-cache-screen-repeat`. This is a one-time installed repeat;
native CI continues to run its separate publisher, microphone, lifecycle and
decoding checks. The compositor candidate has not established an installed
performance improvement. Continuous video and synchronized microphone audio
remain unaccepted on the available software-emulated VM; the review stays draft.

### Native pixel-run submission investigation

A further one-time diagnostic at `c7400bc8` wraps pixman's compositing calls.
The hook is loaded correctly; an initial service-name journal query misses its
output because the desktop's PAM session changes its journal attribution.
The process-name query recovers the logs. About 2.88 million calls occur over
399 seconds in the idle desktop. The reported full-output dimensions are the
backend's composite extent, not proof that every call copies the whole output:
Smithay clips these operations to each supplied damage region. The hook makes
two thread-CPU clock reads per call. That added syscall load materially affects
timings, so the call count is diagnostic evidence, not a native frame-rate or
uninstrumented CPU-cost measurement.

Native opaque text currently submits each horizontal run separately. The next
candidate groups disjoint runs of the same colour within one image, submits at
most 4,096 runs per batch and completes that image before any overlapping next
image. This makes the existing promise of native drawing true more efficiently;
it adds no user-facing capability and changes neither pixels nor layer order.
Regression checks cover exact pixel coverage, reduced submissions, bounded
flushing, immediate failure propagation and the actual software renderer with
overlapping images. The native workflow runs these checks on future changes.
CI and installed verification of this candidate are pending.

While this hook is active, a kernel counter sample finds 102,502 minor faults
and 11.39 seconds of main-thread kernel CPU time over 15.27 seconds. The added
clock syscalls mean this cannot establish the normal compositor's kernel cost.
Temporary allocator settings are explored separately after removing the hook;
the instrumentation change also prevents a causal before/after comparison.
The first allocator comparison overlaps startup and is rejected;
another sample pauses the parent validation but misses compiler children, so
it is not treated as a quiet-host performance result. The parent is resumed
after 28 seconds, recorded with its process identity. No allocator improvement
or media acceptance is claimed. Both diagnostic drop-ins are removed, the
normal desktop services restart, and no allocator setting is added to the OS.

Integrating main `6d93b3b4` retains both the capture-protocol and pointer-setting
dependencies and both capability amendments. Main has assigned ADR 0147 to the
screen road, so the draft recording decision and its references become ADR 0149.

### Installed pixel-run batching repeat and scratch-image follow-up

Desktop `a46144c1`, executable SHA-256
`b46026314a406e6cc1f194912e41553e0b1220ec2a03bb7e9f7561acc99190ab`,
passes native recording, optimized desktop, strict checks and the full OS gate.
Its one-time installed repeat uses the unchanged normal Studio package. With
the person's explicit approval, the other validation's Cargo process is stopped
before it starts a test binary; process identities and states are checked before
recording. No host compiler runs during either flow. The validation is resumed
after cleanup, recorded from 11:00:50 to 11:16:30 UTC on 7 November 2026.
Its build lock is retained throughout by its original owner. A separate timed
resumption safeguard does not fire.

Screen clip `1794049504-496662537-2`, SHA-256
`baffdae4391cf9e24a12d25830c6a7de238817d1c316aaaf596041649375a16a`,
contains 34 distinct frames and exactly one video track without audio. Its
export is `Product demonstration 21.webm`, document grant `ff63078c`. Its 1.436
measured fps and 1.830-second uncovered gap fail media acceptance despite a
complete control flow. Evidence is in `installed-pixel-run-screen`.

Window clip `1794049967-124983050-2`, SHA-256
`96bd5359d400edddf5dcced02b1a07ca96edba5930ceecd87e27ac96599bf142`,
contains 31 distinct video frames and 29.540 seconds of audible 16 kHz mono PCM,
with no measured uncovered audio gaps or meaningful overlap. Its export is
`Product demonstration 22.webm`, document grant `3c9a50b0`. Its 1.053 measured fps
and 1.679-second uncovered gap fail video acceptance. Track endings differ by
95 ms, but only one visual marker matches and the reference checks fail, so
those endings do not establish synchronization. Evidence is in
`installed-pixel-run-window`; no acceptance threshold changes.

The next candidate retains at most two detached pixman scratch images, with
64 MiB total retained stride allocation, plus one checked-out image while
painting. Only software desktop constructors opt in. Other callers retain
immediate retirement. The renderer target borrow ends before an image returns
to the pool, and readback still exports independent owned pixels. A failed
frame can return scratch storage, but a later successful desktop or window
paint clears its full output before drawing. This makes the existing promise
of software rendering true with fewer repeated image allocations; it adds no
capability. No engine patch or unsafe access is introduced.

Regressions check exact dimensions and exclusive ownership, two-image eviction,
actual stride-byte limits, oversized-image bypass, full clearing after refusal,
scratch identity reuse and immutable earlier snapshots. The Studio workflow
also starts for shell and desktop changes so these checks and the installable
artifact are regenerated. Native recording, optimized desktop regressions,
strict checks and the full OS gate pass at `0b3dc75a`.

### Installed scratch-image reuse repeat

Desktop `0b3dc75a`, executable SHA-256
`a1cb1a94ece43ef9eb90bffb01f6250dc3f939aabf026b5b4d4796eb2e081e92`,
is installed with the unchanged normal Studio package. Both complete control
flows pass: stable five-second pause, resume, finish, naming, library review
and byte-identical native export. The other validation's Cargo and compiler
processes are paused with explicit person approval and checked by process start
identity. No host compiler runs during recording. Both processes are resumed
after cleanup: 11:41:08 to 11:55:27 UTC on 7 November 2026, recorded in
`installed-software-frame-reuse-validation/validation-pause-and-resumption.json`.
Its original owner retains the shared build lock. The timed safeguard does not
fire. These are one-time measurements, not a controlled performance comparison.

Screen clip `1794051872-109567679-2`, SHA-256
`2cced51c5e8a85c6ee4115179cb111170c559ef318355d77201a4cc4847aeba6`,
has 67 decoded frames, 65 globally distinct images, and a changing image at every
adjacent frame. Its measured cadence is 2.669 fps with a 1.159-second uncovered
gap. It contains one video track and no audio. Export `Product demonstration
23.webm` receives document grant `2d38449a`. Evidence is in
`installed-software-frame-reuse-screen`.

Window clip `1794052310-454591843-2`, SHA-256
`25d6a514bab1517588c92c39c3ab1fc443e767984ee2c42f1c934b28f9cdc87f`,
has 54 distinct decoded video frames, measured cadence 1.349 fps, and a
2.338-second uncovered video gap. Its one audio track decodes to 40.400 seconds
of audible 16 kHz mono PCM, with a maximum 20 ms uncovered gap and no meaningful
overlap. Track endings differ by 39 ms. Only three visual marker matches are
available, and the independent reference fails; synchronization is unverified.
Export `Product demonstration 24.webm` receives grant `41bf1ce7`. Evidence is in
`installed-software-frame-reuse-window`.

Both media checks fail the unchanged 5 fps continuity requirement. The window
check also fails the independent reference and synchronization requirements.
Neither a close track ending nor successful pause controls establishes media
acceptance. All media and earlier partial files are preserved.

A passive two-read `/proc` sample before recording, during source setup, sees
80,414 minor faults over 15.295 seconds, with 3.90 seconds of user CPU and 6.42
seconds of system CPU. No preload or allocator override is active. This is an
allocation lead from a changing setup workload, not a recording measurement or
a paired comparison with the previous desktop.

### Native ink raster candidate

A separate one-time CPU-clock sample of the normal desktop collects 1,021 user
instruction pointers over 20.016 seconds, with zero lost samples. It uses
`perf_event_open` and closes its sampling descriptor afterward; no injected
library, breakpoint, stack dump or persistent kernel probe is installed. The
competing validation is running again, so this is a cost-location diagnostic,
not a quiet-host timing or media acceptance measurement. Of those samples,
479 are in pixman, 389 in the desktop and 146 in libc. Symbol-size and ELF LOAD
mapping locate 80 in native opaque painting, 75 in full-frame XRGB copying and
32 in artwork scaling. Shared-library samples are identified by library, not
guessed private function names. Evidence is in `diagnostic-passive-cpu-scratch`.

The next candidate uploads the Dock's ink and the in-use indicator's ink as
cropped RGBA layers before opening the frame, then draws them at their original
positions in layer order. Each original image remains opaque, later images
replace earlier images, and gaps are transparent. This avoids reconstructing
and submitting hundreds of colour regions for the same labels on every frame.
The temporary raster is capped at 8 MiB per collection; larger or malformed
collections retain the existing run painter. Import and allocation failures
remain explicit rendering errors. No renderer patch or unsafe access is added.
Regressions cover sparse gaps, overlap order, unchanged inputs, bounded fallback,
overflow refusal and exact software-rendered indicator pixels, including output
clipping. Validation and the installed recording repeat are pending.

At `a29f347c`, the native application check, optimized desktop regressions and
strict desktop checks pass. Its installed repeat is recorded below.
The first candidate is refused by Clippy's fixed-size-chunk rule; the correction
uses `as_chunks_mut::<4>()` and does not suppress the check.

Its subsequent one-time installed repeat completes both control flows and exact
native exports under the shared Cargo lock, with no competing job paused.
The verification reservation runs from 12:28:23 to 12:45:07 UTC on 7 November
2026, including a wait for another formatter before acquisition. No compiler
runs during recording. Screen clip `1794054877-879342217-2`, SHA-256
`bbf389a3e31a8a37c5eeffe81dbdb5ddef0378d352d688ff986d7282721b1eac`,
decodes to 60 frames (55 globally unique images), 2.593 measured fps and a
2.102-second uncovered video gap. Export `Product demonstration 25.webm` has
document grant `5f3c5d0d`.

Window clip `1794055291-302898550-2`, SHA-256
`d0ab25d50b4af43dcb8f15d29820a46416d3772686ac37341bea60ee531d9257`,
decodes to 53 distinct video frames, 1.386 fps and a 1.892-second uncovered video
gap. Its audio is audible, decodes to 38.260 seconds of PCM with no meaningful
gaps or overlap, and ends 125 ms after video. The independent reference passes
after the idle recheck (10.395-second delivery span, ten seconds of PCM, seven
pulses), but only three video markers match; synchronization is unverified.
Export `Product demonstration 26.webm` has grant `fdf33904`. These results do not
establish a video-quality improvement. Evidence is in `installed-native-ink-screen`
and `installed-native-ink-window`; restoration is in `installed-native-ink-cleanup`.

### Display mapping lifetime candidate

A separate one-time kernel CPU sample of desktop `0b3dc75a` collects 541 samples
over 20.057 seconds, with none lost. Of these, 172 locate `do_user_addr_fault`,
23 `__handle_mm_fault`, 13 `handle_mm_fault` and 57 inter-CPU function calls.
Kernel symbols come from that guest's own symbol table. This diagnostic runs
with competing validation resumed and does not measure quiet-host media quality.
Evidence is in `diagnostic-passive-kernel-scratch`.

A minor-fault sample then records one fault per 100 events, including mapping
notifications. Of 731 sampled faults, 560 resolve to `/dev/dri/card1`, 122 to
the heap and 49 to anonymous mappings, over 20.285 seconds; none are lost and 56
mapping notifications arrive. A second 20.159-second sample preserves the full
mapping/sample event sequence: 1,210 of 1,873 fault samples resolve to that DRM
device, with 121 mapping notifications and none lost. Workload and competing
validation differ between samples, so their event rates are not a performance
comparison. Both sample the unchanged normal desktop before installing the ink
candidate. All sampling descriptors close afterward. Evidence is in
`diagnostic-passive-page-fault-scratch` and its `-repeat` folder.

Source inspection shows each upload calls `map_dumb_buffer` and drops the
mapping, even when the detached GEM buffer is reused. The next candidate keeps
one lazy mapping per owned buffer, using the safe `self_cell` 1.3.0 API already
locked through cosmic-text. No new registry package or engine patch is added.
Only unbound allocations expose upload callbacks. Successful mapping admission
is reused, mapping refusal leaves no cached entry, and upload refusal retains
ownership for the existing allocation cleanup. The mapping is dropped before
GEM destruction. Uncertain scanout retirement still quarantines kernel handles
until descriptor teardown. No owned pointer or mapping escapes its callback.
Regression checks cover single admission across repeated uploads, exact byte
writes, mapping refusal without upload, and exactly-once mapping retirement
after an upload refusal. CI, installed capture and the fault recheck are pending.

### Installed retained-mapping repeat

All four checks pass at desktop `34d37b4e`: native application recording,
optimized desktop regressions, strict checks and the full OS gate. The installed
executable SHA-256 is
`4f5c5b8ffbacae7b451b25d611df6093763f82c022c524bdf92c9824bcc81068`.
The person-approved pause stops the other validation's Cargo test compilation
and four compiler children before any test executable starts. Each process is
checked by start identity and resumed after cleanup, from 12:46:52 to 12:56:13
UTC on 7 November 2026. Its original owner retains the shared build lock; the
twenty-minute resumption safeguard does not fire. The process snapshot and
resumption proof are in `installed-retained-mapping-validation`.

Screen clip `1794055757-99418837-2`, SHA-256
`92e3ae8062efc1b70a095a41396e0b3159f8b4597df8179ef127cbf46c9ec282`,
has 78 decoded frames, 63 globally unique images, 3.736 measured fps and a
0.709-second uncovered gap. Export `Product demonstration 27.webm` has grant
`6cbefd39`. Window clip `1794056012-848234452-2`, SHA-256
`99f41be33831ffa344f12ccd1297b4e10cdfc28302bc2db255fd94056da6f357`,
has 90 decoded frames, 89 globally unique images, 2.535 fps and a 0.966-second
gap. One decoded video timestamp repeats near resume. Its 35.700 seconds of
audible PCM have no meaningful gaps or overlap; track endings differ by 77 ms.
Both independent reference checks pass (10.176 and 10.190 seconds of delivery
for ten seconds of PCM, six pulses each). Six visual/audio matches are still
insufficient to establish synchronization. Export `Product demonstration
28.webm` has grant `e3d475e0`.

Both complete controls, stable five-second pause, correct microphone states,
review, naming, library navigation and exact native export. Both fail unchanged
video continuity requirements. Window media also fails monotonic timestamps
and synchronization. These are one-time installed observations in
`installed-retained-mapping-screen` and `installed-retained-mapping-window`,
not a paired performance comparison or demo-quality acceptance.

After cleanup and compiler resumption, two concurrent passive diagnostics sample
the normal ready desktop. A 20.085-second minor-fault sample receives no mapping
notifications and no sampled DRM faults: 232 samples resolve to heap, 95 to
anonymous memory and two to stack. This supports removal of the remapping
mechanism without claiming zero unsampled faults. A 20.082-second user-CPU
sample records 1,146 instruction pointers: 528 in pixman, 440 in the desktop,
and 172 in libc. Desktop symbols include 116 readback-copy samples, 47 artwork
scaling samples and 37 native-ink upload samples. Neither loses records; both
close their sampling descriptors afterward. These locate remaining costs and
are not quiet-host timing measurements. Evidence is in
`diagnostic-retained-mapping-page-faults` and `diagnostic-retained-mapping-cpu`.

### Recording timestamp admission and bulk-copy follow-up

Incoming video receives the pipeline's current arrival time. A stopped or
slowly advancing sample clock can stamp multiple arriving frames at the same
time, including after resume. The new recording-local admission guard remains
installed before the encoder across pause/resume. It refuses missing, repeated,
backward or sub-millisecond advances. A new recording starts with a fresh guard.
It never moves accepted timestamps, changes duration, pads video or creates
duplicate frames. The muxer explicitly selects its existing one-millisecond
timecode scale. Regressions exercise a stopped/rewound clock and pass adversarial
timestamp bursts through VP8, WebM muxing and actual decoding.

The software readback follow-up preserves the existing validated BGR layout,
copies native channel bytes in bulk and clears only the unused fourth byte.
It retains fallible allocation and malformed-layout refusal. Existing readback
and actual software-renderer pixel tests cover its byte contract. Both follow-up
changes still require validation and another normally installed recording walk.

The follow-up's native application check, including the new decoded-WebM
regression, and optimized desktop/strict checks pass at `dad76731`. The local
normal-package build is queued behind another test run through the shared Cargo
lock. The `Studio installable package` pull-request/manual workflow provides a
separate hosted GNOME 49 SDK build using the same production manifest and build
script. It produces an unsigned bundle, source revision, checksums and installed
SDK versions as a review artifact, without enabling verification features or
publishing to an application market. Hosted package validation and installed
recording acceptance remain required before treating that artifact as tested.

### Normal hosted package and installed timestamp repeat

The normal package build passes at `bbc88582` in hosted job `128422292746`,
run `42378949706`. Artifact `13576534432` has archive SHA-256
`b4a019ecad1a95dd97240392258fcf6f216b8ca50ea467408f7b88dd84cd7b6f`.
Its five whitelisted members include the bundle, exact source, disabled
verification-feature declaration, SDK version/commit-prefix listing and payload
checksums. The archive and every payload checksum are verified before normal
Flatpak installation. Installed Flatpak commit is
`440b112795b6df55cc0cc4edb950aa114ed4ecb92bcc5ab96f157c5e3f244779`.
Desktop `dad76731` is the separately tested hosted executable with SHA-256
`681d33343b5576f9f4388af3b0b2f9dac838c3949ce8d5b2732de996c9bd7031`.
The agent and capture binaries retain their separately recorded sources; this is
still a temporary OS overlay, not acceptance of a newly composed image.

The host DATA volume fills during artifact recovery and the WSL session restarts.
The owned 4.824 GB build-cache archive is preserved on C: with its full checksum
verified before removing its old D: copy. No media or VM disk is removed. The
same raw VM disk restarts with its existing four-CPU software profile, and the
transient native integration is restored. A first installed attempt gets one
ready response, then a control timeout before any recording. Its compiler pause
runs from 14:10:48 to 14:13:18 UTC on 7 November 2026 and all five processes resume.
The application subsequently answers repeated control requests. The retry retains
that already validated, warm desktop/application session. Its approved compiler
pause runs from 14:14:09 to 14:23:03 UTC, again resuming all five processes; no
running test executable is paused and neither resumption safeguard fires.

Screen clip `1794060949-363198639-2`, SHA-256
`2bbe9b8cc6715b74c9bed7105a46754ee9bce254b40a7aed8960ef7ab7278b58`,
has 65 decoded frames, 60 globally unique images, 2.891 measured fps and a
0.750-second uncovered gap. Export `Product demonstration 29.webm` has grant
`a96281ac`. Window clip `1794061213-118221434-2`, SHA-256
`4dbf4aef1ddea63954c0f471be0753d276a1a2bd0e5b8bd0fa9fd2c8e1b50522`,
has 144 distinct decoded frames, 3.800 fps and a 1.178-second gap. Neither
contains a non-increasing video timestamp. Its 37.920 seconds of audible 16 kHz
mono PCM contain no meaningful uncovered gap or overlap; track endings differ
by 80 ms. Independent reference delivery spans are 10.252 and 10.219 seconds
for ten seconds of PCM, with six and seven pulses respectively. Eight of 22
audio onsets match visual markers: median video delay is 672.5 ms, maximum
1,054.5 ms, so synchronization fails. Export `Product demonstration 30.webm`
has grant `5f2684ff`.

Both flows pass stable five-second pause, microphone off while paused and after
finish, correct resumed state, review, naming, library and exact-file export.
Both fail cadence and uncovered-gap acceptance; the window also fails sync.
Evidence is in `installed-timestamp-order-screen`,
`installed-timestamp-order-window`, and the two validation folders. Cleanup
checks pass at 14:25:27 UTC: normal app ready, microphone disabled, original
source restored, reference graph removed, forced rate zero, chooser size restored,
installer unfrozen and no debugger, kernel probe or timing preload. Cleanup
proof is in `installed-timestamp-order-cleanup`. All recordings and partial
files remain preserved. None of these one-time observations proves a standing
frame rate or hardware acceptance.

### One-pass native copy candidate

A separate 20.131-second user CPU sample after competing validation resumes
collects 1,219 instruction pointers without loss: 527 in pixman, 424 in the
desktop and 264 in libc. The desktop symbols include 123 in native readback
copying, 47 in artwork scaling and 12 in native-ink upload. Sampling descriptors
close afterward. This locates costs in the normal ready desktop; it is not a
quiet-host performance comparison. Evidence is `diagnostic-timestamp-order-cpu`.

A standalone optimized Rust comparison alternates six pairs of 32 full-size
copies in the same VM, with competing validation active and no recording.
Bulk copy followed by scalar X-byte clearing takes 0.337–0.536 seconds; a
four-pixel mask-and-copy takes 0.132–0.239 seconds. Exact equality checks cover
whole and partial blocks before timing. The retained source, executable checksum
and output are in `diagnostic-wide-copy-comparison`. An earlier exploratory
invocation also favors the wide copy but has no retained output file; only the
retained repeat supplies these ranges. This supports the local operation, not a
claim about recording cadence.

The candidate uses safe byte-array/u128 conversions, retains BGR order and zero
unused bytes, reserves the complete allocation fallibly before writing, and
handles every complete-pixel tail without another allocation. A regression
compares exact channels and stride across widths one through seventeen; existing
malformed-layout and actual software-pixel tests remain. CI and installed
recording acceptance of this additional change are pending.

### Installed one-pass native copy repeat

All five hosted checks pass at `6ebf9396`. Desktop job `128431159098` in run
`42381919756` produces artifact `13578735155`, archive SHA-256
`6612e651e3ef3ab7b72879538ec564170b11ec1f4e5c761f91b40c5d0bef7325`.
The checked executable SHA-256 is
`09734d7a06abc7ac3938bbcaaa33a66d37b38aeba2573493827ba3738ef9cea7`.
The normal application remains package `bbc88582`; both application subtrees
are `af45ac889b25e47507c894118b8951fc5e1c3408`. No new app package or diagnostic
executable is substituted for this OS-only performance trial.

Screen clip `1794062700-874405166-2`, SHA-256
`4f36b818cf516a91c4e3636b797f441e5765eaedfdf47c33ebf80c54be6134a9`,
has 94 decoded frames, 81 globally unique images, 4.367 fps and a 0.330-second
uncovered gap. It has exactly one video track and no audio. It passes the gap
limit and fails the unchanged 4.5 fps minimum. Export `Product demonstration
31.webm` has grant `3cec06de`.

Window clip `1794062944-420682466-2`, SHA-256
`1f3d5377df3b6934dbb55d8e6e3ab977757aa83ec01ded493d752308241614c7`,
has 134 distinct decoded frames, 3.742 fps and a 1.130-second uncovered gap.
Exactly one video and one audio track are present. Its 36.160 seconds of audible
PCM have no meaningful gap or overlap; track endings differ by 105 ms. Video
timestamps increase strictly in both clips. The first reference check passes
with 10.252 seconds of delivery; the post-idle check takes 11.221 seconds for ten
seconds of PCM and fails. Three marker matches with median 403.5 ms and maximum
710.5 ms delay do not establish sync. Export `Product demonstration 32.webm`
has grant `af86ad54`.

Both controls pass five-second pause with fixed elapsed time and microphone off,
correct resumed microphone state, final microphone shutdown, review, naming,
library navigation and exact native export. The approved compilation pause runs
from 14:42:14 to 14:51:45 UTC on 7 November 2026; all five processes resume and
the safeguard does not fire. Cleanup checks pass at 14:52:37 UTC with normal app
ready, no reference audio override or module, original source restored, forced
rate zero, original chooser size and unfrozen installer. Media, execution logs,
source identities, pause/resumption proof and cleanup remain in the
`installed-wide-copy-*` folders. These are one-time installed measurements,
not a paired performance claim; no threshold changes.

### Bounded public artwork scaling candidate

The retained user CPU sample also identifies 47 artwork-scaling samples in
1,219 instruction pointers. Source inspection shows the same icons are averaged
at each frame even when their bytes and requested size are unchanged. The next
candidate shares immutable original/output bytes across picture clones and
retains exact-sized variants by original allocation identity, source dimensions
and target dimensions. Holding the source allocation prevents address reuse
from matching a different picture. New artwork gets a fresh identity. Pixel
comparisons and public immutable access retain their existing meaning.

Retention is per drawing thread, at most sixteen entries and four MiB including
source/output lengths, conservatively recounting shared sources per entry.
Metadata is separately bounded by entry count. Oldest used entries are evicted;
oversized results use the original arithmetic without displacing the cache.
No window or application document pixels are retained. Added regressions cover
hits through clones, exact pixel results, changed size and replacement,
byte-budget eviction, oversized bypass, entry limits and recent-use ordering.
The optimized Studio desktop workflow now includes these Dock tests and strict
checks. CI and installed recording acceptance are pending for this candidate.

### Installed bounded-artwork repeat

All five hosted checks pass at `74c3fef4`. Desktop job `128436708504` in run
`42383770336` produces artifact `13578079808`, archive SHA-256
`b19b219332036c21d33b0f8c0a629803442e17f00dd0de40692ecd5c3f598b6e`.
The installed executable SHA-256 is
`531fc014ba023e7c902f639c0cfd5257a4b5c3a4b01c74fbf034189c7ef4b059`.
The normal application stays on the unchanged `bbc88582` package.

Screen clip `1794064314-752154456-2`, SHA-256
`74f742833d222088250df885b5b2b08b455bf09acd5946f1adc3213d878890f1`,
has 104 decoded frames, 83 globally unique images, 4.793 fps and a 0.711-second
uncovered gap. It passes cadence and fails the gap limit. The largest uncovered
interval is 11.963–12.674 seconds, close to the saved pause boundary of 11.824
seconds. This locates the symptom; it does not by itself prove which source
transition causes the wait. Export `Product demonstration 33.webm` has grant
`f7e6e33c`.

Window clip `1794064574-359315779-2`, SHA-256
`b8f4973d6752785a7c04d5b065c5aefb6081626515315cd321f619a103d06517`,
has 97 distinct decoded frames, 2.621 fps and a 1.405-second uncovered gap.
Its 37.220 seconds of audible PCM contain no meaningful gap or overlap; track
endings differ by 398 ms. Both references pass: ten seconds of PCM arrive over
10.160 and 10.305 seconds, with seven and six pulses respectively. Six of 22
audio onsets match visual markers; median delay is 653.5 ms, maximum 885.5 ms,
so synchronization fails. Export `Product demonstration 34.webm` has grant
`a12afb1e`. Both videos have increasing timestamps and exact expected track
counts, but neither passes full media acceptance.

Both flows pass stable five-second pause, correct resumed microphone state,
final microphone shutdown, review, naming, library and exact native export. The
approved compiler pause runs from 15:09:08 to 15:18:57 UTC on 7 November 2026;
all five processes resume, no test executable is paused, and the safeguard does
not fire. Cleanup checks pass at 15:22:11 UTC with normal app ready, microphone
off, original source and chooser restored, reference module removed, forced
rate zero and unfrozen installer. All evidence is preserved in
`installed-artwork-cache-*`. These are one-time observations, not a paired
performance comparison. No threshold changes.

### Native input continuity during recording transitions

The existing pause and microphone-clock selection paths pause the whole media
pipeline. That includes the selected PipeWire input, so the native consumer graph
must wake again before new video can arrive. The next candidate retains only a
Playing `pipewiresrc` through its parent's pause/clock transition. The existing
source-pad gate still discards every paused video buffer before it reaches
queues or encoding. Native microphone input remains parent-controlled and stops
before its clock boundary is saved. No permission or source identity changes.

An owned state hold is released after the parent returns to Playing, resynchronizing
its base time without disconnecting the live source. Early returns restore the
parent's control. Finish and Drop retire the hold before normal input shutdown,
so finishing while paused cannot restart a video input or lose the retained
microphone clock provider. Non-native verification sources retain their existing
state behavior. Accepted timestamps/durations, clock selection and media limits
are unchanged.

New real-element regressions cover a source kept Playing while its parent
pauses, restored base time, refusal to acquire another owner's lock, ordinary
source exclusion, early-return cleanup, retirement without restart and failure
when its parent disappears. The native PipeWire integration probe additionally
requires a connected Playing video input while the recording pipeline is paused,
a stopped microphone and no encoded frame timestamped after the saved pause boundary. Existing resumed-frame,
source-loss-while-paused, stopped-input and final-decoding checks remain.
CI, a fresh normal GNOME SDK package and another installed recording walk are
pending; this is a candidate correction, not installed quality acceptance.

The first native connection CI at `5afc4832` passes all 33 unit tests, the native
GTK walk and its normal package build, but fails the pinned provider probe's
new exact encoded-duration comparison during Pause. That comparison conflates
capture time and completion of already queued encoding. The next diagnostic
records the latest encoded input timestamp and rejects any timestamp after the
saved pause cutoff instead. It logs queued completion with before/after duration,
latest timestamp and cutoff; only a passing cutoff check can classify it as
pre-pause work. The live input/microphone state checks remain. The source gate
also rechecks Pause after looking up the clock, and admission closes before the
cutoff is sampled, so a callback cannot cross that lookup with stale admission.
No encoded timestamp/duration, media threshold or reference criterion changes.
The failed `5afc4832` bundle is preserved and has not been installed. Native
validation of this correction remains pending.

### Installed native-input continuity results

All five hosted checks pass at `06213b2e`. The normal GNOME SDK package from
job `128448737332`, run `42387780673`, artifact `13579334191` is installed:
bundle SHA-256 `ac0ba3a3aa33af1895fbefd9ed21d0c7b2571128da51a2008b018e5b0697e0ba`,
Flatpak commit `7b85f3d6e209fdbb3efbc6bd34f86ef3f830a34805fbd8bfc338b5c8eebe4e1e`.
The native fixture passes 33 unit tests and the pause timestamp cutoff check:
already queued encoding advances from 1.981356333 to 2.014848333 seconds, while
its latest frame timestamp 2.008880 remains before the 2.013074-second cutoff.
That is queued pre-pause work, not a new paused frame. Desktop source remains
`74c3fef4`; its relevant source trees are unchanged through `06213b2e`.

The first installed screen walk completes its controls and exact export but
fails media acceptance at 3.189 fps and a 765 ms uncovered gap. The microphone
walk stops automatically when its bounded input queue fills. The asynchronously
saved partial `1794067887-270965872-2` is separately retained in
`installed-native-connection-window-partial`, SHA-256
`4a93af4fb815ee32c6df56a8a6d35de93e3b08755710caa0f75df07dd12a8a69`.
It contains 27 distinct video frames and 20.600 seconds of continuous PCM; it
has no completed pause/export acceptance. The original failed collectors are
preserved. No result is substituted for their failures.

A repeat minimizes host activity and pauses only the competing Cargo/compiler
processes with the person's existing approval. No active test is stopped in
this repeat. The two processes resume at 16:39:28 UTC after an 16:30:58 UTC pause
on 7 November 2026; no watchdog fires. Both complete installed flows pass source
selection, stable five-second pause, microphone state, resume, finish, review,
naming, library and exact native export. Media still fails:

| Source | Clip | Decoded frames | Measured fps | Largest uncovered gap |
| --- | --- | ---: | ---: | ---: |
| Screen | `1794069136-28950846-2` | 83 | 4.217 | 450 ms |
| Window and microphone | `1794069404-279901285-2` | 115 | 3.337 | 779 ms |

Screen SHA-256 is `4331fb337533168bdda775a785a7b2843f863115dd9a410a1a0aa5f7b1133f04`;
window SHA-256 is `ceb5a2e9d92031ef9bdc637fcf09c7181bfc039da4be55a0f7d7140e6f214c5e`.
Both have strictly increasing timestamps and expected track counts. Window PCM
is continuous for 34.960 seconds; track endings differ by 35 ms. Eight of twenty
audio onsets match visual markers, median delay 598.5 ms, maximum 675.5 ms:
synchronization is not established. Evidence and summary are retained in
`installed-native-connection-quiet-boundary-*`. These are individual observations,
not a paired performance comparison. Thresholds remain unchanged.

A separate quiet-host reference observation runs without Studio recording after
the competing test exits naturally. Only its Cargo supervisor is paused; all
children finish with timers running. Reference PCM delivers 36.980 seconds over
38.055 seconds; nine visual rises and 22 audio onsets are observed. Audio delivery
versus its SystemClock timestamp drifts from approximately zero to one second.
Nearest-onset matches are diagnostic only: event correspondence and physical
presentation are not independently established. This cannot prove that either
the reference or Studio alone causes the recorded synchronization failure.
Evidence is `diagnostic-reference-quiet-phase-retry`; the supervisor resumes and
normal audio/source/chooser/installer cleanup passes at 16:43:35 UTC.

The earlier `installed-native-connection-quiet-repeat-validation` attempt briefly
stops and resumes an active test before its old guard refuses the run. It produces
no recording and is preserved separately. Subsequent pausers inspect children
before stopping them, or pause only the Cargo supervisor and let active children
finish naturally. `installed-reference-quiet-phase-validation` also preserves a
supervisor-stop acknowledgement race; its finally path resumes the supervisor.
Neither failed setup is recording acceptance.

### Share owned capture pixels without a second full-frame copy

Source inspection finds a full-frame allocation/copy at both direct capture
handoffs: `Arc::from(pixels.pixels())` duplicates storage already owned by scanout
or by the selected window render. Retaining the converted Vec behind an Arc
allows these handoffs to clone ownership instead. Existing slice access, pixel
format, scanout upload, permission checks, successful-submit gating, source
lifetimes and bounded publication/in-flight retention remain unchanged.
Redrawing creates fresh storage; previously published frames retain their bytes.

The public FrameTarget capture methods keep their signatures and behavior.
Additive shared-storage methods default to the existing backend entry points,
including an overridden capture-with-callback method; direct backends explicitly
opt into shared storage and complete only successfully captured trees. The
existing covered-window integration backend exercises the compatibility path.
A new readback regression checks pointer identity, shared mutation refusal,
redraw independence and retained bytes after the original frame is dropped.
Existing pixel and bounded slow-consumer tests remain. Formatting passes; hosted
validation and installed media acceptance are pending for this candidate.


### Installed shared-storage repeat and current rendering diagnosis

All five checks pass at `0a763631`. Desktop job `128466666633`, run `42393747201`,
artifact `13583300787` produces executable SHA-256
`93ced39f14eb6e7d9e37841f97ac98cf6e9a1b2bb59a521922dad7192078ce8b`;
archive SHA-256 is `5f3a534d657a2c04030d8aea9c9f6c83266bf9b12b08a74971634927ddd0ac18`.
It is installed on the same software VM, with normal app `06213b2e`. Exact normal
package input equivalence is retained separately. Both completed installed flows
pass controls and exact export, but fail media acceptance:

| Source | Clip | Frames | fps | Largest uncovered gap |
| --- | --- | ---: | ---: | ---: |
| Screen | `1794071792-622113966-2` | 82 | 4.153 | 615 ms |
| Window and microphone | `1794072103-749261708-2` | 74 | 1.985 | 1,806 ms |

Screen SHA-256 is `a84a144e8ef01937fe309ef9c60ace034c79ae2d50f728bfeafdec7418c99d70`;
window SHA-256 is `3ef36558eee5b0748f7a6143bed9e6b340a9986073c97e3738db6e76557eb66a`.
Both have strictly increasing video timestamps and expected track counts. The
37.700 seconds of PCM have no meaningful hole or overlap, with 111 ms between
track endings. The audio reference passes after one restart and again after idle.
Only two of 22 audio onsets match visual markers; median delay is 687.5 ms,
maximum 693.5 ms. No synchronization pass. All five paused compiler processes
resume at 17:24:37 UTC after the 17:15:15 UTC pause on 7 November 2026. Evidence is
`installed-shared-capture-*`; no paired speedup is claimed.

The following timing-preload recording is a diagnostic, not acceptance. Its
stage journal retains process identities, monotonic/real time, frame counts and
buffer timestamps. During the active interval before pause, publisher input is
2.073 buffers/s, app source 2.113, recording entry 2.112 and encoded output 2.211.
After resume these are 1.400, 1.387, 1.384 and 1.466 respectively. Across 145
publisher pool acquisitions, the maximum wait is 29.4 ms and median 0.35 ms;
observed publisher input queues are empty. The low rate already exists before
encoding. Encoder-output push measurements cover downstream muxing, not codec
CPU, and are not misreported as encoder execution time. Instrumentation can
change cost; these numbers locate stages, not normal media performance.
Evidence is `installed-shared-capture-stage-trace`. The exact native provider
is restored, the preload is removed from the app environment and all paused
compiler processes resume.

A separate 12.122-second sample begins only after confirmed recording. It
retains 252 compositor user instruction pointers with zero lost records:
124 in Pixman, 90 in the desktop executable and 37 in libc. Desktop symbols
include 27 native readback copies. The sampler closes its event descriptor and
mapping and its service becomes inactive. Evidence is
`diagnostic-shared-recording-cpu`, with diagnostic media in
`installed-recording-cpu-live`. Earlier setup failures (missing owned audio
device and a control timeout before confirmed recording) remain separate;
neither produces a valid CPU sample. The revised sampler uses a file signal
from the existing recording confirmation and sends no extra control requests.
Library thumbnail loading is already lazy and cancels when unmapped; inspection
does not establish background thumbnail decoding as the cause.

Source inspection confirms that software `copy_framebuffer` allocates another
Pixman image and composites the complete target into it before its bytes are
copied. The next candidate uses the existing safe `Image::from_slice_mut` API
with checked dimensions, exact stride and fully initialized owned u32 storage.
Only an exclusive temporary borrow reaches the engine. The image/target are
dropped before CPU readback, scratch reuse or publication, so subsequent drawing
cannot mutate a saved frame. The single output copy masks the unused channel
and emits canonical B,G,R,0 bytes. The shared painter and callback/submission
rules remain the same; GLES is untouched.

Scratch retention remains two sizes and 64 MiB, now counting actual Vec capacity.
Invalid dimensions fail before allocation. New checks exercise borrowed-image
layout and initialized bytes; existing real rendering, arrow, pixel, failed
render, retained snapshot and pool-budget tests remain, along with native-word
channel/row/length checks. No dependency, local unsafe code or renderer patch is
added. Formatting passes; CI and normally installed media acceptance are pending.


### Borrowed-word trial rejected by the renderer boundary

The combined candidate `76b9bdf5` passes native application and package checks
but its desktop and OS checks fail to compile. Pixman's FormatCode lacks direct
PartialEq, and, more substantially, Smithay's Bind implementation requires
`Image<'static, 'static>`. A borrowed-word image cannot satisfy that lifetime.
The complete trial is reverted while retaining main's newer changes and all
measurement evidence. The supported static owned-image/readback path is restored.
No leaked buffer, lifetime cast or local unsafe code is introduced. The direct
borrow is not an installed optimization or a demonstrated quality improvement.

After the recording sample, WSL has about 27 GiB available, all 8 GiB of swap
free, and QEMU reports zero swapped memory. This is a later observation rather
than a recording-time memory trace. It does not support swapping as the cause.
The sampler's journal proves event closure and an inactive service; normal
application/source/chooser/installer restoration is preserved alongside it.


### Readback page faults and the completed allocator experiment

All five hosted checks pass at supported source `7354c454`. The installed
executable remains the unchanged `0a763631` binary during these diagnostics.
A recording-time process snapshot measures the compositor over 16.154 seconds:
4.26 user CPU seconds and 9.94 system CPU seconds. Its main-thread scheduler
readings span roughly eleven seconds, with 6.55 seconds running and 4.27 seconds
waiting for CPU. Snapshot collection itself is diagnostic overhead; its media
is not acceptance. Evidence is `diagnostic-recording-activity` and
`installed-recording-activity`.

An idle compositor kernel sample under an approved compiler pause retains 799
instruction pointers over 12.245 seconds, with zero lost records. The largest
symbols are `do_user_addr_fault` (246) and `clear_page_erms` (193); `read_hpet`
appears only ten times. No clocksource or kernel boot parameter is changed.
A subsequent minor-fault sample is protected by the shared build lock and retains
794 records (one per 100 minor faults), zero lost, over 12.056 seconds. There
are 722 heap records, 71 anonymous records and one library record. Faulting code
is Pixman for 408 records and native `copy_xrgb` for 350: 758/794, about 95.5%.
This locates page initialization associated with the two full-frame allocations.
It does not imply swapping; QEMU previously reports zero swapped memory.
Evidence is `diagnostic-idle-kernel-clock-baseline` and
`diagnostic-current-anonymous-faults`.

An isolated Pixman comparison tries default, disabled wide-SIMD and disabled
SIMD implementations. Pixel outputs match, but timings do not establish a
consistent alternative advantage. The desktop implementation selection stays
unchanged. An isolated export/readback allocation comparison uses default versus
16 MiB mmap and 32 MiB trim thresholds. Across three trials, its 24-frame phase
produces 5,002-5,003 minor faults by default and 3,000-3,001 with retention; all
outputs match. Timings overlap, including a slower retained first trial. These
are microbenchmarks, not installed application acceptance. Evidence is
`diagnostic-pixman-native-comparison` and `diagnostic-frame-allocation-comparison`.
Threshold meanings follow the [GNU libc allocation tunables documentation](https://sourceware.org/glibc/manual/latest/html_node/Memory-Allocation-Tunables.html).

A separate reversible runtime-only compositor trial uses 64 MiB mmap threshold,
128 MiB trim threshold and two arenas. Its exact service override and before/after
process identity are preserved. After restart, another twelve-second fault sample
retains 319 records with zero lost and no mapping events. This is not a controlled
proof of speedup: process restart and cache state also differ. Observed RSS is
54,204 KiB versus 85,556 KiB in the older process, with no swap in either; it is
not a demonstrated steady-state memory reduction.

Under that explicit temporary configuration, both installed flows complete
controls, stable five-second pause, microphone state, review, library and exact
export. Screen clip `1794077022-350980926-2`, SHA-256
`d2c23b79636839322acedf95bea0c382a9152280c5603b4a8a84fa372e2d2105`,
has 99 decoded frames, 4.819 fps and a 373 ms uncovered gap: screen acceptance
passes. Window clip `1794077254-953451321-2`, SHA-256
`caaff89ef668f0071b6d86507e667af2b8584b8b5daecd658a602c5af5f7a225`,
has 108 distinct frames, 2.861 fps and a 1.033-second gap. Its 37.880 seconds of
PCM are continuous and track endings differ by 40 ms. The initial reference
passes but its idle recheck fails (10 seconds of PCM over 10.846 seconds). Four
marker matches do not establish sync. Window acceptance fails. Evidence is
`installed-heap-reuse-*`. All nine paused compiler processes resume at 18:51:01 UTC
after the 18:42:17 UTC pause on 7 November 2026. The owned override is removed,
and the new desktop process proves allocator defaults restored. No threshold or
accepted timestamp is changed, and allocator tuning is not shipped.

### Direct export into reusable owned readback bytes

The new candidate keeps the supported static Pixman render target. After the
shared painter finishes and the renderer borrow ends, Pixman composites directly
into an initialized owned byte vector through `Image::from_slice_mut`. This is
an ordinary image copy and does not require Smithay to bind a borrowed image.
The temporary image borrow ends before normalization or publication. The unused
channel is cleared in place, retaining the same allocation and canonical B,G,R,0
bytes. The intermediate exported image allocation and second full-frame copy
are removed.

The destination's alignment and length are checked by bytemuck's safe slice API.
Bytemuck 1.25.2 is already resolved in the workspace; it becomes an explicitly
pinned direct dependency with no new locked version. No local unsafe code, engine
patch, allocator environment setting, permission or timestamp change is added.
Malformed lengths, unaligned destinations and non-native source formats refuse
before the engine copy. Existing rendering/pixel tests remain, and a real-image
copy regression proves pointer retention and independent saved bytes.

The desktop optionally retains at most six readback allocations and 64 MiB of
actual capacity, separately from its existing two-image/64 MiB scratch pool.
Only a successfully unwrapped Arc can become mutable storage. A racing weak-reader
upgrade fails unwrapping and keeps that frame immutable; old weak references
cannot observe new pixels after successful unwrapping. Live readers are never
mutated. Byte, entry, oversized bypass and reader-lifetime regressions cover the
new pool. Full-image copy replaces all pixels before any publication. Existing
source lifetime, successful-submission gating and bounded transport remain.
Formatting passes; CI and default-configuration installed acceptance are pending.
