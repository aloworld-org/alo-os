# Claude task: propose the remaining network-boundary decisions

Work only in `C:\dev\alo-os-claude`, with one editor at a time. Check your loop
status and preserve unfinished changes before pulling main. Read the published
kernel plan and reports first: publication hardening, unwatched filesystem
mutations and inherited descriptors have already landed. Do not repeat them.

Your next deliverable is a decision-ready proposal, not unapproved enforcement:

1. Reconcile your kernel plan's historical tables with the current code and
   published evidence. Distinguish implemented mechanisms, reproduced gaps,
   current-release obligations, later-release hardening and physical acceptance.
2. Prepare a proposed ADR for the production-reachable loopback-proxy egress gap.
   Compare enforcing proxy egress with explicitly trusting owner-started proxies.
   Explain what each means for local models, the visible destination/indicator,
   the record and the existing sovereignty promises. Recommend one option with
   concrete acceptance/refusal tests, dependencies and implementation scope.
   Mark it PROPOSED, not accepted. Do not silently narrow a release promise.
3. Separately outline decisions for inherited file/socket descriptors and checks
   after connection. Account for shared daemon descriptors, audit-record writes,
   and the cgroup descriptor used to leave a turn. Do not assume an extra hook
   is sufficient or that trusted process isolation necessarily permits arbitrary
   agent commands; evaluate these against the actual constitution and ADRs.
4. Preserve ADR 0020's request-scoped destinations, TLS hostname verification,
   local-model behavior, grants, indicator and record semantics. Do not implement
   a new security model, launch processes on behalf of turns, add kernel maps,
   broaden network enforcement or change Windows networking without approval.
5. Publish a descriptive task report containing proposed integration updates.
   Do not edit CHANGELOG.md, ROADMAP.md, docs/autonomy/QUEUE.md or
   docs/autonomy/STATE.md; the desktop integration worker owns those files.
   Use your hardened verify/publish workflow, with exact acceptance evidence and
   fresh gates after integrating incoming main. Preserve existing owner identity,
   no Co-Authored-By, no force push. Never invent a test merely to certify prose;
   if the supervisor cannot honestly publish a documentation-only proposal, report
   that limitation instead of weakening its evidence checks.

Finish with the proposal/report path, pushed SHA if published, recommended choice,
the specific owner decision needed, and actual loop status. Continue only other
already-authorized ready tasks. WSL evidence does not certify physical hardware.
