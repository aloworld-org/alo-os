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
the three actual-worker tests, with formatting, strict lint and warnings-denied
documentation checks passing. In a one-time snapshot experiment, the old test
genuinely ran and passed a broken `||` to `&&` guard. The new test failed that
same fault because the worker reached the missing-engine refusal instead of
the required argument refusal. All five tested faults were caught: the two
original boolean changes and omission of each node, application and surplus
argument check. Restored production and test sources were touched, then all
three integration tests rebuilt and passed before the shared target was released.

No production code changes. This coverage does not establish installed native
consent, frame publication, decoded recording health, or closure of the broader
mutation gate.
