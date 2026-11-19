# Studio recording acceptance candidate
The retained diagnostic run 44493013653 passed 189 of 190 Studio trials. In
Ask each time, screen-picker attempt one selected `open_application` although
the request said Studio was open and asked for its system picker without
starting capture. Its complete first question and both policy transcripts
were retained; artifact SHA-256 is
`49b9551342a43e165842d26b7f1273e3a65afba04e92fb8e05a07247e6a0496c`.
This is another observed model-selection failure, not a fabricated fixture.

The screen-picker purpose now explicitly describes opening Studio's system
picker for the person to choose a screen, without starting recording. A new
translation key preserves ADR 0068. The action name, arguments, ChooseScreen
D-Bus method, consent boundary, fixed trial request and expected road are
unchanged. Independent review traced the actual prompt and adapter-to-UI
handover and found no blocker. Local checks passed three vocabulary tests,
198 adapter tests and eleven named-control/status guidance tests, formatting,
strict all-target lint and warnings-denied adapter documentation. A one-time
fault changing ChooseScreen to ChooseWindow failed the independent real-bus
contract test at the exact method assertion; restored source passed. That
fault proves method discrimination, not deterministic model selection.
The recurring model suite must now evaluate the revised purpose against all
fixed requests, ten attempts per policy; results remain pending. The original
installed candidate and its active walk remain undisturbed.

The recording candidate `1b26cea87135d7b783100ed69996afdf3119c631` passed
signed package run 44491774129 and all 416 hosted verification-instrument
tests plus nineteen CI-helper checks. Its installed walk 44492469477 is
pending. A separate CI-only diagnostic branch retains the existing real-model
input transcript, separately for Full and Ask, and uploads exactly those two
files even after failure. This does not change model instructions, requests,
trial counts, provider settings or application code. It does not fix the
188/190 model result described below.

Five independently written tests execute the real workflow shell with a
harmless trial stand-in. Before the change, three methods failed because the
transcript option and upload were absent; two passed. After correction, all
twenty-four CI-helper checks pass. Five one-time faults were caught: omitted
transcript output, combined policy files, masked trial failure, upload only
after success and uploading a workspace glob. Independent final review found
no blocker and confirmed both test files match the independent originals.
These regressions recur in Studio's CI-helper discovery job. The synthetic
shell checks do not measure model behavior; the new transcript will accompany
future real-model trials. The existing provider writer stores only the fixed
test question and continuation text; credentials follow its separate network
transport and are not written into the artifact.

On source `5b8f686b426e8e812bb316af088ffd308a992b2b`, signed package
44477809005 passed. The focused window-picker trial 44477814824 passed all
twenty attempts, ten under each access policy. Full repeated model run
44477779244 passed all 190 Studio trials: nineteen controls, five attempts
under each policy. Generic requests outside Studio kept the overall job red;
Studio's result does not close those failures. No model action in these trials
was carried out on a machine.

All three Studio jobs in 44477779235 passed: 393 private verification tests and
fifteen CI helper tests, native recording and decoding checks, and optimized
desktop regressions, strict checks and service compilation. These checks do not
replace installed acceptance. Image check 44477779237 passed. Mutation run
44477779261 tried thirty faults: twenty caught, ten unviable and none missed.
Installed walk 44478570107 passed all five initial permission stages, both
source-picker cancellations, screen recording with pause/resume, export
cancellation without a crash, exact-byte saving and library selection by ID.
It then failed selecting the window source: the verifier expected “Window”
while the native picker publishes “Application window.” Retained evidence ZIP
SHA-256 is `f51a9c76f5436506e2b2cab42e72bb691658c60ff27e5d42cab19b062d866357`.
The saved screen video has 123 frames over 24.652 seconds (4.989 fps), strictly
increasing timestamps, a maximum 241 ms timestamp step and 41 ms uncovered gap.
Settings obscures the moving reference in inspected frames, so visual
continuity remains unverified. Window capture, microphone synchronization and
the subsequent native approval walkthrough were not reached. These are
one-time observations of retained media, not a completed acceptance verdict.

