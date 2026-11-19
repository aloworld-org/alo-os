# Studio agent session boundaries

The earlier mutation run missed replacements of the daemon's production
`ApplicationSession` methods and inversion of the agent's live-name filter.
Direct `SessionBus` tests bypass the former. The existing binary fixture
checks discovery precedes a question, without checking that question's actual
offered declarations.

Independent tests now exercise the production wrapper in a child with its own
session-bus environment. A real private D-Bus service must observe exactly one
microphone-enable call followed by one status read; the exact distinctive reply
must return unchanged. No process-wide environment is changed.

Separate actual-agent process fixtures inspect the current catalogue in three
successive rounds on the same socket: a core read, that read plus all nineteen
Studio controls, then the core read after withdrawal. They run under both Ask
and Full policy input, retain the allowed core declaration, and exclude an
unknown name returned by discovery. Scripted protocol replies advance the loop;
these fixtures are not real-model trials, grant-policy tests or installed-agent
acceptance.

The integration owner observed all 288 unique tests pass across adapters (193)
and agent (95), with formatting, strict lint and warnings-denied documentation
checks passing. The wrapper's recursive child invocation is not counted as an
additional test. A one-time snapshot experiment caught all four representative
faults at their intended assertions: pretending the change was delivered,
fabricating empty or other status text, and inverting the live-name equality
check. Both policy tests caught the inversion. The intended tests were checked
to have executed; restored production and test sources were touched, then the
three unique new tests rebuilt and passed. These tests run again through the
ordinary adapter and agent package suites. This does not close the broader
mutation gate or establish installed acceptance.
