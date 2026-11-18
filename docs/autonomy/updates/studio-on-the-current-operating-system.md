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
GUI recording check still needs execution.

The real-model trial at that earlier head failed despite five repetitions at
each approval level: status, three/five-second countdown and finish requests
still took wrong roads. The model is now told to prefer a matching application
operation before the accessibility fallback; recording status, countdown and
automatic review descriptions are clearer. This change requires a new repeated
trial. Scripted tests do not replace that trial or prove execution through the
installed agent.

The private KVM image built and reached its OS boot walk. That walk does not
install Studio or establish recording acceptance. A separate private reference
window and real Pulse monitor are under development for the installed walk.
Their initial local timing check failed; no synchronization pass is claimed.
The actual application handlers, native consent and byte-identical export,
decoded video continuity, audio continuity and matched synchronization markers
still need installed evidence. A build or unit test pass does not establish
smooth video or audio synchronization.

The shared UI kit's application window, input, control and accessibility layers
remain unfinished prerequisites. Studio's GTK interface is a development test
surface. Neither the shared-kit migration nor public market availability is
complete. Local signed-repository verification remains the authorized
distribution scope. No unrelated verification VM is to be paused.
