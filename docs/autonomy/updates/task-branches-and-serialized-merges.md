# Task branches and serialized merges

The owner approved short-lived task branches on 2026-09-18 so machines can
push progress without racing to update main. SHARED_MAIN.md is the current
procedure: one PR per task, a shared integration turn held by either PC, exact combined-tree
nine-gate evidence, squash merge, and cleanup only after verified integration.
Entry-point instructions now point to it ahead of historical loop recipes.

This is an operational policy change, not a supervisor implementation. Existing
publishers remain paused. Progress pushes may be ungated; main merges may not.
The third PC can continue its owned tasks without holding branch pushes.
Shared crate ownership and all hardware acceptance requirements are unchanged.
GitHub configuration and final gate evidence are recorded in this task's PR;
this document alone does not prove either has been applied.

The owner clarified that both PCs merge their own finished tasks and delete
their merged branches. An atomic temporary GitHub reference coordinates the
final integration turns; neither PC depends on the other to perform a merge.
