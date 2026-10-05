# Two things nobody holds

**Written 2026-10-05 by the Mac lane, for the owner.** Two pieces of work stand
between this repository and the things it says it can do. Neither is a lane's
task, neither is in any plan's queue, and **neither produces a red result
anywhere** — which is why both have been true for some time without being
reported.

This is a brief, not a plan. It assigns nothing and ticks nothing. Each half
ends with the one question only the owner can answer.

## Why they are in one document

They have the same shape, and seeing it once is worth more than meeting it
twice:

- **A blocker nobody meets until they pick the task up.** Canvas task 9's real
  dependency was discovered by starting it, not by reading it.
- **An artefact nobody produces until somebody asks.** The image an installer
  pulls is built by a workflow that runs only on request.

Both are *correct decisions working exactly as designed*. Neither is a bug.
Both stop the product existing. And in both cases the assumption that went stale
was written down — carefully, in the right file — somewhere that could never
interrupt anybody. That last pattern has its own entry:
[`docs/misreadings/a-guard-that-cannot-fire-is-a-comment.md`](../misreadings/a-guard-that-cannot-fire-is-a-comment.md).

**Every number below is a hand measurement taken on 2026-10-05, and nothing
re-runs any of them.** Where a figure can go stale it is given with the command
that produced it, so the next reader re-measures rather than inherits. That is
not a formality here: the commit count below read **417** when the lane that
found it measured it on 2026-10-04, and **422** when this lane re-measured it
on 2026-10-05. The difference is the merges in between — including the ones
written to close the gap this brief is about.

---

## One: multi-output, which canvas task 9 turns out to need

### The question

Canvas task 9 — *every screen is a view onto the canvas* — needs two viewports.
That needs a multi-output compositor, which is a plan-sized piece of work in
`alo-shell`. **Does it become this lane's next plan, or queue behind the two
already assigned?**

### What is already recorded, and not repeated here

The measurement itself lives in the plan it belongs to:
[`the-canvas-and-its-places.md`](the-canvas-and-its-places.md), task 9, section
*What this is actually blocked on*. It carries the six file-and-line citations
and the reason the task's own pre-measurement missed it. **This brief does not
restate them**, because two copies of one measurement is the fault that plan's
task 8 constraint is written against.

In one sentence: the dependency task 9 named — the camera having one home — was
met on 2026-10-04, and the real blocker is that **this compositor advertises one
output**, in eighteen files under `crates/alo-shell/src/` that say so in prose.

```
$ grep -rl --include="*.rs" \
    "single-output\|one output\|only ever one output\|has one display" \
    crates/alo-shell/src/ | wc -l
18
```

### What is not recorded anywhere, and is this brief's reason to exist

**The shape of the remaining work, so that nobody has to scope it twice.** Seven
pieces, of which only the last is what task 9's text describes:

1. Discovery returning every usable port rather than the first — `direct_output.rs`'s
   `select` sorts the ports and returns one.
2. A `wl_output` global per output.
3. A `Presentation` per output. It is `output: Option<Output>` today: one, or none.
4. An atomic modeset and a scanout buffer per CRTC, and a frame per output.
5. The desktop raster laid out per output size — the dock, the status area and
   the put-aside panel each lay out once today.
6. Popups constrained to the screen they are on rather than to *the* screen.
7. **Then** a camera per viewport.

**Pieces 3 to 5 run through the draw path, which every surface in the crate goes
through.** That is the whole argument for this being a plan rather than task 9's
remainder, and it is also the argument against starting it casually: the suite
is single-output throughout, so a half-built second output would **stay green
while the second display drew nothing anybody could use.**

### What it buys, stated narrowly

It completes task 9's **code**. It produces **no** `On the machine.` tick: the
acceptance asks for *two displays, each at its own zoom*, shown, and this lane
has one laptop and no second display. The code half is real work and should not
be sold as more than it is.

### Scope is settled, so it is not re-litigated when this is picked up

`docs/features.md:491` — *every screen is a view onto the canvas* — is `[v0.01]`.
`docs/features.md:99` — *splitting works on an external display independently of
the laptop's own* — is `[v0.5]`. **Multi-output makes two existing promises true
rather than adding one**, and `alo-shell` has had one owner since 2026-10-03.
The question is *when*, not *whether*.

### The question for the owner

**Does multi-output become the Mac lane's next plan, or does it queue behind
`access-and-language-plan.md` and
`models-a-person-adapts-and-subscribes-to-plan.md`?**

Until that is answered, canvas task 9's status reads **blocked** rather than
*ready*, so that no lane picks it up and rediscovers the same thing.

---

## Two: an image, which nothing installable has had since September

### The question

**No image that an installer pulls has ever contained the compositor.** Who runs
the build, on which machine, and when does the owner sign?

### What was measured, and by whom

**Found by the laptop lane** on 2026-10-04, while reading a serial capture from
the installer walk and finding no alo OS output in it. Their first reading was
that the compositor was failing to start; the correct reading is that there was
no compositor on the disk to start. **Re-measured independently by this lane on
2026-10-05**, because a finding this load-bearing should not reach the owner
through one pair of hands:

```
$ grep revision image/pinned.toml
revision = "97c970c96f2c13f2341ed94fb264247fb6837442"      # release 0.0.5

$ git log -1 --format='%ad' --date=short 97c970c9
2026-09-20

$ git log --format='%h %ad %s' --date=short --diff-filter=A \
    -- image/usr/lib/systemd/system/alo-compositor.service
7fc158a1 2026-09-28 The compositor a machine boots to, in the image (#213)

$ git grep -c alo-compositor 97c970c9 -- image/    # the pinned revision:
                                                  # no output, so absent
$ git ls-tree --name-only 97c970c9 -- image/      # and image/ did exist there
image/Containerfile
image/Containerfile.dockerignore
image/etc
image/installing
image/pinned.toml

$ git grep -c alo-compositor origin/main -- image/
origin/main:image/Containerfile:6
origin/main:image/usr/lib/systemd/system/alo-compositor.service:3

$ git rev-list --count 97c970c9..origin/main
422
```

