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
GUI recording check subsequently passed recording, a five-second frozen paused
timer, resume, finalization and library responsiveness. It runs again in the
native Studio workflow, using development fixtures rather than installed OS
consent or microphone inputs.

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
At `1306978f7`, status passed four of five model tries at full access and all
five when asking each time. Enable-microphone and five-second countdown each
passed four of five at full access; three-second countdown passed two of five
when asking each time. Other Studio controls passed all five in both modes.
Generic failures also remained. These measured choices fail law 9 and do not
establish installed execution.

The next correction derives named-control advice from offered status reads and
their exact application families, retaining each declaration's purpose and the
generic fallback. Seven independent integration tests were observed failing
before the correction and pass afterward. Independent code review found no
blocker. Repeated model trials remain required.

At `bc768f471`, the four affected Rust packages passed 774 local tests, with
four environment-dependent tests ignored. Formatting, strict all-target
linting and warnings-denied docs passed. Five deliberate faults in the new
guidance (disconnected wiring, false status, mixed families, wrong effect and
detached purposes) each failed the independent tests; the restored seven tests
passed. These are targeted one-time mutation experiments, not a hosted
`cargo-mutants` coverage result. The integration tests rerun in the normal gate.

The hosted gate at `1306978f7` also caught eight purpose descriptions whose
vocabulary snapshot had not been reconciled. Independent ADR 0068 review
classified four as changed claims requiring new purpose keys and four as
rewordings retaining their keys. The implementation now declares those choices;
the official snapshot regeneration and all three vocabulary checks passed.
Unchanged approval, manual-action and change-description keys remain intact.
The two hosted gate failures were this snapshot check and the existing
permissions-directory removal test; a corrected full gate still must pass.
The hosted image check, optimized desktop checks and normal signed-package
checks passed on that head. Native and mutation jobs timed out downloading
dependencies before their tests. They supply no mutation coverage.

The hosted mutation baseline also exposed an outdated Settings source check
that expected the old third grant path. Settings now uses main's shared
application-grant road. The source check again permits exactly two direct
Settings grant/revoke paths, as current main does. Independent review found a
raw application grant could hide behind a previously bound `Changing`; a new
negative fixture was seen failing and the guard now refuses that call in every
Settings source file. All six source-only checks passed when compiled directly
with Rust against the real shell source. Hosted Cargo/mutation validation still
needs to pass; no hosted mutation coverage is claimed from a failed baseline.

The private KVM image built and reached its OS boot walk. That walk does not
install Studio or establish recording acceptance. A separate private reference
window and real Pulse monitor subsequently passed one reference-only observation
with 18 marker pairs, median absolute offset 114 ms and maximum 135 ms. Those
are draw callbacks and actual monitor samples, not decoded Studio output or
compositor presentation.
The actual application handlers, native consent and byte-identical export,
decoded video continuity, audio continuity and matched synchronization markers
still need installed evidence. A build or unit test pass does not establish
smooth video or audio synchronization. Private draft #1397 adds the controller
and bounded artifact transfer. At frozen source
`76f131ecea607e317fa141004655f0d224107a32`, manual package run 44320429214
passed normal packaging, signature refusals and separate reference/controller
packaging. Installed KVM walk 44322063965 was dispatched with that exact source
and package run. It later failed before installation: the terminal's Window
actions band consumed the setup command. No Studio recording was returned.
Private source `83e0fa210c03a4d45c3b3eb00f35ecd41894779a` corrects focus and
requires a fresh command acknowledgement before installing. Its matching
package run 44378112596 passed. Installed walk 44380155898 built and booted the
matching image, accepted the terminal command and installed both packages, then
failed during controller startup before recording. The saved report contains
deployment provenance but no startup exception. Recording acceptance remains
unverified. The private
instrument workflow also passed its real decoder/native-control checks on that
source; local validation recorded 143 Python and 30 focused Rust test passes.
The person-side controller is not an agent tool and cannot establish the
separate installed grant/discovery/approval/revocation acceptance.

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
is test input only. Eighteen deliberate faults in the auditor, decoder, artifact
and controller checks and six walk/provenance faults were caught once locally;
this is targeted mutation evidence, not a hosted coverage result. The Rust
walk/transfer/signal/workflow checks pass 30 tests,
strict linting and warnings-denied docs. The receiver and archive are private
test instruments. The installed screen/window recordings, decoded video/audio
acceptance, actual native consent/export and real-agent permission walk remain
unverified. Normal Studio continues to build with verification disabled.

