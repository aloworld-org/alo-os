# Schedule shortcuts and hardware acceptance at their dependency phases

Date: 2026-09-08. Owner-directed scheduling and reporting clarification.

The owner asked to defer keyboard shortcuts and physical-hardware acceptance
until needed, keep development ordered, and avoid repeating unchanged reminders.

`DELIVERY.md` now explicitly schedules configurable shortcuts in desktop
interaction integration (phase 3), after the underlying window operations, and
physical acceptance in release validation (phase 8), after integrated VM image
checks. These phases already contained the requirements; the new boundaries
clarify that they are not blockers for earlier independent compositor work.
Neither requirement is removed, completed, or moved to a later release.

`WORKER.md` carries this sequencing into subsequent loop iterations. Ordinary
keyboard input, focus isolation and meaningful component tests remain current
work. Concrete hardware dependencies must still be raised when they actually
prevent progress; no release certification is allowed without physical evidence.

Routine owner updates cover current work and immediate blockers. Unchanged
later-phase obligations remain documented, without a repetitive reminder in
each update. Task reports retain their actual verification limits.

Integration handoff: reference this report in STATE and align current QUEUE and
ROADMAP scheduling with the DELIVERY phase boundaries. Retain unchecked release
acceptance items and historical evidence. No functional changelog entry is
warranted by this documentation-only clarification.

Verification: inspected the existing delivery order and worker instructions;
`git diff --check` passed. No code, test gate, release tier or runtime behavior
changed. Published from the separate integration helper checkout without editing
the live worker's tree or the four single-writer progress documents. A worker
already running may finish its current task before reading the updated plan.