**The pinned image was built eight days before the compositor entered
`image/`, and `main` has moved 422 commits since.**

### Three corrections to the first telling, because two of them invite a wrong fix

The finding is right and three of the details in its first telling were not.
They are corrected here rather than quietly restated, because one of the errors
makes the problem look cheap to fix.

**The git tag and the published image are different commits.** The tag `v0.0.5`
points at `3df82d54`, dated **2026-09-27**. The image was built from
`97c970c9`, dated **2026-09-20**. Seven days and two artefacts under one name —
which `ROADMAP.md:207-208` already warns about in as many words: *"A sentence with
`v0.0.5` in it is wrong whichever was meant."* **This matters because the tag is
a one-day near-miss on the compositor, which invites "just retag it", and the
image misses it by eight days and everything built since.** The honest figure is
the image's, because the image is what reaches a disk.

**Five images have shipped, not one.** `ROADMAP.md` records five inside one
unfinished milestone. What there is one of is a *pinned* image —
`image/pinned.toml` names exactly one digest, *the one alo OS image an installer
pulls* — and that one is 0.0.5.

**`release.yml` is not the image road.** It builds the Windows installer
executable and publishes a draft Release, on a `v*` tag. The image is built by
`image.yml` alone, whose trigger is `workflow_dispatch:` and nothing else.
Conflating them makes the handle look like it is attached to tagging, which it
is not.

### Why nothing reports it, and why that is not a bug

Three deliberate decisions, each well argued in the file that holds it:

- `image.yml` runs on request only — *"It runs only when somebody asks it to, so
  it is never a red run nobody asked for."*
- It **never signs**. ADR 0036 reserves that to a person holding the private
  half of the key, signing by digest.
- A digest reaches `image/pinned.toml` **only once that signature verifies**.

None of that is wrong. The gap is that **nobody has turned the handle in two
weeks and no artefact says so.** There is no staleness check anywhere, because
an image that is never built cannot fail a test.

### What this reframes

`evidence-it-boots-and-the-agent-acts.md:323` already records the effect: *"No
certified machine has ever displayed this compositor."* It attributes it to the
tests running nested. **That is true and it is not the binding reason.** The
binding reason is that the compositor has never been in an installable image, so
no amount of further testing, and no walk on any machine, could have displayed
it.

The same correction applies to the exit gate's machine column. *Nothing has been
shown on a machine* has read as *nobody has got round to checking*. It is
**the thing to check has never been produced**: capabilities with finished code
have never existed on a disk together, because the last pinned image predates
most of them.

And [`the-week-to-the-first-install.md`](the-week-to-the-first-install.md)'s
section *What is knowingly missing on the day* — written so that *silence cannot
imply more was tested than was* — lists no video, no update policy, a refused
camera and microphone, and the three promises only a chip can keep. It says the
day tests *the install and the first start*. **It does not say the desktop is
absent**, which on the pinned image it is. That list was right about everything
it named; what it could not name is the thing nobody had measured.

### What this lane will not do

**Build or publish an image.** `alo-image`, `image/` and `.github/workflows/`
belong to the installer lane; publishing is outward-facing; ADR 0036 reserves
signing to the owner. This lane holds a working toolchain and a virtual machine,
which is a reason for more care rather than less — **having the means is the
whole of the temptation and none of the authority.** The laptop lane reached the
same conclusion independently and for the same reasons.

### Two things not verified here, so nobody treats them as measured

- **The registry.** Every statement above is about what this repository records.
  This lane did not query `ghcr.io`, so *no image containing the compositor has
  ever been published* is a claim about `image/pinned.toml` and git history, not
  about what tags exist on the registry.
- **The runner's disk.** `image.yml`'s own header says a hosted runner cannot
  hold the build — 4.5 GB of weights on a bootc base, against roughly 14 GB free,
  measured 2026-09-12. That is the file's measurement, quoted, not re-taken.

### The question for the owner

**Is the installer lane asked to run `image.yml` from current `main` tonight,
and do you sign the resulting digest?** The road is three steps and only the
first is a lane's: build a candidate, the owner signs it by digest, the digest
is pinned.

**And a second, smaller one: does the staleness check become a task?** Nothing
anywhere compares the pinned revision against `main`. That is this document's
guard-that-cannot-fire pattern at release scale — the assumption *the pinned
image is current* is written down nowhere and checked by nothing. It belongs to
the installer lane's crates or the workflows, not to this lane, which is why it
is a recommendation here rather than a change.

---

## The order between the two, recommended

**The image first, and it is not close.** Multi-output makes one more capability
reachable on a machine that does not yet exist. An image is the precondition for
*any* of the machine halves ever being shown — including every capability this
fleet finished in the three weeks since the pinned revision.

Multi-output is worth scheduling and is not worth hurrying, because its own
acceptance needs a second display that this lane does not have. It can be built
well, later, by a lane that can see it work.

## What this document is not

It is not a plan and it creates no tasks. It does not edit `QUEUE.md` or
`STATE.md`, which have one writer by
[`SHARED_MAIN.md`](SHARED_MAIN.md). Where a decision here turns into work, the
work belongs in the plan that owns the crate, and this file should then say so
rather than grow a queue of its own.