Independent pre-run review also caught the real Flatpak CLI refusing two local
bundles in one install invocation, skipped cleanup after the first error, and
empty XDG state paths bypassing preference history. Test-first corrections use
two checked installs, attempt every owned resource before propagating cleanup
failures, and restore automatic microphone selection only after a verified fresh
profile was temporarily overridden. Both persisted WirePlumber state filenames
are examined. An owned local PipeWire session confirmed source-only clearing
preserves configured sink/video choices; it did not change an existing session.
The complete private Python suite passed 143 tests with all five live native
tests enabled, including the final four cleanup regressions. CI now reruns this suite with actual
decoder and native transports, explicit dependency preflight and no skip gate.

Pre-run trace: `.github/workflows/walk.yml:104` validates the exact package run;
`:134` calls `prepare_installed_walk.py:49`, which verifies both original
inventories before creating fresh input copies. The scoped walk starts at
`crates/alo-walking/src/the_first_walk.rs:155`; `studio_installed_walk.rs:22`
checks guest bytes before the installs and controller, and `:29` waits for the
actual journaled exit status. `installed_session.py` calls the app's real D-Bus
interface, whose `apps/alo-studio/src/ui/control_bus.rs:11` registration reaches
`control_actions.rs:35`. Native source/export consent is still operated by the
person-side test. `crates/alo-walking/src/studio_walk_files.rs:95` accepts only
closed evidence names. The host's `decode_media.py:99` and
`recording_health.py:53` audit decoded observations; `reference_health.py` must
separately pass before synchronization is certified. No guest run has yet
completed this chain.

The shared UI kit's application window, input, control and accessibility layers
remain unfinished prerequisites. Studio's GTK interface is a development test
surface. Neither the shared-kit migration nor public market availability is
complete. Local signed-repository verification remains the authorized
distribution scope. No unrelated verification VM is to be paused.

## Terminal focus failure found by the installed walk

Private KVM run 44322063965, source `76f131ecea607e317fa141004655f0d224107a32`,
built and booted the image, but the installed-flow completion check timed out.
The retained terminal picture shows Window actions focused after maximising;
the serial log records repeated window-placement actions while the setup
command was being typed. No Studio session report or recording was returned.
This is evidence of a pre-installation focus failure, not a recording verdict.

The private branch now includes integration source `6656d0c91`. Its hosted
repository gates and native/desktop checks passed. All 19 Studio model requests
passed five repetitions at both approval levels (190 Studio tries), while the
full model job failed on other requests. In particular, two close-application
tries unnecessarily requested Studio status; the broader model gate remains
unresolved. The hosted mutation job also found surviving changes and did not
complete its entire candidate set. Neither job is reported as a pass.

The private walk now presses Escape to leave the terminal band, submits a
harmless journal acknowledgement, and requires that fresh acknowledgement
within 30 seconds before typing the installation command. Three independent
Rust tests failed against the old steps and passed after the correction,
including missing and stale acknowledgements that must stop before installing.
These checks run through the normal repository gate. A new exact-source package
and installed KVM walk are still required; installed media and agent permission
acceptance remain unverified.
Focused walk validation passed 33 tests, formatting, strict all-target linting
and warnings-denied documentation. Five deliberate faults (missing Escape,
stale-log acceptance, a 20-minute preflight, missing Return and the wrong
acknowledgement) each failed a regression test; the restored tests passed.
These five mutation checks were performed once locally, not by the hosted job.

## Matching package and private agent probe

The corrected private source `83e0fa210c03a4d45c3b3eb00f35ecd41894779a`
produced a successful package in run 44378112596. Installed KVM run 44380155898
used that exact source and package. It built and booted the image, accepted
the terminal command, installed both packages and saved their exact deployment
identities. The controller then exited 1 during startup, before its first
recording/status event. The saved report contained only provenance, so the
startup cause cannot be established from this run. No recording was produced.

The media-verification branch now includes the current parent and a separate
fixed agent probe. Its independent tests were observed failing before each
implementation. The probe checks actual Linux identity, trusted machine-file
metadata and content, exact discovery, refusal, and state across five native
permission boundaries on one connection. Initial connection establishment is
bounded, and a cancellation watchdog closes that same connection at the shared
deadline even when a daemon trickles an incomplete reply. It cannot select
another socket, impersonate another identity, grant access or approve an act.

