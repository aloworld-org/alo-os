# Private recording verification

These instruments are person-side tests, not agent tools. The normal Studio
package has verification disabled. The separate reference package and disposable
KVM walk retain only their bounded reports, pictures and recording files.

After collecting `screen.webm`, `window.webm`, `reference.json` and
`session-report.json`, inspect decoded frames to choose explicit rectangles in
each recording. Store them in a JSON file:

```json
{
  "screen": {"marker_region": [20, 30, 40, 40], "motion_region": [50, 80, 100, 30]},
  "window": {"marker_region": [20, 30, 40, 40], "motion_region": [50, 80, 100, 30]}
}
```

The values above describe the format only. Replace them with observed pixel
coordinates `[x, y, width, height]`; the marker box must avoid text and borders,
and the motion box must include independently changing content. Do not infer a
pass from a flashing marker alone. Both rectangles are bounded by the real
decoded frame dimensions. No default or automatic rectangle selection is used.

On a host with FFmpeg and ffprobe installed, run:

```sh
python3 -I apps/alo-studio/verification/audit_installed_walk.py EVIDENCE \
  --source EXACT_SOURCE_COMMIT --regions regions.json > media-report.json
```

Exit zero means the report's `decoded-recording-media` scope passed. A failed
provenance check blocks decoding; a failed live reference prevents window
synchronization certification. Other failures remain visible independently.
The report retains consumed input hashes, deployment identities, regions and
the health inspectors' measurements. It changes no input files, samples,
pictures or timestamps.

This report cannot certify native consent, export, source isolation, microphone
capture stopping during pause, cleanup, installed agent grants or a hardware
microphone. Full installed acceptance remains `not_reached`, even when the
decoded media scope passes. Complete the separate acceptance walk in
`docs/walks/recording-a-demo-in-studio.md` before describing Studio as ready.

The `verification-instruments` job in `.github/workflows/studio.yml` reruns
these tests on pull requests, including real decoder and isolated GTK/AT-SPI
fixtures. Synthetic test media and patched decoder observations in unit tests
are never installed acceptance evidence.

The separate `studio-agent-probe` binary in `alo-walking` prepares the installed
permission check. It accepts no arguments and requires
`STUDIO_PRIVATE_AGENT_WALK=1`. It reads its own Linux credentials, verifies the
root-owned machine description, and opens the fixed agent socket once. The
person controller must launch it as the image's agent with all capabilities
and supplementary groups dropped and `NoNewPrivs` set. Running it as the person
or root is an error.

Its five stdout lines are `ask-before-grant`, `granted`, `revoked`, `full`, and
`ask-again`. The controller must make and verify the corresponding native
Settings choice before echoing that exact line to stdin. The probe neither
grants access nor approves a change. It checks discovery and valid refused
operations immediately after each boundary, retaining the same connection.
Connection establishment, input acknowledgements and daemon responses share
one five-minute deadline. Use fresh, exclusively owned controller pipes;
arbitrary blocking output writers are outside this instrument's contract.

The private workflow delivers that same-source binary as a separate checksummed
input. The controller requests native Settings, authenticates the disposable
person's sudo session, and launches one confined probe before recording. Native
Settings grants Studio for an hour, withdraws it, and checks the policy changes
on the same connection. Successful cleanup restores the initial policy. If an
owned process cannot be reaped, restoring full trust is refused.

Run this only in the fresh disposable profile with no other controller. Native
read/controls cannot distinguish a concurrent replacement folder with the same
display name as Studio. The helper refuses observable foreign or ambiguous
state; it is not a general grant-management tool.

Unit and local process tests do not establish installed permission acceptance
or prove sudo's descendant shutdown on the image. Actual native permission,
approval, exactly-once execution and execution-record checks remain required.
The person-flow report retains bounded startup/cleanup failures even when no
recording was created.
