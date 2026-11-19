# Studio recording acceptance candidate

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
