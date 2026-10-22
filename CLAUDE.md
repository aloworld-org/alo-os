# CLAUDE.md — the alo OS constitution

alo OS is a sovereign AI workstation: an operating system whose
interface is an agent and whose first-class workload is a model the
customer owns. It is built by a very small team with big-tech
discipline. You are that team. Everything here is absolute;
everything else is judgment.

The workplace that runs on it lives in `alo-workplace`, and it is
two products over one platform — **mail and meet** — which
**ship as separate applications**, each installed, updated, granted
and opened on its own ([ADR 0122](docs/decisions/0122-the-workplace-ships-as-separate-applications.md)).
*Projects instead of applications* is the layer above them, not the
thing that packages them together. The rendering engine is
`alo-engine`, and it is **started**: it is the repository
`aloworld-org/alo-browser`, which already renders an alo screen. Its
first stage needs no operating system and no hardware, so it does not
wait on this repository and this repository does not wait on it
([ADR 0002](docs/decisions/0002-the-shell-is-native.md)). This
repository is the system underneath both: the shell a person signs
into, the service that lets an agent reach the machine, and the image
that boots.

## The five laws

1. **Nothing leaves silently.** Every network egress an agent causes
   is visible at the moment it happens and afterwards in a record. On
   a machine sold on sovereignty the indicator is a feature, not a
   diagnostic. With a local model a working day produces **zero**
   inference egress, measured at the network boundary — and we
   publish the measurement rather than the promise.
2. **No code runs unless the person chose it.** Every capability on
   the broker's list is an enumerated verb with typed, validated
   arguments. No `exec`, no shell and no "advanced" escape hatch is
   ever added to that list, because a model that can write code that
   runs has escaped every other control in this repository. Running
   code is a separate grant that only the person gives, at one of three
   levels (ADR 0064): a sealed box the kernel locks down (the default),
   asking each time, or full trust. At every level each run is recorded
   and can be undone. The law binds the **agent**, never the person:
   alo OS ships a terminal, because a system that does not trust its
   owner with a shell is a toy.
3. **Done means the machine still works.** Input → validation →
   policy → execution → record → error paths, on real hardware. An
   OS that boots but cannot print is not a released OS. No `todo!()`,
   no `unwrap()` outside tests, no stubs, no "we will finish it in
   the next change". When time is short cut **scope** — one printer
   that works, not half of all printers — never depth.
4. **One file, one responsibility.** A file that gains a second
   reason to change gets split in the same change that discovered it,
   not noted for later. This is a law rather than a preference
   because of what this repository is: a privileged service, a
   compositor and a policy engine, where the file nobody wants to
   open is where the security bug lives. Small files are how the
   capability model stays reviewable by somebody who did not write
   it.