The private verifier now selects its exact reference title after the portal's
source-kind separator. The portal keeps ownership of its translated kind label.
Five independent tests exercise the actual Session picker through native GTK
radio controls, the session-bus owner lookup and AT-SPI activation. The original
verifier failed the positive selection test; the correction passes all five,
including ambiguity, foreign-owner, hidden/disabled and cancellation refusal
paths. Three one-time faults were caught: the old kind label, selecting any
title, and omitting Share. The fixture publishes its response atomically after
one fault run exposed a test-only partial-JSON read race.

The reference is brought forward by its native Dock action before each Start,
including the cancelled countdown. The ordinary application-focus door alone
does not reorder Settings above application windows; the person's Dock road
does. The verifier waits for fresh reference activation and refuses while a
showing shell surface remains active. This setup check does not replace visual
inspection or decoded moving-region evidence. The new helper is included in
the private session archive, not the application's public controls.

A separate local GTK 4 experiment caught a naming assumption in that setup:
the accessibility root uses the program name, while the frame uses the window
title. GTK 4.20.4 source confirms that distinction and the registered
GApplication object path used for accessible children. The helper identifies
the reference through that path and requires exactly one owned frame with
the expected title, a frame/window role and active/showing/visible states.
It presses the Dock once and then reads fresh state; its polling deadline is
checked between bounded tree scans, not a hard ten-second wall-clock limit.
The ordinary Studio warning selector also uses the application's explicitly
set program name instead of its display label.

Independent tests first failed the missing foreground-before-Start condition,
archive omission, program-name identity, wrong application path, wrong role
and warning-owner selection. The completed local suite passes 416 checks,
including five live GTK picker tests, fifteen foreground tests and three
warning tests. Eight additional one-time faults were caught: repeated Dock
activation, ignored covering Settings, ignored activation states, omitted
foreground setup, omitted archive helper, accepted foreign application path,
accepted label role and the old warning-owner name. Together with the three
picker faults, all eleven were caught. The regressions recur in Studio's
private-verification discovery step. The GTK observation and deliberate faults
were one-time checks, not installed Wayland or recording acceptance.
Independent final review found no concrete blocker and confirmed that all six
integrated test/fixture files match the independently prepared sources.

On the preceding CI-only source `377ac0d91e5865d679f75edb2a7612c8013c4e8e`,
Studio run 44487234768 and package run 44487234877 passed. Repeated model run
44487234777 passed 188/190 Studio trials: Full-mode status try one selected a
file search, and Ask-mode five-second countdown try two selected recording
quality. These new failures remain open despite the preceding 190/190 result.
Mutation run 44487234780 timed out installing dependencies before mutation
testing, so it supplies no mutation result.

Repository gate 44477779266 failed before workspace tests in both the first
attempt and one infrastructure retry. The initial download exhausted its
90-second bound. Fallback rewrote the runner's mirror-list URI to HTTP and
exhausted the 60-second index-fetch bound. The correction reuses the existing
isolated Python archive helper to retain HTTPS, source suites, signing fields,
direct security endpoints and unrelated repositories. Existing fetch limits,
DNS fallback and installation from checked downloads remain in place. Hosted
gate 44487235191 on correction `377ac0d91e5865d679f75edb2a7612c8013c4e8e`
completed dependency setup and reached workspace checks; its final outcome is
pending. One successful setup does not establish download reliability.

Four independently authored tests execute the dependency-step shell with
temporary APT files and intercepted commands. The original workflow passed three
and failed the HTTPS source-byte assertion. The correction passes all four;
with the existing archive/helper wiring tests, nineteen checks pass. Three
one-time deliberate faults were caught: HTTP from the helper, ignored retry
failure, and installation without `--no-download`. All four tests passed after
each restoration. Independent review found no blocker. These regressions recur
in Studio's CI-helper discovery step; the deliberate faults were one-time
checks. They do not certify network performance or real package signatures.

