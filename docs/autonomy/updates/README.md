# Task reports

One task, one descriptive report, one writer. This directory keeps parallel
contributors from appending to the same progress files. See
`../SHARED_MAIN.md` for publication and integration ownership.

Choose a unique lowercase hyphenated filename describing the change, such as
`secure-file-moves.md` or `native-desktop-cursor-rendering.md`. If that name is
taken, describe the distinct follow-up, for example
`secure-file-moves-refusal-tests.md`. Do not use code-only queue identifiers.
Legacy identifiers may appear as secondary references inside the report.

Each report contains:

- A descriptive title, date, workstream and responsible contributor.
- What changed, with relevant source paths and a user-readable change description.
- Decisions and acceptance criteria, including required approvals.
- Exact verification commands, platform and results; distinguish executed checks
  from pending ones, skipped measurements and physical acceptance.
- Remaining limitations and proposed changelog, roadmap and queue updates.
- Status: in progress, ready for integration, or blocked with the exact reason.

The commit containing the report identifies the code under review; no report
needs to predict its own commit hash. Keep reports scoped to one task, and never
change another contributor's report. After publication, corrections use a new
descriptively named follow-up report.

The integration owner consolidates ready reports into the shared documents and
references their paths in STATE.md. Reports are evidence to review, not automatic
permission to tick a release gate. README.md is guidance, not a pending report.
