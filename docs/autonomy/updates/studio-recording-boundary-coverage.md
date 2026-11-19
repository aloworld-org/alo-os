# Studio recording boundary coverage

The integration mutation run at `0c6d1351150f95d7539285b085836a806d4fac2a`
tried 182 of 1,285 faults within its hour: 106 caught, 52 missed and 24
unviable. Another 1,103 were untried. This follow-up addresses selected status
read, capture identity and descriptor boundaries; it does not turn that failed
gate into a pass.

Status reads now have tests for the inclusive 8,192-byte limit, multibyte text,
unanswered reads, the returned application identity, and legacy transports that
cannot read. A real private D-Bus fixture checks the transport limit directly,
so the outer adapter guard cannot hide a broken transport guard. Undeclared
change-description words are refused.

Capture tests transfer actual file descriptors with wrong markers, empty
payloads and excess descriptors, and include zero-height frames. Provider tests
exercise bounded opaque identities and the actual worker's argument and missing
private-engine refusals. The registry callback's existing node-selection rule
is extracted into a borrowed helper, preserving the exact node name, kind, id
and serial. Independent tests existed and failed compilation for that missing
helper before extraction. Live registry discovery, its timeout, frame feeding
and the complete publishing lifecycle still require further coverage.

On 2026-11-18, all 205 tests across adapters, capture protocol and provider
passed locally, as did strict lint and warnings-denied documentation. These
tests run again through the ordinary package suites. Independent review found
the cases exercise their intended boundaries. Seventeen representative
deliberate faults were caught in one-time experiments. The initial provider
unit selection omitted binary tests; those seven cases were rerun with the
correct target and each failed at its intended assertion. Restored sources
were touched and the combined 61-test focused baseline rebuilt and passed.
Empty
payload and truncated ancillary-data cases can fail at more than one guard,
so passing them does not prove every individual guard indispensable.

This is separate from the frozen installed approval and recording revision.
No private fixture establishes native consent, decoded media continuity,
audio synchronization or public release readiness.

Hosted native and desktop recording checks in run 44419989906 passed, as did
image build 44419989837. The hosted workspace gate exposed one formatting
difference in the new provider test that the local snapshot had already
formatted. That correction is now in the owned source, and a fresh full
workspace formatting check passed there. Of 16,061 hosted tests, 16,060 passed;
the remaining permission test is explicitly listed in
`.github/the-tests-a-hosted-runner-cannot-run.txt` because it requires a non-root
process. The formatting failure, rather than that declared runner limitation,
blocked the gate. The corrected revision still needs its hosted verdict.
