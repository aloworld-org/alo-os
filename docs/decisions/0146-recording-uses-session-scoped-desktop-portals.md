# 0146 — Recording uses session-scoped desktop portals

Date: 2026-11-04

**Status:** Proposed

The owner asked for a separate native demo recorder with screen or window
selection, optional microphone audio, preview, and saving. Completing its OS
integration and walking it on a booted alo system are authorised. Market
publication is a separate step.

## Decision

`alo-portald` remains the public `org.freedesktop.portal.Desktop` frontend. Its
existing caller identification, grants, and answer record also govern the new
FileChooser and ScreenCast interfaces. Installing another public frontend would
give applications two different authorities and bypass the existing policy.

The native chooser and consent dialog are rented from the GTK portal backend.
The Documents portal exports exactly the selected file into the requesting
Flatpak sandbox. File selections become application grants in the existing
list; their document permissions are materialisations of those grants, and are
withdrawn when the grant is revoked. A dialog cancellation is recorded as a
person's cancellation, distinct from an unavailable backend or a policy refusal.
No broad home-directory access is added to the recorder.

The recorder exports its actual caller window through the standard XDG foreign
parent protocol while a native source dialog is open. The GTK picker imports
version 1 handles, while the pinned Smithay helper serves version 2. A bounded
version 1 adapter uses Smithay's existing surface roles and parent validation;
it introduces no renderer, input engine, or separate permission authority.
Its exports, imports and child relationships are bounded, and destruction
revokes their relationships. A newly mapped dialog may follow
an explicitly exported parent only while that parent holds the keyboard;
unknown, revoked and background-parent handles confer no focus. This window
relationship grants no screen or file access. Consent still passes through the
same authenticated portal request and central grants policy.

Each recording requires fresh source selection. There is no stored source,
restore token, or always-allow selection. The compositor publishes only the
selected source, and only to the authenticated portal service. Source handles
refer to actual output or surface lifetimes. Window capture renders that
window's surface tree and its popups separately; cropping the full desktop
would leak an overlapping window. Locking, signing out, source destruction,
caller disconnection, closing the session, or revoking its grant stops the
producer and closes the portal session.

The compositor never waits for a picker, bus round trip, media process, or
recording consumer in its frame loop. The catalogue, pending requests, active
streams, and latest-frame handoffs have explicit bounds. A slow consumer cannot
freeze the desktop or create an unbounded queue of private frames.

The media connection returned to an application sees only its selected stream
and the PipeWire objects needed to consume it. Other current and future nodes
are hidden, including cameras, microphones, and other applications' streams.
An unrestricted PipeWire socket is never a fallback.

## Narrow rented-binding extension

The upstream `pipewire` Rust crate at 0.10.0 lacks safe wrappers for
`pw_core_get_client` permission updates and `pw_core_steal_fd`, which the upstream
desktop portal uses to restrict and hand off its media connection. Before
patching that binding, this ADR permits vendoring the exact MIT-licensed crate
and adding only those wrappers. Its published crate archive SHA-256 is
`8585aba8a52ad74ccc633b8e293c1dc4277976bd5d510b925533f34fd6685f38`.

The permission method borrows a live core and checks the permission count. The
descriptor handoff consumes the uniquely owned core and returns an `OwnedFd`;
it cannot be called on a shared core, and existing borrows prevent consuming it
while listeners or registry proxies remain. The wrappers contain no alo policy
and do not alter PipeWire's engine or protocol. The workspace continues to
forbid unsafe code. The vendored changes, provenance, and removal condition
(upstream safe equivalents) must be documented beside the binding.

## Evidence required

### Native audio scheduling alongside the alo portal

The 2026-11-06 VM investigation found native PipeWire and PulseAudio processing
threads running without real-time priority. Their journals show
`org.freedesktop.DBus.Error.UnknownInterface` when the RT module asks the alo
portal for `org.freedesktop.portal.Realtime`. PipeWire 1.4 chooses a portal by
name ownership before checking that interface, so an owned but unsupported
interface prevents its native RTKit fallback.

Configure only the native `pipewire.conf` and `pipewire-pulse.conf` RT modules
to use the installed system RTKit broker directly (`rtportal.enabled = false`,
`rtkit.enabled = true`). Retain RTKit's authorization, priority bounds and
watchdog; do not grant scheduling capabilities to applications or agents and
do not copy or replace a complete upstream configuration. Keep sandbox/client
configuration unchanged. Verify actual service thread scheduling and real
audio delivery in the current VM before treating this as a capture-quality
fix. Remove the routing override when the public portal supplies a compatible
Realtime interface or upstream reliably falls back from an unsupported one.

### Capture publisher buffer-wait investigation

The 2026-11-06 software-VM trace observed an acquisition from the private
`GstPipeWirePool` begin without returning while compositor inputs continued and
the appsrc queue stayed at its two-frame bound. Thread inspection found the
publisher's appsrc thread waiting on a condition. The sink's `on_process`
signals that condition without the pool mutex, so a notification can be lost
between the empty-buffer check and the wait. Taking the mutex in that callback
would reverse the existing pool/loop lock order.

Evaluate a bounded timed recheck of that predicate, first in the private trial.
If it resolves the observed wait, carry a minimal, separately checksummed patch
against the pinned build copy and a regression that deliberately withholds a
notification after making a buffer available. Keep the imported engine source
unchanged. A timed recheck must preserve flushing, pause, cancellation and the
existing queue bounds; it must not manufacture frames or report a stalled
recording as continuous. The installed application flow and decoded media are
still the acceptance evidence. Remove the patch when an upstream version fixes
the notification protocol.

