# Claude task: fail-closed kernel publication and remaining boundary audit

Continue in `C:\dev\alo-os-claude`, on `main`. Own only the kernel workstream;
the desktop integration worker owns `C:\dev\alo-os` and the four shared progress
documents. Use descriptive task names, not queue codes.

First inspect the working tree, running workers, published plan and ADR 0020.
Preserve unfinished changes. Pull when clean; integrate incoming main changes
without overwriting another contributor's work. Never run two editors in this
checkout simultaneously.

1. Harden publication after the premature push reported for `7c42297`. Every
   supported publication path, including manual recovery, must require successful
   gates on the exact tree being published. Test that failed gates, zero matching
   acceptance tests, blocked/partial worker results and failed combined-tree gates
   cannot publish. A push race requires integration and fresh verification. Keep
   commits recoverable; no resets, force pushes or removal of another process's
   BPF pins. Keep the supervisor's prerequisite checks and exact-test evidence.
2. Audit the remaining production-reachable network gaps: unconnected UDP,
   inherited/already-open sockets and loopback proxies. Distinguish current-release
   requirements from later hardening. Preserve request-scoped destinations,
   hostname TLS verification, local-model operation, indicator and record behavior.
   Do not widen grants, create a new audit mechanism or silently change accepted
   policy. Record any required new architectural decision and request approval.
3. Run each acceptance test and the applicable Linux workspace, rustdoc and pinned
   BPF gates. Run Windows checks for portable/supervisor changes; never present
   Windows cfg-excluded tests as Linux enforcement evidence. Rebase onto incoming
   main when needed and rerun gates on the combined tree before normal push.
4. Write a descriptive report under `docs/autonomy/updates/`, including proposed
   integration updates. Do not edit `CHANGELOG.md`, `ROADMAP.md`,
   `docs/autonomy/QUEUE.md` or `docs/autonomy/STATE.md`.
5. Commit each completed verified task using the existing owner Git identity,
   without a Co-Authored-By trailer, and push normally to main. Then continue the
   published ready plan through the kernel supervisor, only after the interactive
   editor has stopped writing this checkout. Verify the loop is actually running;
   do not call an idle supervisor active. Stop visibly on unresolved failures or
   decisions; do not silently weaken gates to keep the loop moving.

Report pushed SHAs, exact executed acceptance evidence, remaining limitations,
next task and actual loop status. WSL is development evidence, not physical
hardware acceptance. Do not modify Windows boot, disks or host-wide networking.
