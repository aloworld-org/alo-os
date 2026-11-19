# Studio recording acceptance candidate

This candidate combines the private launch correction, Ubuntu archive setup,
capture boundary and worker refusal checks, and live application-session and
agent-prompt checks. Independent merge review confirmed that the launch,
native action-name handling, private instruments, walking and shell code remain
byte-identical to installed-test source `00236462b297e862663f941818fa2fc8b23aa18a`.
The only additional production change is the reviewed capture-node selection
helper extraction. No model instructions or offered operations change here.

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
and decode checks. Its dependency setup took 32 seconds, compared with previous
15-minute timeouts. This is one hosted measurement, not a download guarantee.
The preceding launch correction's hosted private instrument suite passed all
393 tests. These results do not replace an installed recording walk.

Installed run 44428146796 uses the unchanged source above and matching manual
package run 44427344779. Its continuous video, synchronized audio, native
approval, withdrawal and exactly-once execution results remain pending. The
low-work recording profile must be audited against its explicit five-frame-per-
second limits; this does not certify the default 1080p profile or physical
microphone hardware. Local signed-repository validation is the authorized
distribution scope. Public Market publication and shared UI-kit migration
remain separate work; the GTK interface remains the development surface.
