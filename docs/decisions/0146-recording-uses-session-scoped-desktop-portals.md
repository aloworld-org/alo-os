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

Permanent checks must cover caller and session ownership, cancellation,
revocation, stale selections, unknown media activity, and refusal without a
record. A real private PipeWire server must prove both the selected stream's
availability and an unrelated stream's invisibility after descriptor handoff.

A private branch image walk must exercise the installed recorder, real native
source selection, recording with and without audio, pause and resume, playback,
sandboxed export, cancellation, revocation, and lock. Fixture recordings do not
establish that walk. Hardware microphone evidence is identified separately from
a virtual test device. This decision does not claim these checks have passed.