All 37 focused Rust tests, formatting, strict all-target linting and
warnings-denied documentation passed locally. This root-owned test lane
actually exercised the root-file fixtures and root-process refusal; hosted
ordinary-user tests explicitly skip fixture observations requiring root.
Thirty-nine deliberate faults across the probe checks were each caught once
locally, with the original sources restored. Normal repository tests rerun the
regressions; those targeted mutation experiments are not a recurring hosted
measurement. Independent read-only review found no remaining blocker within
the implemented probe scope.

The controller now receives the same-source probe as a separate checksummed
input. It requests native Settings, stages the executable in a fresh person-owned
directory, authenticates the disposable account through sudo's input, and drops
to the agent identity with no capabilities, supplementary groups or future
privilege elevation before launching it. Native choices are read back before
acknowledging the five boundaries. The child is reaped before restoring policy;
an unreaped child prevents restoration of full trust. No public broker verb or
normal application package gains this test instrument.

Independent tests caught three cleanup gaps: a failed terminate or wait skipped
the kill/reap attempt, and a failed selector close skipped native restoration.
A further regression requires leaving the restrictive policy in place when
the child still runs. Local tests exercise real pipes/files, native-control
fixtures, refusal paths and independent cleanup; they do not certify sudo's
descendant signal path on the installed image. Native Settings read/controls
also cannot distinguish an external replacement folder with Studio's display
name, so this instrument requires an exclusive fresh disposable profile.

Startup and cleanup exceptions now enter the uploaded report with their stage,
type and a bounded message. An early install/controller failure also fails the
host's Settings wait immediately. Actual native approval, exactly-one execution,
execution/refusal records and installed permission withdrawal remain unverified.
These local checks do not complete installed recording acceptance.

The integrated private instruments pass 200 Python tests, including real local
process pipes, isolated GTK/AT-SPI fixtures and real decoder checks. The delivery
changes pass 18 focused Rust tests, formatting and strict all-target linting.
Twenty-eight deliberate faults in the new Python helpers and evidence paths,
four delivery faults and eight workflow-ordering variations were caught; the
original sources were restored. These are local instrument checks, not a claim
that the installed recording or native permission flow passed.

## Preserve the requested application and operation

Real-model run 44378095749 at source `5ce2ca8a6` passed 189 of 190 Studio
tries. One request for a five-second countdown proposed a quality change
instead; an unrelated slider request also read Studio status. Both are genuine
failures. Other generic application cases failed too, so the full repeated-model
gate remains failed. The earlier 190-of-190 Studio result does not supersede
these later observations.

The generic application guidance now scopes a status read to the application
the person requested, preserves fallback for applications with no offered
status read, and keeps the requested operation unchanged after reading state.
Independent instruction tests with two synthetic application families failed
before the correction and passed afterward. All 210 local instruction and
evaluation tests passed, alongside formatting, strict linting and documentation;
the pre-existing weights-dependent language test remained ignored. Five
one-time deliberate removals were caught by the independent regression tests.
No model request, expected operation, repetition count or acceptance threshold
was weakened. The unchanged repeated-model gate must still establish whether
the correction improves actual choices.

## Finish the requested operation without unrelated status reads

At source `27f9a7d0f`, real-model run 44383621155 passed all 190 Studio tries.
The overall gate still failed: Ask's fourth open-Firefox try called Studio status
after the requested open completed. Existing generic file selection/opening,
private typing and chooser cases also failed. This is not an all-model pass.
The source's native capture job passed, including real framebuffer and pinned
publisher record/pause/resume/decode checks; installed recording remains separate.

The named-control guidance now says to answer when a generic application
operation completes the person's whole request, without appending a named
status read. Remaining requested steps still continue. Independent review
caught and corrected an overbroad earlier wording before it was committed.
The new instruction regression failed before both corrections, then passed
with the complete instruction/evaluation suite, formatting, strict linting and
warnings-denied documentation. Three one-time scope removals were caught.
The real-model requests, expected actions and repetition counts remain unchanged;
the next real-model result must validate the new guidance.

## Fresh audio startup observations

The matching private package at `b03735c80` passed in run 44389702513. Installed
KVM run 44390952063 uses that exact source and package; its result is pending.
The earlier startup failure at `83e0fa210` still has no retained exception, so
the observation below does not establish that guest's failure cause.