On source `59dfbf8809b05d53da333f5876c6a4a74e9938a8`, signed package
44454878392 passed, but installed walk 44455470524 stopped at native grant
withdrawal: the native command reported that **Take back** was unavailable. It passed 32
checks, failed one and did not reach 158, including recording and export.
Retained evidence has SHA-256
`20a93b53590c2962d42226dc273bd9a5670af1ceaba26cbbe47184504a436690`.
Studio component verification 44454872190 and image build 44454872185 passed.
Repository gates 44454872182 were accepted under the declared runner exception:
16,157 passed, one declared permission-test failure and 101 skipped. Mutation
run 44454872170 caught sixteen of twenty-five trials; nine were unviable.

A grant's decreasing remaining duration was included in its accessibility
action. Whole-tree equality therefore detected a change every frame and removed
and republished unchanged controls. Independent tests reproduced that churn with
only one nanosecond elapsed. Grant actions now carry the stable grant ID and
resolve it among the current page's live rows when pressed, before following
the existing deferred selection path. Visible labels, descriptions, states and
attributes still participate in publication equality. Expiry and replacement
still change the tree. This removes duration-only churn; it does not make
structural tree replacement atomic or prove precisely which D-Bus read failed
in the installed walk.
The repaired baseline had six passing tests and two failures for duration-only
publication. All eight independent real-bus tests passed after correction.
Four deliberate faults were caught: discarded grant identity, duration in the
action, accepting any live grant and refusing the selected live grant. Restored
sources passed all eight new tests and 96 surrounding Settings checks (104
total). The existing every-page projection test was observed failing on its old
ARow expectation, then adapted to the new grant action without weakening its
every-row coverage. Formatting, strict library/test lint and warnings-denied
shell documentation passed; the five existing Smithay dependency warnings
remain visible. Independent final review found no material blocker and verified
that the candidate source and tests match the reviewed copies. Tests recur in
repository gates; deliberate faults were one-time experiments.

The same source's real-model run 44454872256 passed Studio 189/190: one Full-mode
window-picker request proposed opening the already-open application. The window
picker purpose now explicitly says it opens Studio's system picker without
starting recording, under a new translation key as ADR 0068 requires. The fixed
request, expected actions and repeated trial counts remain unchanged. Wording
ambiguity is a hypothesis; corrected real-model trials are required, and the
previous source's 190/190 result does not close this failure.
The regenerated vocabulary changes only that key and sentence. All three
vocabulary checks and all 198 adapter tests pass, as do adapter formatting,
strict all-target lint and warnings-denied documentation. These local checks
do not measure real-model selection; the matching-source repeated trials and
installed recording walk remain required.

Cancelling Studio's export chooser now receives an explicit empty URI list
from the FileChooser portal. GTK reads that list before checking cancellation
or refusal; alo previously omitted it when no file was handed over. The common
response boundary now supplies a typed empty string array only for FileChooser
replies lacking the field, preserving successful addresses, response codes,
records and other interfaces. Independent tests first failed in eight no-file
cases, then all thirteen passed after correction. Independent review found no
concrete blocker. The [GTK compatibility note](../../quirks/gtk-file-chooser-reads-uris-before-the-response-code.md)
distinguishes the installed GTK 4.20.4 crash from a one-time GTK 4.14.5 consumer
experiment, where the corrected reply returned cancellation successfully.
Five deliberate faults were caught by the intended assertions: omitted URI
list, wrong list type, overwritten successful URI, refusal sent as success,
and a changed reply map on another portal interface. Restored sources passed
thirteen FileChooser and eight OpenURI tests, followed by the entire portal
suite: 168 tests, including one documentation test. Formatting, strict
all-target portal lint and warnings-denied documentation checks passed. These
regressions recur in repository gates; the deliberate faults were one-time
experiments. Independent final diff review found no code blocker. The corrected
installed recording flow remains pending, including actual document writes,
window/microphone media and native Ask execution. The current library check
proves reopening by ID, not playback. FileChooser Request.Close remains a
separate platform gap; this walk uses native Cancel and Save.

