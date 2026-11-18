# Studio on the current operating system

The November 9 recorder branch predates the current application grants, native
file chooser, agent controls and shared drawing crate. Its recordings do not
verify an installation built from current main.

Integration takes place in `app/alo-studio-current`, review #1394, based on main
at `cdf0614d2ee56b69cbe154d14ad9594d3d198a88`. The original recorder branch,
review #1091 and its evidence remain preserved.

Studio keeps its existing capture sessions and semantic recording controls.
Export uses main's native SaveFile sheet and Documents portal. Settings uses
main's general application picker. The agent reads the current application
inventory after checking permission; a refused Studio action cannot trigger an
inventory read. Session grant views carry neither full access nor terminal
access. Grant changes, including sign-out cleanup, update the latest file under
its lock before notifying the daemon.

Native text caches belong in `alo-drawing`, where text rendering now lives.
They retain bounded storage and must preserve the existing pixel output and
font isolation tests. The drawing crate must keep its existing independence
from Wayland, Smithay and D-Bus.

## Validation is still in progress

The acceptance steps and media limits are in
[the Studio walk](../../walks/recording-a-demo-in-studio.md). Checks that rerun
on pull requests live in `.github/workflows/studio.yml`,
`.github/workflows/studio-package.yml` and the repository's current gates.
On November 18, the normal package and local signed repository checks passed
in hosted CI at `da827931ee32ecb256734ac8c053cb195ffe9e58`, including refusal
of a wrong signing key and altered repository metadata. Verification features
were disabled. Native capture plugin checks passed on that head too. These
checks run again in the package and native workflows.

Independent test review found that only two Studio operations crossed the
real-bus fixture and that the GUI check could pass with a broken pause handler.
The checks now cover all nineteen method/argument pairs, refusal after grant
revocation, the observed paused phase and a timer frozen for five seconds.
Independent request/action expectations also reject other valid Studio changes.
The four affected Rust packages passed 766 tests locally, with four
environment-dependent tests ignored; formatting, strict linting and
warnings-denied documentation passed. These checks rerun in CI. The strengthened
GUI recording check subsequently passed, as detailed below.

The real-model trial at that earlier head failed despite five repetitions at
each approval level: status, three/five-second countdown and finish requests
still took wrong roads. The initial follow-up preferred a matching application
operation before the accessibility fallback and clarified recording status,
countdown and automatic review descriptions. Its repeated result and the
narrower correction are below. Scripted tests do not replace trials or prove execution through the
installed agent.

At `2385dee33`, all eighteen Studio changes passed all five tries at both
approval levels. The status-only question still passed only two of five at full
access and none when asking each time. The full set also retained pre-existing
file-manager, chooser, scrolling and search-box failures; these are not new
passes or proven regressions. The broad application-verb preference has been
removed. A narrower, registry-derived status-read rule tells the model to read
live state before answering a state question. Its regression was observed red,
passed after the change and failed again when that rule was deliberately
disconnected. The restored implementation passed. The affected four packages
now pass 767 local tests, strict linting, formatting and warnings-denied docs.
The next real-model repetitions remain required.

The hosted mutation baseline also exposed an outdated Settings source check
that expected the old third grant path. Settings now uses main's shared
application-grant road. The source check again permits exactly two direct
Settings grant/revoke paths, as current main does. Independent review found a
raw application grant could hide behind a previously bound `Changing`; a new
negative fixture was seen failing and the guard now refuses that call in every
Settings source file. All six source-only checks passed when compiled directly
with Rust against the real shell source. Hosted Cargo/mutation validation still
needs to pass; no hosted mutation coverage is claimed from a failed baseline.

The private KVM image built and reached its OS boot walk. That walk did not
install Studio or establish recording acceptance. The separate private reference
window and real Pulse monitor first failed their local timing check; the
subsequent reference-only result is recorded below.
The actual application handlers, native consent and byte-identical export,
decoded video continuity, audio continuity and matched synchronization markers
still need installed evidence. A build or unit test pass does not establish
smooth video or audio synchronization.

## Private recording instruments

The private KVM boot walk at `73fbe941ec29485f6d3667e86e9737a1e1a39f51`
subsequently passed 64 checks, with none failed and 103 intentionally skipped.
It did not install Studio. This was one branch walk, not an installed-media gate.

The independently strengthened development GUI check passed: recording, a
five-second paused interval with identical elapsed time, resume, finalization
and library responsiveness with 128 previews. This check reruns in the native
Studio workflow; verification features and generated sources make it distinct
from installed OS consent or microphone acceptance.

Private walk tooling now includes bounded loopback file transfer, a separate
flashing reference application with a real Pulse monitor, and process-owned
native accessibility control selection. The reference first failed on real
audio gaps and clock drift. Using the monitor's actual sample clock for both
reference pipelines produced one healthy 35-second observation: 18 matched
markers, median absolute offset 114 ms, maximum 135 ms. This measured draw
callbacks and monitor samples once, not decoded Studio media or compositor
presentation. Its checker rejects missing channel boundaries, delivery gaps,
repeated timestamps, static markers and insufficient matches. Independent
tests caught a missing-leading-coverage fault before that checker was fixed.

The private Python suite includes five real GTK/AT-SPI fixture cases. The nine
Rust transfer/signal tests, all-target linting and documentation checks passed.
A broad debug walk test run was stopped during expensive
pre-existing pointer image searches; no full local walk-suite pass is claimed.
The reference example compiles and passes strict linting. The separate reference
Flatpak subsequently built in manual run 44310023010, source
`b92b185fdce5d7d149bd8b7c6ce9a0ba7d5cf074`. That run also passed the normal
package's signed-install, wrong-key and tampered-metadata checks. Its reference
artifact predates the guest controller archive and cannot run the new installed
walk. None of these helpers is a broker verb or an agent command surface.

The guest recording controller and private KVM wiring are now written. The host
requires an exact successful manual package run from the image's source commit,
checks both producer manifests and feature identities, and serves only closed
input names. The fresh test account installs normal Studio and the separate
reference, operates real native consent controls, records screen/window clips,
checks pause/resume and byte-identical export, and returns only named evidence.
The workflow states explicitly that this person-flow completion is not decoded
media or installed-agent acceptance. These new steps have not run on a guest.

Independent tests specify decoded-media limits before their implementation.
The real FFmpeg seam preserves nonzero timestamps, video/PCM gaps and overlaps,
partial sample tails, track counts and distinct marker/motion regions. Review
caught unfinished streaming WebM being accepted after its available frames
decoded cleanly; the regression was observed red and the auditor now also
requires finalized positive container duration. Seventeen decoder tests pass,
alongside 34 observation-audit and 22 package-preparation tests. Synthetic media
is test input only. Twelve deliberate faults in the auditor, decoder and artifact
checks were caught once locally; this is targeted mutation evidence, not a hosted
coverage result. The Rust walk/transfer/signal/workflow checks pass 25 tests,
strict linting and warnings-denied docs. The receiver and archive are private
test instruments. The installed screen/window recordings, decoded video/audio
acceptance, actual native consent/export and real-agent permission walk remain
unverified. Normal Studio continues to build with verification disabled.

The shared UI kit's application window, input, control and accessibility layers
remain unfinished prerequisites. Studio's GTK interface is a development test
surface. Neither the shared-kit migration nor public market availability is
complete. Local signed-repository verification remains the authorized
distribution scope. No unrelated verification VM is to be paused.
