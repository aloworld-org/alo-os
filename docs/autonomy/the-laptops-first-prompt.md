# The laptop's first prompt

The certified laptop (`docs/hardware.md`) arrived on 2026-09-14. Until the
installer exists (`v0-5-the-installer-plan.md`) it is a 32 GB Windows machine,
and that makes it the one machine this project has that can hold the four
catalogue models the Mac could not. This prompt uses it for exactly that, with
nothing installed on it that a customer would not also install.

Install [Claude Code](https://claude.com/claude-code) and [Ollama for Windows](https://ollama.com)
on the laptop, clone the repository under your home directory, open Claude
Code in the checkout, and paste everything below the line.

---

You are working on the **certified laptop** of alo OS —
`github.com/aloworld-org/alo-os`, a sovereign AI-native operating system. This
machine will later be installed with alo OS through the installer being built
now; today it is a Windows machine with 32 GB, and you are using it for the one
measurement no other machine here can make.

**Read first, in this order:** `CLAUDE.md` (the constitution — absolute),
`docs/autonomy/the-mac-lanes-first-prompt.md` (the rules for sharing one
repository with three other lanes: one branch `main`, pull before you start
and again before you push, push after every finished task, never force-push,
no `Co-Authored-By` trailer, commit as the configured git user), and
`docs/autonomy/v0-5-the-models-measured-plan.md`, **task 9**.

**Your one task is that task 9** — *the four entries the measuring machine
could not hold* — and you are the machine that can hold them: `eurollm-9b-instruct`,
`teuken-7b-instruct`, `mixtral-8x7b-instruct`, `gemma-2-9b-instruct`. Grade
each the way the Mac graded the others, and the plan and the Mac's reports say
exactly how: the same ten exercises in `alo-driving`, the same bar (nine in
ten, with task 6's second-round rule), asked both freely and held to the
protocol's envelope (ADR 0032), through the pinned runtime — **Ollama 0.34.0
exactly**, the version `image/Containerfile` pins; check `/api/version`
before anything — with the machine, date, runtime, digest and counts written
beside every grade, and the attempts verbatim in the report. Read the licence
of each against its publisher's own repository before fetching, as the Mac
did (its report for task 11 shows the shape). The report names this machine:
its exact model, CPU, memory, and whether the weights ran on a GPU or the CPU.

**Set the machine up yourself; do not ask.** The Linux side the gates need is
what the PC has: Ubuntu under WSL2 with the toolchain `docs/autonomy/LOOP.md`
lists and `a-loop-on-a-mac.md` step 2 details. Ollama runs on Windows. Rust
on Windows for the harness if you need it; the gates run in WSL. Everything
the Mac found setting up is in `docs/autonomy/a-loop-on-a-mac.md` and
`docs/quirks.md` — read them before you hit the same thing.

**What you may touch:** `crates/alo-models/data/catalogue.toml`, a report in
`docs/autonomy/updates/`, task 9's status in the plan, and `docs/quirks.md`
for anything this machine taught you. Nothing else — the model crates are the
Mac lane's and it may be working on its task 16 at the same time; pull before
you push, and if its catalogue changes land under yours, rebase rather than
overwrite. If a grade needs a change to `alo-driving` or `alo-models` code,
that is a finding in your report, not an edit.

**The rules that never move:** the exercises, the bar and the door are not
changed to suit a model; a grade with no machine beside it is a claim; a
model that cannot be loaded says why (`too-large-for-the-measuring-machine` is
not available to you — you are the machine that holds them; if one still will
not load, measure and say what stopped it); nothing leaves this machine except
the `ollama pull` you name in the report; and **nothing is ticked *on the
machine*** — a model graded on Windows on this laptop is a fact about the
model, not a certification of alo OS on this hardware.

**If any of the four clears the bar**, say so in the report's first paragraph
as *the first catalogue model that could be given the agent*, with its
quantisation, counts and residency — and stop there: what the catalogue
recommends and what the image pins are decisions the plan makes in tasks 15
and 6 of `v0-01-lane-b-plan.md`, not this run.

**When you are done:** commit, pull, push, and tell the owner in a short
report: the machine's exact model and memory, Ollama's version, each model's
two grades with counts, whether any cleared the bar, and what you fetched.
Then stop; the laptop's next job is to be installed.
