# Studio worker argument refusals

The mutation artifact at `0c6d1351150f95d7539285b085836a806d4fac2a`
missed changes to the capture worker's argument guard. The existing process
test required a failed exit and an error prefix. A broken guard could admit a
bad argument, fail later while constructing media, and satisfy those assertions.

Independent process assertions now distinguish the stages. Invalid node and
application identities are tested separately, alongside an extra argument.
Each must return the exact argument error and no node announcement. An isolated
empty plugin registry makes the subsequent engine check a distinct refusal;
valid arguments must reach that exact engine error. No media graph or hardware
is needed for these checks.

The ordinary provider integration suite runs these assertions on every test
run. The integration owner observed all eight provider tests pass, including
the three actual-worker tests. Strict lint and documentation checks are pending.
A prepared one-time experiment will require the old assertion to pass a broken
guard, then the new assertion to reject that same fault and four further guard
omissions/boolean changes. That experiment has not run yet. It restores and
touches snapshot sources and rebuilds the passing baseline before release.

No production code changes. This coverage does not establish installed native
consent, frame publication, decoded recording health, or closure of the broader
mutation gate.