A separate one-time observation used the existing development container with
PipeWire 1.6.11 and WirePlumber 0.5.19, a fresh private D-Bus session and fresh
audio state, without hardware or an existing person's session. `pw-cli` listed
the default metadata store, but `pw-dump` omitted it until an owned loopback
node supplied automatic source/sink entries. The old controller refused this
valid empty starting state. The corrected controller reached the application
launch boundary using real audio services; the observation stopped before
launching Studio, cleared its temporary source preference and reaped its audio
processes. This was a one-time diagnostic, not an installed recording gate.

The controller now validates persisted history and any existing live preference
before creating its fixture, then requires valid live metadata before choosing
the microphone or launching Studio. It checks each new snapshot, so a configured
source appearing after the first observation also refuses. Only absent metadata
may retry. Independent review found malformed metadata being treated as absence;
its regressions failed before the properties check was added. Failure still
cleans up the owned fixture without clearing a preference it never changed.

The complete Python instrument suite passes 209 tests, including nine new
independent startup tests and the real decoder/native-control fixtures. Ten
deliberate startup faults were caught once in temporary source copies. The
regressions rerun in the verification-instruments CI job; these local results
do not establish installed startup, recording continuity or synchronization.

At integration source `11ee0062a`, model run 44389552194 passed all 190 Studio
tries. The complete gate still failed: one Ask close-application request
appended Studio status, and generic archive, scroll, selection, file opening,
private typing and file chooser cases also failed. The completed-request wording
has therefore not eliminated unrelated status reads. No request, expected road
or repetition count was weakened, and the failed gate is not reported as passed.

## Native approval evidence preparation

The private approval instruments now validate bounded appended execution
records, observe recorder D-Bus calls and successful replies, and observe one
fresh pending Ask process through its matching successful exit. Independent
test-first review caught an interface-omitted duplicate call escaping the
delivery counter and normal systemd invalidations rejecting valid exits. Both
regressions now pass. Thirty lifecycle tests use a typed private D-Bus fixture;
a separate one-time systemd observation established the actual notification
order through garbage collection. Thirteen deliberately introduced lifecycle
faults and the duplicate-call regression fault were caught in temporary copies.
Earlier record-reader and delivery checks caught twenty and eleven faults.

The 298-test instrument suite passed before the Settings locator follow-up.
The expanded 299-test run later passed 298 tests and hit one two-second child
startup timeout in the FIFO refusal test; that test passed independently in
0.156 seconds without a code change. The corrected Settings recovery branch
passed all 210 tests. These are local observations, not installed acceptance.
The standing verification-instruments job discovers all these regressions.

The new observers are not yet connected to the installed controller. Process
exit is not model success, and a record alone is not one delivered action.
The completion observer captures a pending Ask turn, not a fast Full turn.
Native approval, policy withdrawal, actual app state, matching execution record
and single delivery must still be exercised together after provider setup.
## Decide completion at the end of each continued prompt

Real-model run 44389552194 at `11ee0062ad773aa36c18ed5f89f9d0a2f1ebf9c2`
passed all 190 Studio tries but failed the complete gate. One Ask close-Firefox
try appended `studio.status` after Firefox had been asked to close. Generic
archive, scrolling, selection, file-opening, private-typing and chooser cases
also failed. The native recording and desktop jobs, image check and nine hosted
gates passed on that revision; hosted mutation coverage remains pending.

The final continuation instruction now compares the whole request with the
results and answers if it is complete before considering more work. Further
reads must serve a remaining requested step; application reads stay within that
step's application. Nonapplication reads remain available, and completed
changes must not be proposed again. Independent tests capture the actual
continued prompt through the agent loop at both approval policies, including
their distinct completion reply forms. They preserve the genuine close-Firefox
fixture, a two-step close/open request, early-answer refusal and rejection of an
unrequested status read. The completion-priority test failed before the change.
These are recurring prompt and scripted-loop checks; they do not prove what a
real model will choose. The real-model cases, expected actions and repetition
counts remain unchanged, and a new repeated-model result is still required.

The local instruction, agent-loop and evaluation suites, formatting, strict
linting and warnings-denied documentation passed. Nine one-time deliberate
faults were caught, including removing completion or remaining-work scope,
moving completion behind further reads, and allowing a completed change again.
The tested production text was restored afterward. The pre-existing language
test requiring local model weights remains ignored. Installed recording and
native agent approvals remain separate acceptance requirements. Private child
#1399 fixes an independently reproduced empty-audio-metadata startup refusal;
its signed-package run 44393682212 passed and exact-source installed walk
44394653212 is running. Neither a package pass nor a local audio observation
establishes installed video continuity or audio synchronization.
