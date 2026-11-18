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

Native Settings is selected with `alo-app alo --in settings`: the lowercase
identifier is the published surface contract. `Settings` is its displayed
title, and using that title for alo's own surface returns no matching window.
The independent controller regression keeps those two names distinct. A
one-time isolated AT-SPI reproduction with the actual `alo-app` binary observed
the uppercase refusal and lowercase success; installed recovery still needs
the private walk.

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

The private `record_delta` reader prepares the execution-record part of that
check. It compares a bounded append with a validated checkpoint, refuses
replacement or rewritten history, and requires one microphone change under the
expected agent, approval and policy. Unknown events refuse certification; this
is deliberately a reader for the fresh walk's supported events, not a general
record export. Its independent JSONL fixtures are synthetic test inputs.
`test_record_delta.py` reruns with the verification instrument suite.

The private approval controller joins this reader with a genuine Ask alo
request after provider setup. A matching record alone proves neither native
approval nor one D-Bus delivery. Installed approval acceptance also requires
the native approval card, independently observed application state and delivery
count, and the matching appended record. A direct agent proposal does not cause
the shell to show a card without the task window's real reply path.

`delivery_observation` prepares that separate delivery check on the person's
session bus. It requires one changing method call, its matching successful
reply, and an observed status-query barrier after the real agent turn ends.
Owner changes, duplicate calls, error replies, disconnection, excessive traffic
and missing replies refuse certification. `test_delivery_observation.py` uses
an independent service on an isolated session bus; it does not run installed
Studio. The private approval controller calls this observer after actual agent
exit, including a zero-delivery barrier for the withdrawn change.

The observer also refuses interface-omitted calls to the recorder object: D-Bus
allows them, so monitoring only the declared interface could hide a duplicate
delivery. An independent private-bus regression reproduces that case.

`asking_turn_observation` prepares the corresponding process-completion check.
It subscribes on the system bus before an empty asking-unit baseline, then
captures the fresh pending turn's exact agent command, role, invocation, PID
and monotonic start time. Completion requires that same process's successful
exit facts; inactive state or a removed unit alone is insufficient. Cached
signal facts survive systemd garbage collection. Replacement, invalidated
identity, owner loss, ambiguous starts, malformed wire types, excessive traffic
and timeout refuse certification.

The initial command identifies the observed process; this does not attest to
unchanged unit configuration afterwards. systemd normally invalidates its
command/status tuples at exit. The observer tolerates those notifications but
continues requiring the anchored process identity and explicit exit facts.
`test_asking_turn_observation.py` uses a typed service on a private bus. A
separate one-time WSL systemd observation established the notification order;
neither is an installed Ask alo result. The private approval controller uses
`capture_started` to wait for a fresh Full turn to become
active, validating its identity through the startup transition. It refuses a
turn that exits before the process identity was captured. `capture_pending`
keeps its shorter three-second wait after a native approval card appears.
The two capture methods are single-use alternatives, not retries.

The expanded lifecycle suite has 40 passing private-bus tests. Six one-time
startup faults were caught in temporary copies. Two startup-only identity
faults initially survived; independent follow-up tests now catch both. These
fault experiments are not recurring gates or installed evidence.

The checksummed `studio-session` archive has sixteen fixed modules and a
separate `--approvals` entry. It requires the disposable installed person
session and owns a fresh normal Studio instance. The host submits three real
Ask alo requests: enable with native Allow once, disable after withdrawing the
grant while that same card is pending, and disable at Full access. The pending
reader sends only a bounded Waiting query; native keys supply approval. The
controller requires matching process exit, application state, appended record
and recorder delivery for each request. A refusal record has no approval ID;
the separately observed pending card is retained without inventing one.

An independent cleanup section opens native Settings even after the approval
section fails. It restores the initial policy only after observed agent exit;
otherwise it withdraws its owned grant without increasing access. If Settings
cannot be reached, it records that failure and still stops and reaps its owned
application and attempts report delivery. Later model sections require both
approval and cleanup to succeed. The bounded `approval-report.json` is a
write-only retained output, never an executable input. These paths have local
regression coverage; their first installed native-approval result is pending.

In a fresh profile with no audio devices, `pw-dump` can omit the empty default
metadata store even while `pw-cli` lists it. Startup validates saved microphone
history and any observed live preference before creating its owned fixture.
It then waits within a bounded interval for complete default metadata before
selecting that microphone or launching Studio. Only absence may retry;
malformed metadata, duplicate stores and configured microphone preferences
refuse immediately. The startup-order regressions rerun with the verification
instrument suite. This check does not certify installed audio or media quality.