The booted PipeWire 1.4.11 publisher was observed to terminate with
`all buffers have been removed` during an isolated recorder lifecycle. Its
GStreamer sink treats the consumer's link removal as a fatal provider failure.
Upstream commit
[`ee2c6eb41ebab02c91faeb567cba86ece44dd996`](https://github.com/PipeWire/pipewire/commit/ee2c6eb41ebab02c91faeb567cba86ece44dd996)
changes buffer ownership so a provided source can survive link removal.
Before adding an application-specific recovery state machine, evaluate that
rented plugin in isolation. If adopted, pin its exact source and license,
package it privately for the capture worker, and retain the existing PipeWire
daemon and permission-scoped descriptor protocol. The provider's node identity
must survive pause/resume; closing or revoking its session must still remove it.
Remove the private copy when the OS's packaged plugin supplies the required
lifecycle behavior. An isolated plugin trial is not a booted acceptance pass.

The private copy also carries upstream's buffer-release race fix
[`2770143f506a92e242f8a5b8b870929b515b48c0`](https://github.com/PipeWire/pipewire/commit/2770143f506a92e242f8a5b8b870929b515b48c0).
It acquires the PipeWire loop lock before inspecting the buffer that the loop
may concurrently remove. Keep this as a separately checksummed upstream patch,
applied to a build copy; retain the imported source unchanged. The plugin from
1.6.9 was evaluated but requires SPA format definitions absent from the OS's
1.4 headers, so it is not a drop-in replacement for this pinned base.

Permanent checks must cover caller and session ownership, cancellation,
revocation, stale selections, unknown media activity, and refusal without a
record. A real private PipeWire server must prove both the selected stream's
availability and an unrelated stream's invisibility after descriptor handoff.

A private branch image walk must exercise the installed recorder, real native
source selection, recording with and without audio, pause and resume, playback,
sandboxed export, cancellation, revocation, and lock. Fixture recordings do not
establish that walk. Hardware microphone evidence is identified separately from
a virtual test device. This decision does not claim these checks have passed.

The current software VM's installed-app trace also locates long waits around
the VP8 encoding branch. A controlled comparison of the same 30-frame,
452 × 360, 5 fps sample takes 6.037/6.668 seconds with two encoder threads and
3.580/3.840 seconds with one. Evaluate a single VP8 encoder thread for the
Lightweight profile, retaining its size, frame-rate, bitrate and queue bounds.
Preflight must use that same configuration. This reduces coordination on a
small machine; it does not establish recording continuity or allow missing
audio to be concealed. Repeat installed recording and decoded-media acceptance.

The installed four-CPU trial has continuous audio but a video gap at Resume,
with the audio ending approximately that gap before video. GStreamer's pipeline
samples running time again after its children have paused; Studio has already
stopped admitting frames before that potentially slow transition. Freeze the
saved running time at that admission boundary and use the same request instant
for the visible timer. A real delayed GStreamer child must reproduce the added
time without the fix and keep it out of the saved timeline with the fix. Do not
fill missing audio or synthesize video to make this check pass.

An independent native microphone probe then delivers 48 seconds of uninterrupted
PCM over 48.755 seconds between first and last delivery. Delivery latency means
that interval alone does not establish clock-rate drift. For microphone
recordings, select the microphone's GStreamer clock before its first sample and
start a fresh recording timeline. Keep the selected PipeWire stream connected;
discard preview buffers from the preceding clock epoch before admitting video.
Silent recording retains the system clock. Verify clock selection and stale
preview rejection in permanent tests, then repeat the normally installed flow.
Do not stretch timestamps, fabricate samples or trim a failed recording to make
synchronization appear to pass.

The first microphone-clock trial exposes 179 ms of overlapping decoded audio at
Resume. When the microphone supplies the clock, first pause that input, then
snapshot the now-stopped clock and gate video. Taking the snapshot while the
microphone clock is still advancing moves retained samples backwards on Resume.
Keep the earlier request-boundary snapshot for a system-clock recording, where
native source shutdown would otherwise add time without capturing frames.

The timed-window investigation also finds that the separate window-capture
render never completes Wayland frame callbacks. Desktop presentation completes
callbacks only for its own submitted surfaces; a covered captured application
can therefore stop repainting while its old pixels keep being published.
After a successful selected-window capture, complete callbacks for its mapped
surface tree and its own popup trees, using that submitted frame's time. A
failed capture, an unrelated window and an unmapped surface must receive none.
Keep existing FrameTarget implementations compatible through an additive
default method. Verify actual Wayland callbacks, including failure and repeated
completion, before rebuilding the desktop and repeating installed recording.

The installed repeat also exposes a compositor-thread wait outside the capture
engine: the status area's speaker-volume reading runs `pw-dump` synchronously.
Wayland client traces show delayed frame callbacks, and a live compositor
snapshot waits without a deadline on the tool's output pipes. Move repeated
hardware readings to one outstanding worker with a bounded handoff. The local
clock and regional formatting remain on the frame path; slow, failed or stale
hardware readings become unknown rather than blocking frames or inventing
silence. A late hardware result must not overwrite a new clock format. Verify
that a blocked reader does not block refresh, that outstanding work stays
bounded, and that failure and stale results cannot retain measured values.