5. **The person chooses. alo never takes the choice away.**
   *The owner, 2026-10-14: "we are not dictators and we give the users a
   choice to choose."* On their own machine, a person decides what alo OS
   does. Every protection is a default they can change, not a wall.
   Anything with a risk is offered with the risk in plain words and
   switched on by the person.
   It is never removed from them "for their own good". Two kinds of
   protection stay on at every setting, because neither takes a choice
   away: those that restrict nothing (the record, undo, the egress
   indicator, plain warnings), and those that guard the person from
   others (no telemetry, never a silent fallback, helpdesk only when
   invited). Only the law, or an organisation's policy on machines it
   owns (ADR 0016), may truly forbid. A change that takes a choice
   away from the person is a bug, whatever the reason given for it.
   The review question for every feature: *can the person choose this,
   knowing what it costs?* (ADR 0064)

   **What this law reaches, and what it does not.** It is about a choice
   a person **has or was promised** — anything this repository has told
   them they may decide. It is not a duty to make every decision
   configurable: a shell that offered a setting for each of its own
   behaviours would be unusable, and some of its best decisions are ones
   nobody is offered. The reveal has no delay before it appears and no
   setting for one, because a delay makes reaching a surface depend on
   how fast somebody can move, which is the one thing a person with a
   tremor or a trackball cannot control. That takes nothing away; it was
   never offered, and offering it would take something away from them.
   **So: never offering is not taking away. Withdrawing is.** What
   `docs/features.md` promises is the record of what was offered, which
   is what makes this checkable rather than a matter of opinion.

   **And "nobody has chosen it yet" is not a reason.** Before a machine
   boots, no choice has been exercised — not one accessibility setting,
   not the dock's, not the indicator's. A reason that turns on nobody
   having used a thing yet is not an argument about that thing; it is an
   argument that this law does not apply until somebody boots, which is
   the whole period in which we are deciding. *Whatever the reason given
   for it* is not addressed to bad reasons. Nobody needs protecting from
   those. It is addressed to good ones.

   *Both paragraphs were added on 2026-09-30, after
   [ADR 0076](docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
   was found to have withdrawn a promised choice seven days after this
   law landed, citing it zero times, and to have given that exact reason
   for doing so. The law was not wrong; it was unreadable on the
   question, because its headline is broader than its body and two
   careful readers disagreed about which bound.*

## The gate — nothing is done until all of this passes

There is no "and we will add the tests afterwards". A change that ships
without them has not been finished, it has been abandoned.

- `cargo fmt` clean and `cargo clippy -D warnings` clean. **Zero**
  warnings — not "known warnings", not "warnings we live with".
- Unit tests for logic, and **the refusal path tested as carefully as
  the happy path.** For anything an agent can reach, "and it was
  stopped" is the test that matters. Code tested only when it works
  has not been tested.
- **The capability guarantees are tests, not prose.** Each of these
  is a test that runs in CI, and each of them is a promise we make in
  public:
  - with no invocation, `alo-agentd` makes no context calls at all;
  - a verb cannot reach outside its grant;
  - a revoked grant takes effect immediately, and an expired one is
    gone;
  - one approval causes exactly one execution;
  - every execution *and every refusal* leaves a record;
  - no agent-caused egress escapes the indicator.
- **An integration test on real hardware** for anything that touches
  the machine. A green suite on a developer's laptop proves nothing
  about a certified workstation, and this is an operating system.
- Documentation in the same change: rustdoc on public items, the
  contract updated if a public surface moved, `docs/quirks.md` when
  reality disagreed with a specification.
- A user-readable change description, written while the knowledge is fresh.
  Parallel contributors include it in their task report; the integration owner
  consolidates it into `CHANGELOG.md` under `docs/autonomy/SHARED_MAIN.md`.
- **Say whether a measurement was performed once or runs forever.** A proof
  somebody did by hand and a check that runs on every build are different
  things, and **a report cannot tell them apart unless the writer says which it
  was.** Both read as *measured* a week later. So a report that describes a
  measurement names where it will run again — a test, a build step, a gate — or
  says outright that it will not.

**A check that stands in for the thing is not the thing**, and this repository
has rediscovered that four times in four sets of clothes. It is written here
because it is cheap to catch in review and expensive to catch in a release:

- `test -x` on a converter binary stood in for the converter **starting**. Three
  releases shipped one that died at launch on eleven missing libraries, and no
  document of any format opened on a real machine.
- `gst-inspect-1.0` on an element name stood in for a file **decoding**. It still
  does; that gap is open and named in the recipe and in
  `crates/alo-image/tests/the_image_can_play_what_the_decision_says.rs`.
- A hand measurement in the pinned base — real, and correct — was written up as
  though the build did it. The claim was false the day it was made, in the same
  commit as the code it described, and stood for six days.
- A suite green on one machine stood in for a suite that passes anywhere. Three
  unrelated tests assumed they were alone on the machine; each failed a gate for
  a reason that had nothing to do with what it tests.

The question to ask of any green thing is **what would have to be true for this
to pass while the product is broken** — and then whether anything checks that.

**And no rushing.** A date never justifies a shortcut. When something
has to give it is scope — one printer, one certified machine, one
adapter — because a system built in a hurry is a system nobody can
trust, and being trustworthy is the only asset this product has. We
would rather ship less, later, than ship something whose guarantees we
cannot demonstrate.

## Standing rules

- **Read the promise whole, then ask, then build.** Before building
  anything, find what this repository already promises about it — in
  `docs/features.md`, in `docs/design/`, in the design file, in an ADR —
  and **read the sentence to its end**. Where the promise could be read
  more than one way, **put the reading to the owner before writing
  code**, not after. Added by the owner's direction on 2026-10-09:
  *always first understand the promise, discuss it with me and then
  build, because otherwise this is not good to build without
  understanding fully the rules.*

  **The qualifier is usually the rule.** `docs/features.md` promises
  *nothing is stacked — no window is buried behind another **where the
  person cannot find it***. Read to the dash, it forbids one window
  overlapping another; read to the full stop, it forbids alo losing
  one. On 2026-10-09 this lane had designed the first and was about to
  build it, which would have stopped a person putting one window behind
  another on their own machine — **a change that takes a choice away
  from the person, which law 5 calls a bug whatever the reason given for
  it.** The owner caught it by asking whether it was really meant.

  **And code is not the authority on what exists.** Twice on 2026-10-08
  two lanes reported a thing missing after reading only the code — the
  Dock's contents and the window frame — and both were designed in full,
  one of them a component used 1,814 times. A thing absent from the code
  is a thing **not yet built**, and which of those two it is cannot be
  read off the code at all. Look for the design before reporting an
  absence, and say which you looked at.

- **Task branches, one merge coordinator.** Follow
  `docs/autonomy/SHARED_MAIN.md`: branch checkpoint pushes may precede full
  gates; only an exact, fully gated combined tree may be squash-merged through
  a pull request into `main`. Keep legacy direct-to-main publishers paused.
- **One task at a time; branches bounded by the machine, not by a
  number.** Each agent actively implements **one** task. A machine holds
  as many unfinished branches as it can keep **gated and green**, each in
  its own worktree — in practice two or three where a gate takes minutes,
  one where it takes an hour. **The binding constraint is one Cargo
  operation per machine**, so branches beyond what the machine can build
  are paperwork rather than throughput: they queue on the same compiler.
  **When a task is waiting on CI, review or a decision somebody else
  owns, start another approved, independent task rather than idling.**
  *One branch until its PR merges* reads like discipline and is a stall;
  the fixed count that replaced it was itself replaced on 2026-10-02 by
  the owner, once the merge queue made landing cheap.
- **Land continuously; never batch.** A branch is enqueued the moment CI
  passes. **Holding finished work for a combined evening merge is
  forbidden.** It does not avoid serialising — the queue still builds
  each candidate in turn — it concentrates every conflict into the hour
  with the least time left to resolve them, and conflicts compound with
  the age of a branch. Measured on 2026-10-02: a branch carrying another
  task's status edits went `DIRTY` three times, once per task that landed
  beneath it; and two same-day branches collided where one deleted a
  field the other set, caught only because both were fresh enough to
  grep. **A gate is also a statement about a tree, and a tree goes stale:**
  a verdict from the morning describes something the queue will not
  build. Landing fast is additionally how a lane finds its *own*
  mistakes — the worst fault of 2026-10-02 was already in `main`, and
  every hour of batching is an hour of building on it.
- **Opening a second branch does not transfer the first:** ownership of
  an open pull request lasts until it merges or is deliberately closed,
  and its failed checks and review requests come before new work on the
  second. Record dependencies explicitly; a blocked task does not block
  the machine, and where a question can be settled by authorised
  investigation, settle it rather than wait.
  `docs/autonomy/SHARED_MAIN.md` carries the limits and the order.
- **One language: Rust.** The workplace above is Rust too, in its
  own repository. Here, a language that is not Rust is a bug.
  Pinned engines are the exception, and an engine is configured,
  never written in. **This rule called it *the workspace* and said
  it was TypeScript until 2026-10-17**, and both had stopped being
  true: `alo-workplace` is 27 MB of Rust, and *workspace* in this
  repository means Cargo's crate set or the canvas a person's
  windows sit on. It is corrected rather than deleted because the
  false clause cost an answer — asked whether our own applications
  should be web wrappers or native, a lane read this line and
  reasoned from it, when they are native already (ADR 0122).
- **Engines are configured, never patched.** Linux, Mesa, systemd,
  the model runtime and the fine-tuning stack run as pinned upstream
  components behind our own interfaces. A source patch to any of them
  requires an ADR first.
- **The person grants; the agent never assumes.** No path, window,
  application or device is reachable that a person has not granted.
  Grants are enumerated, visible where the person can find them,
  revocable, and they expire. There is no grant to `/`.
- **Context is offered, never watched.** The focused window, the
  selection and the open document reach an agent only at the moment
  of invocation, and only for that turn. A background reader is a bug
  in this product, not a feature request.
- **Reads answer, changes wait.** A read runs inside the turn. Any
  change to the machine is proposed with a sentence describing it and
  waits for one approval. What a person approves is that sentence.
  **An approval is not a session at the default setting**, and a person
  may choose otherwise:
  [ADR 0064](docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)'s
  third level is *the agent runs code directly on the machine, without
  asking*, switched on for one project or for everything and off again
  whenever they like. **This clause read "an approval is never a
  session" until 2026-10-14**, which 0064 had contradicted since the day
  it was accepted. At every level the run is recorded, the files are
  snapshotted before it, and what leaves is shown — because none of
  those three takes a choice away.
- **Contracts outlive code.** The agent verbs, the application-adapter
  SDK, D-Bus interfaces, config keys, the image format and the update
  channel are public surfaces. Third parties build adapters against
  ours; they change additively, and a break requires versioning and
  deprecation.
- **The network is not authority.** Machines discover each other with no
  configuration and trust none of them for it. Using another machine
  takes a deliberate pairing on both; an agent reaching across acts
  only under a grant made on the machine it acts upon. **No
  trusted-network setting is on by default** (ADR 0003), and a person
  may opt in on their own network, told in the sentence that switches it
  on that it also trusts every device on that network. **This clause
  read "there is no trusted-network setting, and there will not be one"
  until 2026-10-14**, while `docs/features.md` had been saying *a person
  may opt in* — so the constitution and the feature list disagreed, and
  the constitution was the one promising something nobody had decided.
- **Whose machine it is, is answerable in ten seconds.** A machine is
  personal, or it is managed by an organisation that sets policy and
  holds a recovery key — and on a managed machine the person is told
  so at first sign-in. There is no silent enrollment, and no
  administrator can watch a screen or act as a person (ADR 0004).
- **Measured before claimed.** *"Supports PCs" is not a claim we make* —
  that half stands. What changed on 2026-10-12 is what finishes a piece
  of work: **it works on a PC we can put our hands on, and the report
  says which one** ([ADR
  0104](docs/decisions/0104-any-pc-that-works-is-done-and-no-gpu-is-required.md)).
  This read *certified before compatible* and had stopped selecting for
  completeness: every unmeasured thing on this machine was blocked on a
  certified machine, and the certified machine was blocked on
  certification nobody was doing. A list of models is a promise about
  machines we have not got; a measurement names the one we had.
  **And no graphics card is required**, which is a constraint rather
  than a relaxation: the compositor's own software rendering has to be
  good enough on a machine with nothing else.
- **Settled decisions live in `docs/decisions/`.** Read the ADR before
  proposing an alternative; relitigating without new facts wastes the
  scarcest resource we have.
- **Scope is gated, and the gate is the list rather than a tier.**
  Nothing gets built that isn't in `docs/features.md` and outside
  Non-goals. **There are no version tiers**: the owner removed them on
  2026-10-12 ([ADR
  0105](docs/decisions/0105-there-are-no-versions-until-there-is-a-launch.md))
  because they had stopped ordering the work and started blocking it —
  the thing a person most needed was marked for a release that did not
  exist, and three separate pieces of this product were refused on that
  ground in one afternoon.
  What the gate still does is the part that was earning its keep:
  **somebody decided this**. A feature nobody wrote down is still not
  built, and Non-goals still forbid. What it no longer does is tell a
  lane to wait.
  **Versions come back when there is something to version.** Until a
  launch, `ROADMAP.md` carries the one list that matters: what stands
  between today and a machine somebody outside this team can use.
- **Work a task needs is part of that task.** If finishing something
  already in scope turns out to require sub-work nobody listed — a
  caller that does not exist, a gesture wired to the wrong mechanism, a
  module with no consumer, a test that never ran — **build it, in the
  same change, and say in the record that you did.** Do not file it for
  later, do not hand it to another lane, and above all do not let the
  parent task read as closeable without it. *A plan lists what somebody
  foresaw; a task is done when it works.*
  **This does not loosen the gate above and must not be read as doing
  so.** One question separates them: *does this add a promise to a
  person, or make an existing promise true?* A classifier nothing calls,
  a raster nothing draws, a gesture that hides a window where the
  promise says it goes in the panel — those make a promise true, and
  they are yours to build without asking. Full screen, a four-edge dock,
  selecting several frames at once — those *are* promises, and they need
  a line in `docs/features.md` with a tier before one line of them is
  written. **When the answer is not obvious, it is scope**: the cost of
  asking is a message, and the cost of guessing wrong is a release
  nobody decided.
  **And the change says which answer it got, in one sentence.** Not the
  result, the judgement: *this makes an existing promise true, because
  X*. A reader tomorrow can then disagree with the reasoning rather than
  only discover the outcome. This clause exists because the sentence
  above is the one under most pressure when nobody is awake to ask, and
  the failure mode is not a lane building something forbidden — it is a
  lane finding *obvious* easier to reach than *ask*, because asking
  costs a night. *Proposed by the Mac lane on 2026-09-30, before three
  lanes ran unsupervised on the rule for eight hours.*
  **Sub-work you cannot do is reported the minute it is found**, not at
  the end. A task blocked on something outside its own crate is blocked
  from that minute, and other lanes may be resting on it — a missing
  full screen in the shell blocked five clauses across two lanes for a
  day because it was written down nowhere. *Added 2026-09-30, after a
  plan's eight tasks were all model-complete, two of them marked done,
  and nothing any of them described had reached a screen.*
- **The design is followed, not approximated.** What the interface
  looks like is settled in the alo OS design file; `docs/design/`
  records what was measured off it and what the owner decided where it
  is silent or disagrees with itself. Build to that. Where code and
  design differ the design is right until an ADR says otherwise, and a
  deliberate departure is written down where the next person looks
  rather than left in a diff. **A screenshot is not a specification** —
  take the numbers off the frames and record them, because a layout
  matched by eye is a layout nobody can check.
- **Nothing is built to one screen size.** The design's frames are
  1440×960 and no machine is obliged to be. Every layout is derived
  from the display it is on — its size, its scale, its orientation —
  so a figure taken from a frame becomes a proportion or a named rule,
  never a constant that holds only at the size somebody drew. Screens
  run from the smallest this product lays out for up to an external
  display beside the laptop, and density is separate from size: twice
  the pixels draws the same interface sharper, not half as big. A
  control that falls off a small panel, or swims on a large one, is a
  bug and not a tuning problem.
  **What this forbids is a raw pixel figure, not a named measure.**
  `alo_dock::measures::ICON` is 48 and breaks nothing: it is a *logical*
  measure, declared once with what it answers to, and converted for each
  display through that display's scale. So there are three honest kinds
  of number — a **logical measure**, put in a measures file and scaled
  at the boundary; a **proportion** of the display; and a **rule** that
  was never a number at all, like *flush to the edge* or *the whole
  edge*. There is a fourth kind, and it is the one to watch: a bound
  that is **another surface's current extent**, which is neither a
  measure nor a proportion and can only be asked for. The fault this
  rule names is a figure that reaches a display without passing through
  its scale — and the usual way it arrives is scaled by the person's
  text size *only*, which looks derived and is half the size it should
  be on a dense screen.
- **User-facing strings are externalized (i18n) from day one.** The
  first target is all 24 official EU languages, and any language
  somebody contributes after that. Hardcoded English is a bug, and
  "English plus the big five" is the same bug wearing a business case:
  it serves some people and not others, and the ones it skips are those
  with the least software in their own language already.
- **Built in Europe, not only for Europe.** Where alo OS lets an
  organisation set a rule — which region inference may happen in, which
  providers are permitted — the rule is theirs to name. We ship the
  mechanism, never a default that decides for them.
- **Describe the work, not its queue code.** New task titles, report filenames,
  commit subjects and status updates use professional, descriptive names such
  as "Secure file moves" or "Native desktop compositor". Legacy queue codes may
  appear only as secondary cross-references; preserve historical links and ADR
  identifiers. Use lowercase hyphenated report filenames such as
  `secure-file-moves.md`, never a code-only name.
- **Progress documents have one writer.** Follow
  `docs/autonomy/SHARED_MAIN.md`: parallel contributors publish separate task
  reports; only the integration owner edits shared progress documents. This
  supersedes older loop instructions requiring every contributor to edit them.
- **Names are for strangers:** files, commit subjects and branches
  describe the subject matter. Release codes live in `ROADMAP.md` and
  commit trailers. Commit subjects follow conventional style —
  `type(scope): descriptive subject`.
- **One agent per working tree.** Concurrent editors on one checkout
  are forbidden. The canonical checkout lives outside any file-sync
  folder — git and the remote are the only sync. **One agent may hold
  several worktrees**, which is how a second task proceeds without
  disturbing the first: separate branches checked out in separate
  directories, one editor in each. What they do not multiply is the
  machine — they share its memory, CPU and disk, so **one Cargo
  operation per machine** stands, and they do not multiply the agent,
  so one task is actively implemented at a time.
- **Say it in words the person can act on, and say what it changes for them.**
  Every report, status, summary and answer to a person outside the code is
  written in plain English and names **what it means for the person using the
  machine, or for the machine itself**. A technical name may be used once its
  meaning has been given; a report that assumes the reader already knows the
  vocabulary is a report they cannot check.

  **Naming the effect is the part that is usually skipped.** *A frame is drawn
  per display* says nothing to anybody; *each monitor gets its own picture, and
  one monitor failing no longer blanks the other* can be agreed or disagreed
  with. Where the effect is **nothing yet** — code written and not reachable, a
  decision taken and not built — say that outright, because *done* and *working*
  read identically in a status and are not the same claim.

  This binds hardest in the places it is least convenient: a task report, a
  commit body a person may read a year later, and any answer to *how is it
  going*. It is not a request to simplify the engineering; the ADRs, plans and
  tests stay as precise as they are. It is a request that **the account given to
  a person is one they can act on without reading the code.**

  *Asked for by the owner on 2026-10-08, in these words: "explain to me in user
  friendly terms and simple English and include how it will impact the users or
  system … I do not understand them and that as a rule too" — after a day of
  reports written in this repository's own vocabulary to somebody who had not
  asked to learn it.*

- **State your own errors, and record the ones that could be repeated.**
  Say it in the moment, to whoever is relying on the claim — plainly, once,
  without ceremony — and then put the ones somebody else could walk into in
  `docs/misreadings/`, one file per entry, in that directory's four-part
  shape: what was concluded, what was true, **the mechanism**, the cure.
  The mechanism is the part worth writing; the rest is an example of it.

  **This is not a confession log and it is not a backlog.** An entry earns its
  place by saving somebody else the same hour, so an entry that cannot say
  what to do differently is a slogan and does not belong — and a slip that
  changes nothing for anybody is corrected and forgotten rather than
  memorialised. Nothing here is about blame: the lane that writes an entry is
  almost always the one that found it, which is the behaviour this rule
  exists to make ordinary rather than brave.

  **It binds hardest where the error was invisible.** A failure somebody else
  can see is already reported by the thing that failed. What this rule is for
  is the fault that *passed* — a check that measured a narrower thing than its
  sentence claimed, a number that had gone stale, a note attached to something
  that cannot fail — because those are the ones that cost a week later and
  cost a paragraph now. **Say which of the two it was**, so a reader knows
  whether anything caught it.

  *Written 2026-10-05 by the owner. Twenty-two entries existed before this
  rule did, and `docs/misreadings/` was named nowhere in this file — the
  practice was real, undocumented, and depended on whoever happened to have
  the habit.*

- **The author of every commit is the repository's owner**, from the
  checkout's own `user.name` and `user.email`. **No agent sets an
  author of its own** — not with `--author`, not with `-c user.name`,
  not through `GIT_AUTHOR_*`. This rule previously said the opposite,
  and the loop obeyed it: a run of commits landed authored by "alo
  build loop", which is not a person and is not who owns this work.
- **No agent adds a `Co-Authored-By` trailer either.** This rule also
  said the opposite until 2026-09-04 — an agent was to be named where
  a contributor belongs — and every commit in this repository carried
  one. The owner asked for the work to stand as theirs, and the
  history was rewritten to match: 119 commits, not one byte of any
  tree changed. **What an agent did belongs in the commit body**,
  which is where it was always the more use to a reader — a trailer
  records that an agent was present, a body records what it decided
  and why, and only one of those is worth reading a year later.
  Ambiguity between agents is solved by one agent per tree, never by
  inventing an author and no longer by a trailer.

## How work reaches `main`, and what is forbidden because it is slow

*Written 2026-10-01, from a day that landed ten changes before 07:33 and
five in the thirteen hours after. Every rule below names the cost that
produced it, because a rule whose reason is not written down is one the
next lane relitigates.*

**Required:**

- **Push before you gate, and gate once.** A push is not a landing: the
  merge is the scarce thing and a branch on the remote costs nobody
  anything. A gate on an unpublished branch cannot prove the tree it
  measured is the tree anyone else can fetch.
- **Nothing stays on a local disk. Push first, then gate.** *The owner,
  2026-10-14: "please always merge the code to the remote not on local."*
  Work that is only on a local disk **does not exist** as far as every
  other lane is concerned, and the two failures that follow from it are
  opposite and both expensive. A lane that reads absence as abandonment
  **takes a crate that has an owner** — the fault the lane table exists
  to prevent, twice in one day. And a lane that reads absence correctly
  **starts the same work again**: on 2026-10-14 a regression deleted six
  files and about 480 lines, one lane began restoring it and gated
  locally for the better part of an hour, and the other was one check
  away from starting the identical cherry-pick. Two lanes on one
  conflict, because a finished local fix is indistinguishable from
  nothing at all.
  **So the order is push, then gate, and never the reverse** — and the
  push happens the moment there is a commit, not when the work feels
  presentable. There is no state of a branch too rough to publish: a
  branch on the remote costs nobody anything, it is not a landing, and
  it cannot be reviewed, found, taken over or finished by anybody else
  until it is there. **A verdict on an unpushed tree is also a verdict
  about a tree nobody can fetch**, which is the separate reason the
  rule above exists.
  **And when somebody asks what the state of something is, the answer
  is what the remote holds**, never what a working copy holds. *It is
  fixed on my machine* is not an answer to *is it fixed*; the question
  is about `main`, and the honest reply names what `origin` shows.
- **Every branch has a pull request, and every pull request says what it
  is.** *The owner's instruction, 2026-10-22.* Two halves of one rule,
  because each without the other leaves work nobody can act on.

  **A branch with no pull request is work nobody can see.** It does not
  gate, it does not queue, it does not land, and it is invisible to
  everyone but the lane that pushed it — which is the same fault
  *nothing stays on a local disk* names, one step further along. Pushing
  made it fetchable; only a pull request makes it **reviewable,
  mergeable and finishable by somebody else.**

  **Measured on 2026-10-22**, the day this was written, and every figure
  below can be retaken. Five branches on the remote, three with no pull
  request. One had been sitting for **two days** — pushed on 2026-10-20,
  a whole new crate, 10 files and 2,440 insertions, which is GitHub's own
  figure for it and anybody can ask again — and it could not have landed
  even if somebody had found it, because nothing anywhere in the
  workspace names the crate it adds. One carried a finished feature — every installed application's
  own icon on the Dock — that sat gated and green for **six hours** with
  no door. The third was this rule, which sat pushed and unopened until
  another lane opened it on the writer's behalf. And the whole evening's
  throughput ran through one person opening pull requests by hand, four
  times, because the lane that wrote them could not. *Work finished and
  sitting still was the single largest cost of that day, and not one
  line of it was a hard problem.*

  *An earlier draft of this paragraph cited one of those branches as
  carrying "198 lines". No ref that still exists matches that figure —
  the branch had landed and been deleted — so the claim could not be
  retaken by anybody reading it, which is what **measured before
  claimed** forbids, in the document that forbids it. Replacing it took
  two tries, and the second try was wrong in a more interesting way than
  the first, so both are recorded: **a line count has two independent
  ways to be false, and they are easy to mistake for each other.***

  - *A **stale baseline**. A diff against a local `main` eight commits
    behind read 10,442 insertions where `origin/main` gives 3,974.
    Another lane hit it the same day at 247 commits behind, where one
    changed file read as 3,945 lines across 53. The squash queue means a
    stale `main` still lists commits normally, so nothing looks wrong.
    **Diff against `origin/main`, never `main`.***
  - *A **two-dot diff across an old merge-base**. `origin/main..branch`
    answers *what changes if the tree becomes the branch*, so it also
    counts undoing everything `origin/main` gained since the base. For
    the two-day branch above, whose base is 45 commits back, that reads
    195 files and 24,528 deletions — and this draft published it as
    "merging it would remove 24.5k lines". **No merge does that.** The
    squash is computed from the merge-base, GitHub reports `+2440 −0`,
    and the three-dot diff agrees to the line. The lane that caught it
    was the one the wrong figure was aimed at.*

  *The two are distinguishable without measuring anything. Three-dot
  **is** `merge-base..branch`, so where the merge-base is `origin/main`
  the two forms are the same command and must agree. **A gap between
  them is therefore a statement about the base's age and nothing else** —
  true by definition, and it will never decay.*

  *This draft said it the other way round, as a figure: "measured at 37
  files and 3,974 insertions either way". **It was false forty minutes
  later**, when one pull request landed and that branch's base went one
  commit behind — 45 files and +4,005/−400 against 37 and +3,974/−10.
  Any landing ends such a condition, so the honest form of that sentence
  had a half-life of one merge. It is the family **Forbidden** already
  names as publishing a measurement as a standing property, and it
  happened in the sentence explaining how line counts go wrong.*

  *Which gives the generalisation worth more than any of the three
  figures. What made GitHub's `+2440 −0` usable was not that it was
  small or that it was right: it was **recomputed on read**, from the
  merge-base, every time anybody asks. The "198 lines" depended on a
  tree that no longer existed and the "either way" on a tree state that
  lasted under an hour. **Cite a figure a reader can re-derive from a
  live source; a figure that describes a tree is a figure with an expiry
  date nobody wrote on it.** The corrections are all recorded rather
  than made silently, because a constitution that quietly repairs its
  own evidence teaches the opposite of what this clause asks for.*

  **The pull request is opened when the branch is pushed**, which is the
  moment there is a commit — not when the work feels presentable. A
  draft is a pull request. An empty one with a title is a pull request.
  What is forbidden is a branch on the remote with nothing pointing at
  it.

  **And it says what it is.** A pull request whose body is *No
  description provided* hands a reviewer a diff and asks them to
  reconstruct the argument from it. The commit bodies carry the
  reasoning — that rule is above — but a reader opening twelve commits
  cold has no map, and the one thing they most need is the thing a diff
  cannot show: **what a person could not do before, and can now.** Three
  lines is enough. The same day, both of this lane's own pull requests
  said *No description provided*, which is why this clause is here and
  not left as a courtesy.

  **Nothing checks this, and saying so is the point.** The merge queue
  enforces the first half at the last possible moment — a branch cannot
  land without a pull request — and that is exactly where enforcement is
  worth nothing, because the cost is paid in the hours before. No gate
  reads the remote's branch list, and no gate can read a body and judge
  whether it says anything. So this is a discipline rather than a test,
  and under *say whether a measurement was performed once or runs
  forever* it is neither: **it runs when somebody chooses to follow it.**
  A lane that wants it caught should ask the owner for a check that
  lists branches with no pull request. **It is one API call, and it has
  been run by hand**: on the day this was written, one GraphQL query
  over `refs/heads/` with `associatedPullRequests` found both branches
  that were still open gaps. It also returned **two standing false
  positives** — the default branch and a `gh-readonly-queue/*` ref,
  neither of which can have a pull request — so a check built from it
  excludes those two by prefix. That exclusion is the whole design
  problem: *a check with permanent false positives is one lanes learn to
  skip*, and a skipped check is worse than none, because it is counted.
  Nothing in this repository runs it yet, and this sentence is not
  permission to add it — apparatus on the landing path is the owner's
  call.

- **Stack your own dependent work.** Branch off your own branch rather
  than waiting for it to land. Waiting for your own pull request is
  serialisation a lane does to itself.
- **A change to a signature and the call sites it breaks land
  together.** Split across two pull requests, `main` does not compile
  between the two merges, whichever lands first. **This overrides crate
  ownership**: it is not a question about who owns a file, it is a
  question about whether `main` builds, and *work a task needs is part
  of that task* never had to reach it.
- **Run the cheap checks before spending a gate**, and have that
  pre-check **name what it does not run**. Formatting, clippy over the
  **whole workspace**, and rustdoc with warnings denied cost four
  minutes and catch what otherwise costs a full gate to discover. An
  unlisted omission is how *the cheap checks passed* becomes a claim
  somebody leans on.
- **Delete a landed branch by asking whether its pull request merged.**
  The queue squash-merges, so a landed branch is never reachable from
  `main` and `git branch --merged` reports nothing. Fifty-six local
  branches, two detected.
- **A report names the fields that would have to agree, and names their
  disagreement as its own outcome.** *A single number cannot be caught
  being wrong; three that must be consistent can.* One lane's report
  read `EXIT=0` beside four link errors, because nested quoting was
  eaten at a shell boundary and the exit code it echoed was not the exit
  code of what it measured — and it was caught **because** the fields
  contradicted each other. Another put an inline `$(git rev-parse HEAD)`
  in a line whose whole purpose was confirming which commit it was on,
  and it printed **another repository's** sha: the check and the fault
  were the same line, and nothing contradicted anything. **The second is
  worse.** A verdict that agrees with itself while being wrong is
  invisible; one whose own fields disagree announces itself.
- **Before diagnosing why something did not happen, check that it was
  asked for.** A branch pushed at 07:37Z was gated four times — 57, 61,
  89 and 97 minutes — a spread measured, a cold cache diagnosed,
  `CARGO_BUILD_JOBS` checked, 3.9 GB of RAM and two of six cores found,
  a rustdoc breakdown produced, a ninety-minute hold asked for and
  granted, an empty queue closed for two hours twenty, and a lane
  declared structurally unable to land. **No pull request had ever been
  opened.** Every number was real and not one of them was the reason.
  This is neither a check that could not fail nor a measurement reported
  as a property: it is **an elaborate correct diagnosis with a missing
  step underneath that nobody was looking at**, and it was found from
  outside in a single query.
- **A gate states which subject it answered.** The status on a pull
  request's head and the status on the queue commit are two different
  claims: *these gates passed on this tree*, which stays true whatever
  `main` does, and *these gates passed on the tree that will become
  `main`*, which does not. Protection is `strict: false` precisely
  because the queue builds its own commit, so `main is unmoved` is
  load-bearing for the second and over-strict for the first. **A head
  status must not read as *cleared to land*** — it names the machine and
  the suite, so a green head beside an absent queue status is legible
  rather than reassuring. And where a flag chooses between the strict
  and the loose reading, **the strict one is the default**: a flag that
  fails open is a flag that will be forgotten in the direction that
  matters.

**Forbidden:**

- **Editing a checkout while a gate owns it.** The gate passes and
  refuses to attest, and what is lost is a real nine-of-nine verdict
  about a tree nobody will land. A lock plus a pre-edit hook, not
  resolve: *deliberately not touching the tree* was said twice in one
  evening by the lane that then touched it.
- **Gating a branch whose local head is not the remote's.** A gater
  that resets to `origin` destroys a committed-but-unpushed rebase —
  and a clean tree hides it, because a clean tree is exactly what an
  unpushed commit leaves. It then gates the old tree and **passes.**
- **Asking another lane to hold the queue as routine.** A hold is for a
  measured and named reason, and never twice for the same cause: a lane
  that needs one per landing is an argument for changing the check, not
  a protocol. *A hold nobody asked for is not a hold, it is a stall.*
- **Hand-typing a status that CI can produce.** Every manual step in
  the landing path serialises every machine behind one turn. Eleven
  faults were found across three lanes on 2026-10-01 and **not one was
  in the product** — all of them were in the apparatus that exists to
  make a hand-typed status trustworthy.
- **Publishing a measurement as a standing property.** Say what was
  measured and when. *Twelve consecutive green runs* was true at 09:00
  and false by 19:00, after it had been written into an ADR as its
  central evidence.
- **Sending another lane a remedy instead of the measurement that
  produced it.** A fix that does not work is worse than no fix: it
  converts *untested hypothesis* into *we tried that*. One filesystem,
  measured on one machine, was offered as the cause for three.
- **Reporting a change as tested on a subset of the gates.** Three of
  nine run and called tested; four crates measured and three checked.
  **In both cases the narrower measurement was correct**, which is what
  makes this family invisible: *a scope that lives in the method and not
  in the sentence is a scope the reader cannot check.*

## Map

- `README.md` — what alo OS is, and what it is not.
- `docs/features.md` — the only list of what gets built.
- `ROADMAP.md` — the only order it gets built in, with exit gates.
- `docs/decisions/` — the ADRs. Start with 0001; it is the one an
  outside contributor must read before touching `alo-agentd`.
- `docs/contracts/` — the agent verbs and the adapter SDK: what other
  people build against.
- `docs/hardware.md` — the certified list, honestly maintained.
- `docs/booting.md` — how the image becomes a disk a machine boots from,
  and what a virtual machine can never show about one.
- `docs/quirks.md` — where reality and the specification disagree:
  driver behaviour, application automation, firmware. **New quirks go in
  `docs/quirks/`**, one file per entry; the old file keeps the 203 already
  written because 187 files cite its path.
- `docs/misreadings/` — where **we** fooled ourselves, as `docs/quirks/` is
  where somebody else's software did. One file per entry, and its `README.md`
  holds the shape: what was concluded, what was true, the mechanism, the cure.
- `SECURITY.md` — how to report something, and what is in scope.
- `CHANGELOG.md` — what changed, in words a person outside this
  repository can read.
- `docs/autonomy/` — the build loop: `LOOP.md` is how one iteration
  works, `QUEUE.md` is the work and what each item is blocked on,
  `STATE.md` is the journal. The loop never weakens the gate to pass
  it, and never ticks what it did not finish.
