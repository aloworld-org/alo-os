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
still took wrong roads. The model is now told to prefer a matching application
operation before the accessibility fallback; recording status, countdown and
automatic review descriptions are clearer. This change requires a new repeated
trial. Scripted tests do not replace that trial or prove execution through the
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

The shared UI kit's application window, input, control and accessibility layers
remain unfinished prerequisites. Studio's GTK interface is a development test
surface. Neither the shared-kit migration nor public market availability is
complete. Local signed-repository verification remains the authorized
distribution scope. No unrelated verification VM is to be paused.

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
