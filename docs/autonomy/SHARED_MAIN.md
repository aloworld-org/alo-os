# Working together on main

The owner chose direct publication to main on 2026-09-07. Pull before each task,
integrate concurrent work before publishing, and push every completed, tested
task. No feature branch or pull request is required by this workflow.

## Separate checkouts, shared remote

- The continuous desktop worker owns `C:\dev\alo-os`.
- Claude Code uses a separate clone, for example `C:\dev\alo-os-claude`.
- Both local branches can be `main`; they are separate Git repositories.
- Current division: desktop worker owns graphics/compositor and its integration.
  Queue item 6b in `crates/alo-files` is reserved for the Claude handoff in
  `CLAUDE_TASK.md`. The owner starts that session; reservation does not mean it
  is already running. Keep task ownership explicit before taking another item.
- Use a separate Linux `CARGO_TARGET_DIR` per checkout. Coordinate tests that
  alter the shared WSL kernel, cgroups, BPF pins or system services; separate
  build directories do not isolate those resources.

## Task lifecycle

1. Start with a clean working tree and `git pull --ff-only origin main`.
2. Implement one complete task and update its tests and progress documentation.
3. Pass the required Windows/Linux/component checks and make a local commit.
4. Fetch `origin/main` again. If it advanced, rebase only unpublished task
   commits onto it. Resolve conflicts deliberately and rerun the required
   checks on the combined tree before publishing. Never rewrite published work.
5. Push normally to `main`. If another push wins the race, repeat integration
   and verification. Never use force-push or discard another worker's changes.

The Rust supervisor performs this sequence for its worker. It retries up to
three publication races; an unchanged remote after rejection means a real push
error, so it stops and preserves the local commit. Rebase conflicts and failed
integration checks also stop publication and preserve the work for resolution.
The worker itself still does not stage, commit or push; the supervisor does.

Keeping main clean means it contains integrated, tested work from both checkouts.
Pulling only at task start is insufficient: another task can finish while this
one is being implemented.
