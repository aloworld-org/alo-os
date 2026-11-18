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
Compiler failures found during integration are being fixed before the installed
recording walk. A build or unit test pass does not establish smooth video or
audio synchronization.

The shared UI kit's application window, input, control and accessibility layers
remain unfinished prerequisites. Studio's GTK interface is a development test
surface. Neither the shared-kit migration nor public market availability is
complete. Local signed-repository verification remains the authorized
distribution scope. No unrelated verification VM is to be paused.
