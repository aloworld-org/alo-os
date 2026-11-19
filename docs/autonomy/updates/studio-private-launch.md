# Studio verification starts its confined helper

Installed run 44418962139 reached Studio Ready but stopped before recording:
32 checks passed, one failed and 158 were not reached. Authentication succeeded,
but `sudo -k -v` deliberately did not cache credentials for the subsequent
noninteractive launch. Invalidation and validation now run separately. The
temporary ticket is invalidated again on every exit.

The controller now requires the child's first exact stage token before native
Settings preparation. Child launch diagnostics are bounded and retained, and
the uploaded person-flow failure includes bounded command stderr. Native state
checks and the rule of one changing action per boundary remain unchanged.

Independent tests were run first: the documented credential model and real
unprivileged child pipes exposed the old behavior, and the uploaded-report
test exposed missing native-command stderr. After correction, the full suite
ran 393 tests: 392 passed and one legacy assertion confused identical initial
and final invalidation commands. Correcting that assertion was followed by
four passing tests in that module. Independent focused validation passed all
26 tests. Seven deliberate faults were caught, and the restored focused suite
passed again. These tests recur in Studio's verification-instruments workflow;
the seven fault experiments were one-time measurements.

A separate one-time measurement used real sudo in a private mount namespace
with temporary account records, policy and timestamp storage. The old sequence
returned validation 0 and launch 1; the corrected sequence returned validation
0 and launch 0; final invalidation returned launch 1. No host account or policy
was changed. This local measurement does not prove installed permissions.

The failed installed artifact has SHA-256
`92cadecbce12645995768a5bea6298c3b919b943655cf07f457a4dfd380f9d31`.
Native preparation also failed pressing **What alo can use**, before the old
controller read the dead child's pipes. Its missing stderr prevents a causal
diagnosis of that separate failure. Installed recording, decoded media and
native approval acceptance remain required. No model-facing instructions,
verbs or production Rust change in this correction.