Further boundary review reproduced an expiry race between selecting a Settings
row and executing its deferred Choose. The pointer and reader paths select at
one clock reading; `a_key_reaches_settings.rs` supplies a later reading to
`SettingsWindow::pressed`. When a grant expires between them, the live row list
shrinks and its old index can name a different grant or action. Three independent
tests first failed by revoking a different grant, opening the folder picker from
an expired grant, and missing a selected live grant after an earlier one expired.
This makes an existing promise true: Settings must act only on the control the
person selected. A further independent case reproduced the same problem for
Give alo a folder, which became OpeningTheRecord after a preceding grant expired.
Settings now retains the selected row until its deferred Choose and resolves
that action against current rows. Grants match by non-reusable ID; other rows
match their action. An absent target does nothing. Every new targeting event and
every key press clears the pending target, preserving normal keyboard navigation
and subsequent choices. All thirteen focused tests pass, including persisted
grant state, folder-picker behavior, repointing and consumed-target checks.
Independent source review found no concrete blocker. Four deliberate faults
were caught: discarded selection, fallback to the old index, retaining a
consumed selection and retaining grants but discarding other selected actions.
Restored sources passed all thirteen focused tests, seven reader tests, 57
Settings-window tests, five keyboard tests and six pointer tests (88 total).
Formatting, strict library/test lint and warnings-denied shell documentation
checks passed; the five existing Smithay dependency warnings remain visible.
These tests recur in repository gates; the deliberate faults were one-time
experiments. One experiment was interrupted during compilation, then resumed
from the verified partial source with its log retained. These local results
do not establish installed recording acceptance.

On source `f5f0f0ea6d382ee80b7e86c71de134ee6f679a9b`, signed package run
44447595166 passed. Studio verification 44447583077 passed native capture,
desktop regressions, 393 instrument tests and fifteen CI helper tests. Image
build 44447583206 passed. Real-model run 44447583007 passed Studio 190/190 and
close-application 10/10, but failed generic file, slider, private-typing and
chooser cases. Mutation run 44447583040 tried fourteen faults: eight caught,
six unviable and none survived. These results do not cover the subsequent expiry
correction or close earlier broad mutation gaps. Repository gate 44447583129 was
accepted under its declared runner exception: 16,147 tests passed (three leaky),
one declared permission test failed and 101 were skipped. Formatting, strict
lint, documentation, supervisor and BPF gates passed. Installed run 44448153799
passed all five native permission stages, including withdrawal, source-picker
cancellation and screen recording through pause/resume and review. It then
crashed while cancelling the first export chooser, before saving. The session's
`native-export-destination` event records the proposed destination, not a saved
file. GTK 4.20.4 terminated with SIGSEGV; the controller subsequently reported
ServiceUnknown. No media was uploaded before that failure, so decoded
continuity and synchronization remain pending. Its retained artifact has SHA-256
`37f7b0f1291af0ed6006c75b6285229f9902d01e1dc5d3363054fd6601d088bf`.

The earlier installed run, 44438292120, used source
`81f53c36d2f18836d8a76dd66948f6f603a49952` and matching package run
44437613964. It passed the previously failing ungranted boundary and created
the application grant through native Settings, then failed withdrawal readback:
32 checks passed, one failed and 158 were not reached. Its retained artifact has
SHA-256 `42174c9edca5381fd322ecebb2b61ce573fff274bd527bde704046d2134a3aba`.
It produced no recording acceptance evidence.

Independent real-Settings tests reproduced the cause before correction:
targeting compared a grant snapshot including its decreasing remaining duration.
The same live grant therefore returned `NothingInSettings` when pressed later.
Settings now matches that grant's stable ID among currently active rows. Other
rows retain exact equality; expired grants and replacements with another ID
cannot be selected through an old control. The normal reader dispatch and click
path persist withdrawal to the real grants file in the regression test.

All 74 focused Settings checks passed, including the independently written
live, expired and replaced grant tests. Three deliberate faults were caught:
snapshot equality, application-name equality and acceptance of an absent row.
After source restoration, all seven reader tests passed again. Formatting,
strict library/test lint and warnings-denied shell documentation checks passed;
five existing vendored Smithay dependency warnings remain visible. Independent
source review found no further concrete blocker in the subsequent permission
and native approval flow. These regressions recur in repository gates; the
three deliberate fault experiments were performed once. The subsequent f5f0f0
installed walk verified withdrawal and reached recording, as described above.

On the preceding source, hosted Studio verification passed native recording,
pause/resume and decoding, 393 instrument tests, desktop regressions and service
builds. The signed package and image build passed. Real-model trials passed all
190 Studio requests and ten close-application requests; generic file, slider,
scrolling, private-typing and chooser failures kept the overall job red. The
repository gate was accepted under its declared runner exceptions: 16,144 tests
passed, one declared permission test failed and 101 were skipped. Ten generated
mutations were tried: five caught and five unviable. This does not close the
earlier broad integration mutation gaps or establish installed media acceptance.

An ungranted Studio command could create an approval card even though delivery
would refuse it. The installed walk caught this before recording. Independent
daemon-router tests reproduced `Proposed` instead of `Refused` both before a
grant and after withdrawal. Separate tests also reproduced missing authority
in successful records: Ask stored no application grant, and Full stored
`by_full_access=false`.

The validated call now includes the checked adapter's application requirement
before proposal or authorization. Existing capability checks enforce it and
retain the grant or Full-access authority in execution records. The additive
call API preserves every original requirement and deduplicates only the new
one. Delivery still checks application permission. Model arguments, instructions
and operation names remain unchanged.

The permission correction passed 316 capability checks (including four
documentation tests), 348 turn tests and twelve daemon permission tests.
Formatting, strict all-target lint and warnings-denied documentation passed for
the three changed authorization crates. Independent review found no blocker in
the additive API or common call entry. Six deliberate faults were caught:
missing application requirements, requirements only on reads or only on
changes, a wrong application identity, discarded original requirements and
duplicate asks. Restored sources were touched and all six focused capability
tests and four daemon regressions rebuilt and passed. These were one-time
fault experiments; the regression tests recur in repository gates. The daemon
test also requires the actual approval card to name exactly Studio.

This candidate also combines the private launch correction, Ubuntu archive
setup, capture boundary and worker refusal checks, and live application-session
and agent-prompt checks. Independent merge review confirmed that those changes
matched their previously reviewed branches before the permission correction.

The combined tree passed 25 focused Rust tests across adapters, agent prompt
refresh, capture-node selection, worker refusals and descriptor validation.
Formatting, strict all-target Clippy and warnings-denied documentation passed
for those four crates. The first local command incorrectly selected a library
target for the binary-only provider; the corrected runner uses binary targets
and stops on any failure. A complete corrected run passed. Independent review
confirmed the tests match their previously reviewed branches. These tests recur
in repository gates; earlier targeted fault experiments remain separately
documented rather than counted as a new combined mutation run.

Hosted native run 44430834993 passed the recording, pause/resume, finalization
and decode checks. Its dependency setup took 32 seconds, but the log confirms
the helper changed zero files and downloads still used the Azure HTTP mirror.
That timing does not demonstrate a benefit from the archive change. Subsequent
test-first support for GitHub's standard mirror-list URI passes fifteen CI
tests and five additional targeted faults. The old workflow mutation test was
corrected to target its intended filter and command; the complete private
instrument suite then passed all 393 tests locally. The preceding launch
correction also passed 393 tests on its hosted runner. These results do not
replace an installed recording walk.

Installed run 44428146796 used source
`00236462b297e862663f941818fa2fc8b23aa18a` and matching manual package run
44427344779. It failed before recording: 32 checks passed, one failed and 158 were not
reached. Ready, package provenance and the initial native permission boundary
passed. The confined helper then reported that an ungranted operation was not
explicitly refused. The retained artifact has SHA-256
`a7413c99947cfa90f9933d1b72988068eec62e30a79a63e81336d3f706cb7872`.
Continuous video, synchronized audio, native approval, withdrawal and
exactly-once execution remain unverified in that installed run. The
low-work recording profile must be audited against its explicit five-frame-per-
second limits; this does not certify the default 1080p profile or physical
microphone hardware. Local signed-repository validation is the authorized
distribution scope. Public Market publication and shared UI-kit migration
remain separate work; the GTK interface remains the development surface.
