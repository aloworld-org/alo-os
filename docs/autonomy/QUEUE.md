# Queue — v0.01

Worked in order by `LOOP.md`. Every item names what it implements, so an
iteration can read the reasoning rather than guess at it.

## The honest constraint

**This loop cannot finish v0.01, and no amount of iterations will change that.**
Roughly half of what v0.01 needs ends on hardware and an operating system this
loop does not have:

- **A Wayland compositor, the portals and the bootable image are Linux.** They
  will not compile, let alone run, on the machine the loop runs on today.
- **The exit gate is the certified machine**, verified by somebody sitting in
  front of it, once. No test suite substitutes for that.
- **"Agents point at the local model" belongs to `alo-workplace`.** Different
  repository, and this loop never touches another one.

What the loop *can* finish is the part everything else stands on: the capability
model as working code. Grants, verbs, approvals and the record are pure logic —
portable, exhaustively testable, and the thing that makes `alo-agentd` a service
worth writing rather than a daemon with a permission dialogue. Doing that first
is also the right order: the Linux work becomes an implementation of a settled
model instead of a place where the model gets decided by accident.

Items are **ready** (buildable and testable here), **linux** (needs a Linux
host), or **hardware** (needs the certified machine). The loop takes ready items
and stops when there are none left.

---

## Already built, outside the loop

`crates/alo-models` — read it before starting item 1, because it sets the house
style the rest should match, and two of its decisions constrain later items.
`crates/alo-capability`, `crates/alo-record`, `crates/alo-egress`,
`crates/alo-files`, `crates/alo-keeping`, `crates/alo-shortcuts`,
`crates/alo-appearance` and `crates/alo-strings` were built by the loop and are
described in the items below. The first four depend on each other in one
direction only: `alo-capability` decides and reaches nothing, `alo-egress`
decides about what leaves, `alo-files` is the only one an agent can reach that
touches a disk, and `alo-record` observes them and is reachable from none of
them. `alo-keeping` reaches `alo-record` and puts it on a disk; nothing reaches
back, which is what keeps *nothing takes an entry out* true of `alo-record`
while something, somewhere, can shorten a record. The last three depend on
nothing in this workspace at all, because a person pressing a key on their own
machine, choosing their own wallpaper or reading their own machine in their own
language is not an agent doing something and needs no grant.

Seven edges cross that: **`alo-files` depends on `alo-strings`** since item 9b,
**`alo-shortcuts` does** since 9c, **`alo-appearance` does** since 9d,
**`alo-capability` and `alo-record` do** since 9e — the deciding crate because
every refusal it makes is read by somebody, and the record because it writes
down the words that person was shown rather than a second rendering of its own —
**`alo-models` does** since 9f, because where an answer came from is a
sentence somebody reads before they decide what to send, and **`alo-egress`
does** since 9h — the last to cross, and the one whose sentence a person reads
while it is happening. `alo-strings` still depends on nothing, and the direction
is the one every other edge here takes — a crate that says something reaches the
crate that knows how things are said, never the reverse. Every crate in this
workspace has now crossed it, so *hardcoded English is a bug* is a rule with no
exceptions left in it rather than a rule with a list. **`alo-keeping` is the
first crate that never had to cross**: it was written after the 9-series, so it
has never held an English sentence and no type in it has ever had a `Display`.
That is what the rule looks like once it is finished being applied.

Since **item 9g** the edge is load-bearing rather than incidental: a verb is
*declared* from `alo_strings::Word`s, so `alo-capability` cannot express a
capability whose words are not translatable, and `alo-record` renders the
sentence a person approved rather than copying one. `alo-files` lost a file to
it — `saying.rs` — which is what it looks like when two things that said the
same thing become one.

| | |
|---|---|
| `catalogue.rs` | What alo OS offers, every licence stated, commercial use answered outright; the CPU costs and defaults from ADR 0007 |
| `runtime.rs` | `ModelRuntime` — what alo OS asks of a runtime, in our words |
| `ollama.rs` | The adapter, and the only file that knows Ollama exists (ADR 0006) |
| `source.rs` | Where a question is answered and what that costs in egress (ADR 0008); the region policy an organisation names |
| `provider.rs` | Providers somebody adds themselves; the key lives in the keyring, never in the settings |
| `secret.rs`, `tried.rs`, `trying.rs` | Added later by the loop — testing a provider before it is saved (item 10) |
| `words.rs`, `refusing.rs` | Added later by the loop — everything this crate says, and which rule refused a question (item 9f) |

**44 tests when the loop started, 90 now, clippy clean against the workspace
deny list.** Two patterns later items must follow:

- **A promise in `docs/` is a test, not a sentence.** "Every model states its
  licence" and "a paired machine is egress too" are tests. Anything an item
  claims should be one.
- **Errors say what to do, not what went wrong.** `provider.rs` is the reference:
  *"give the provider a name — it is what you will see when it answers"*.

---

## Ready

- [x] **1. Grants** — implements ADR 0001 §3 and `docs/contracts/agent-verbs.md`.
  `crates/alo-capability`: `reach.rs` (what a grant covers, what a verb asks
  for), `grant.rs` (one grant and what has to be true of it), `grants.rs` (the
  list, its queries and its refusals), `path.rs` (containment, decided
  lexically). 28 tests, clippy clean. **This is the vocabulary items 2–5 speak**
  — `Grantee`, `Reach`, `Ask`, `Grant`, `Grants`, `GrantId` — and they belong in
  this crate beside it rather than in one of their own.

  Three decisions the next items inherit. **Nothing reads the clock:** every
  question that depends on time takes `now`, so expiry is testable without
  sleeping and the daemon and the settings panel cannot disagree about the
  moment. **Identities are matched exactly** — agent names, application ids and
  paths — because matching loosely matches more than the person picked.
  **Containment is lexical and touches no disk**, so whatever executes a verb
  must resolve symbolic links *before* asking whether a path is granted; the
  reasoning is in `path.rs` and item 6 is where it gets honoured.

  Storage is serde, as with `Providers` — where the list is written and when is
  the daemon's, and it does not exist yet.

- [x] **2. The verb registry** — implements `docs/contracts/agent-verbs.md` and
  law 2. Five files in `crates/alo-capability`: `arg.rs` (what an argument can
  be, and what survives arriving), `sentence.rs` (the approval sentence and
  where its words come from), `verb.rs` (everything that has to be true of a
  verb), `verbs.rs` (the closed list), `call.rs` (a validated call, what it
  would touch, and whether the grants permit it). 37 tests, clippy clean.

  **The registry cannot hold a verb that breaks the contract, and it manages
  that by not being able to receive one:** `Verb::checked` is the only
  constructor and there is no `Deserialize` around it. Law 2 is carried by
  `Takes` being a closed list with no free-text kind in it — a model cannot
  compose something to run because there is no shape for it to arrive in — plus
  a declaration-time refusal of names that announce an interpreter, which is a
  tripwire and is documented as one. A verb's implementation passing an argument
  to a shell is still only stopped by review, and `verb.rs` says so.

  Three items inherit decisions from this. **Every argument is required** — an
  optional one makes the sentence conditional. **The sentence names every
  argument**, or the verb is refused. **Being permitted and being approved are
  separate**: `call.rs` answers the first and deliberately does not answer the
  second, which is item 3's.

- [x] **3. Approvals** — implements ADR 0001 §5. Four files in
  `crates/alo-capability`: `proposal.rs` (a change put to a person, and what is
  never put to one), `approvals.rs` (the list waiting to be answered, and the
  answering), `approval.rs` (one answer, worth one execution), `authorised.rs`
  (the only type meaning may-run, and both doors into it). 24 tests and two
  doctests, clippy clean.

  A change now travels `Call` → `Proposal` → `Approved` → `Authorised`, and
  each step can only be reached from the one before. **One approval, one
  execution** is carried twice over: the list refuses a proposal answered twice,
  and `Approved::redeem` takes `self`, so a second execution is not a program
  that compiles — which is what the `compile_fail` doctest on `redeem` asserts,
  with a twin that passes so the pair cannot rot into a test of a typo. The
  arguments travel inside the approval and there is no accessor lending them
  out, so an approval for one argument set cannot authorise another.

  Three decisions the next items inherit. **The grants are asked last at the
  moment of execution**, which is where a revoked grant becomes immediate; being
  permitted is checked when the call is made, again when it is proposed, and
  again there. **A read is never proposed and a change never takes the read
  door**, so ADR 0001 §5 is two refusals rather than a convention. **A refusal
  carries what it refused** (`Refused::call`), because item 4 records refusals
  and one that threw away the call could only say that something was stopped.

- [x] **4. The record** — implements ADR 0001 §7. Seven files in
  `crates/alo-record`, a **new crate** rather than more of `alo-capability`:
  `line.rs` (text as the record keeps it), `written.rs` (one argument),
  `what.rs` (what a call was), `happened.rs` (the four things that can happen),
  `entry.rs` (one moment and what happened at it), `record.rs` (the list),
  `explain.rs` (what can be asked). 42 tests, clippy clean.

  **Why a crate of its own**, against what `alo-capability`'s docs previously
  said. The deciding crate must not deserialise — `Call`, `Value` and `Proposal`
  serialise and deliberately do not read back — while a record exists to be read
  back, so it needs its own types either way. And the record needs
  `InferenceSource` from `alo-models`, which would have put an HTTP client and a
  TLS stack behind the crate whose whole value is being small enough to audit.
  `alo-capability` still depends on nothing but serde and thiserror.

  Three decisions the next items inherit. **`Grants::permitting` answers with
  the grant rather than with a yes**, and `permits`/`refusal` are now two halves
  of that one search — so nothing can be permitted by a grant the record cannot
  name, and `Authorised::against` carries it. **A record is evidence, not an
  instruction**: nothing in `alo-record` turns back into a call, and nothing
  takes an entry out. **Two things are never kept** — the question a person
  asked, and the arguments of a call that never validated.

- [x] **4a. Where the record is written, and what prunes it** — cut from item 4,
  and listed here as the daemon's until this iteration read it again. Two of the
  three things it names are portable and testable on any machine: **the file it
  is written to** and **the decision about how long evidence is kept**. Only
  *which* path and *when* a shortening runs are `alo-agentd`'s, and those are
  item 4b under *blocked — linux*. `crates/alo-keeping`, a **new crate**:
  `keeping.rs` (the rule), `head.rs` (the first line, and where the record
  starts), `writing.rs` (appending, one entry at a time), `reading.rs` (reading
  it back), `pruning.rs` (the only thing in alo OS that removes evidence),
  `damage.rs` (what could not be read), `failing.rs` (why there is no record to
  write to), `words.rs` (14 phrases and one countable string). 59 unit tests, 15
  integration tests against a real filesystem, 781 tests and 20 doctests across
  the workspace, clippy clean.

  **A new crate rather than more of `alo-record`, and the reason is the
  promise.** `alo-record` says nothing takes an entry out — no `remove`, no
  `edit`, no `forget` — and something has to be able to. In one crate that
  promise would be true of a type and false of the file list around it, which is
  how a security reviewer checks it. So the crate that *can* shorten a record is
  separate and small, and everything in it is about making that hard to do
  quietly: what goes is decided by a rule and a moment with no way to name an
  entry, shortening is a method on the writer so nothing that is not holding the
  record open can do it, and it refuses a record it cannot read all of rather
  than rewriting the evidence that something was wrong.

  **The decision the item did not contain: where the mark goes.** A record that
  has aged out its first six months and a machine that did nothing are the same
  short file, so the shortening has to leave one. An *entry* saying so was the
  obvious answer and is wrong — an entry has a moment, so the next shortening
  ages it out, and after two rounds the record looks untouched again. The mark
  is therefore the **first line**, which pruning never walks: a record says
  where it now starts, and no later shortening can take that back.

  Three decisions the next items inherit. **The record file is a public
  surface**, written down in `docs/contracts/record-file.md`: one line of JSON
  per entry, a format number in the first line, additive change only, and a
  record from a newer alo OS refused rather than appended to. **A missing record
  is not an empty one** — reading refuses it, because a deleted record answered
  as *nothing happened* is the failure this crate exists to prevent, and making
  one is a deliberate act by the daemon. **A window is measured from the epoch
  forwards, not from `now` backwards**: `SystemTime::checked_sub` walks into
  1969 on Windows and answers `None` elsewhere, so a wrong clock would remove
  different things on different machines. `docs/quirks.md` records that and the
  rename-over-an-open-handle it also found.

- [x] **5. Egress policy** — implements law 1 §8. Five files in
  `crates/alo-egress`, a **new crate**: `destination.rs` (where something is
  going, and what may be shown as an address), `leaving.rs` (one egress about to
  happen — who, where to, why), `policy.rs` (what an organisation permits, and
  the refusal in words), `departing.rs` (the only type meaning may-leave),
  `indicator.rs` (what is leaving right now). 27 tests and 3 doctests, clippy
  clean.

  **The decision and the indicator are one call**, and that is the whole design:
  `Indicator::beginning` asks the policy and shows the result, and it is the only
  constructor of `Departing` — the token a caller must hold before it opens a
  connection. *Permitted but not shown* is therefore not a state that exists,
  which is the guarantee `CLAUDE.md` names, carried by a type rather than by
  whoever writes the next verb remembering. Ending a departure takes `self`, so
  one connection can never take two lines off the indicator; the `compile_fail`
  doctest asserts it, with a passing twin.

  Three decisions the next items inherit. **The rule is stated once**:
  `EgressPolicy` is made `From<&SourcePolicy>` rather than written down again,
  and a test walks every policy against every source to prove the wider boundary
  and the inference one cannot disagree. **A question answered on this machine is
  not a departure** — `Leaving::asking` refuses it — so law 1's zero-egress claim
  is the absence of a type rather than a counter that happens to read zero.
  **Only a paired machine is in the building**: a host that answers on the same
  wire is outside it, because ADR 0003 refuses "it is only our internal network".

  Enforcement at the network boundary is Linux and is a later item; this crate
  guarantees the ordinary path, not code that never asked.

- [x] **5a. The record of what left** — implements law 1's second half in
  `alo-record`. One new file, `departed.rs` (the only door from an egress into
  the record), plus three new `Happened` variants and the query behind them. 12
  new tests and 3 new doctests, two of them `compile_fail`; 218 tests and 8
  doctests across the workspace, clippy clean.

  **The decision, which the item asked for before it asked for code.** Neither
  of the two options as written: `Answered` did not become provenance beside a
  departure, it became `AnsweredHere` and stopped having a source at all. A
  question answered somewhere else *is* the departure it caused, so it is
  `Happened::Left` and nothing beside it; a question answered here is
  `Happened::AnsweredHere`, which has nowhere to name and no field to name it
  in. Nothing is lost — `Destination` says everything `InferenceSource` said,
  and `alo-egress` already maps one to the other in one place — and two things
  are gained. `caused_egress` became a variant rather than a calculation, so no
  two readers can work it out differently. And the record can no longer
  contradict itself: saying *the answer came from a provider* now requires a
  `Departing`, which only the indicator makes.

  Three decisions the next items inherit. **An egress entry is unreachable
  except from a `Departing`, and a held-back one except from a `NotPermitted`**
  — both `pub(crate)`-constructed by the indicator, asserted by `compile_fail`
  doctests that were checked to fail on the privacy and not on a typo.
  **`Why` now deserialises**, additively, like `InferenceSource` gained
  `Serialize` in item 4; `Leaving` still does not. **`alo-record` depends on
  `alo-egress` and no longer on `alo-models`** except in tests — the observer
  reaches the decider, never the reverse.

- [x] **6. File verbs, the portable half** — implements `docs/features.md`'s
  v0.01 file verbs and the rule item 1 left for it. `crates/alo-files`, a **new
  crate**: `verbs.rs` (the six, declared), `real.rs` (the path this machine
  would really open), `resolving.rs` (the one thing here that touches a disk),
  `touching.rs` (the only type meaning *this may touch the disk*). 24 tests, 3
  integration tests against a real filesystem, 3 doctests, clippy clean.

  **The item exists for one sentence in item 1**: containment is lexical, so
  whatever executes a verb resolves the real path and asks about *that*.
  `Touching::of` takes an `Authorised` — the end of ADR 0001 §5's journey — and
  asks three questions of every path the call names, in this order: do the
  grants permit it as written; where does it really lead; do they permit that.
  A link out of a granted folder dies at the third, as a refusal by the grants,
  in their own words. The order is the security property: **a path nobody
  granted is refused before the disk is touched**, so a refusal cannot tell an
  agent whether a file it may not reach exists.

  Three decisions the next items inherit. **Every path a call names is asked
  about**, not only the ones the verb declared its grant is over — the test that
  proves it uses a verb whose author forgot one. **`Real` has no public
  constructor**, which seals `Resolving`: one implementation ships, so nothing
  can hand the grant check a real path it made up, and the crate's decision is
  still testable on a platform where making a symbolic link needs a privilege.
  **`Refused::not_granted` is new in `alo-capability`** — the one question that
  crate cannot ask itself comes back as the same type, so this refusal reaches
  the record by the road every other refusal takes.

- [x] **6a. File verbs, the acting half** — the `std::fs` calls behind the six,
  taking a `Touching` rather than a path. Nine files in `crates/alo-files`:
  `doing.rs` (the one door, and the last grant question), `answer.rs` (what each
  of the six answers with), `failed.rs` (why the machine could not), `named.rs`
  (one thing in a folder), `looking.rs` (the three reads), `changing.rs` (rename
  and move), `archiving.rs` (what goes into an archive), `walking.rs` (a folder,
  without walking out of it), `zip.rs` and `crc.rs` (the format). 64 unit tests,
  13 integration tests against a real filesystem, clippy clean.

  **The item asked for two things and found a third.** It opens what `Touching`
  resolved and resolves nothing twice, and a read asks the *open handle* how big
  a file is rather than asking the name again. The third is `Did::of`'s: a
  change **creates** a path that nothing had asked the grants about — a rename
  invents a name, a move and an archive invent one inside a folder — and under a
  grant over a single file (ADR 0001 §4) that name is one nobody granted. So the
  grants are asked once more, at the authorisation's own moment, and a no is a
  `Refused` like every other. *A grant covers where a file goes, not only where
  it comes from.*

  Three decisions the next items inherit. **A refusal by the grants and a
  refusal by the machine are different types** — `Refused` leaves by `Err`, and
  `Failed` travels inside a `Did` — because a record that called a full disk a
  refusal would tell a security review the grants stopped something they did
  not. **The authorisation comes back either way**, so `Entry::ran` is written
  from a call that was attempted, whatever the disk made of it. **Every bound
  says it was reached**: a listing, a read and a search answer with a flag, an
  archive refuses, and a bounded answer that did not say so would read exactly
  like a complete one.

  What it could not close is **item 6b, below under *blocked — linux***: the two
  gaps between checking a path and acting on it that only Linux calls close.

- [x] **7. Keyboard shortcuts** — implements `docs/features.md`'s v0.01 "keyboard
  shortcuts, and a person can change them". `crates/alo-shortcuts`, a **new
  crate** that depends on nothing in this workspace: `modifier.rs` (what is held
  down), `key.rs` (the closed list of keys), `chord.rs` (one combination, and the
  three it refuses to be), `action.rs` (what a shortcut does), `defaults.rs`
  (what alo OS ships with), `changes.rs` (what a person changed, which is all
  that is written down), `shortcuts.rs` (the two resolved, and every question
  asked of them), `clash.rs` (two actions wanting the same keys). 41 tests and a
  doctest, clippy clean.

  **The item's own sentence — express a conflict rather than letting the last
  binding win — turned out to be two problems, and only one of them is a
  refusal.** `bind` refusing a chord something else holds is the easy half. The
  half that shapes the crate is that **a release can add a default onto a chord a
  person already moved something onto**, which no refusal at bind time could have
  seen coming, because the binding was made before the default existed. So a
  clash is a thing the model *holds and reports* (`Clash`) as well as a thing it
  refuses (`Taken`), and the resolution is stated rather than emergent: a
  person's binding beats one we shipped, and two of their own on one chord fire
  nothing, because choosing between them would close a window somebody meant to
  maximise.

  Three decisions the next items inherit. **Only the difference is stored** —
  `Changes` is the settings file and the defaults live in the code, which is what
  lets a default be improved for everybody who never touched it; item 8's
  appearance model has the same shape and should copy it. **A promise elsewhere
  is a refusal here**: `Ctrl+C`, `Ctrl+X` and `Ctrl+V` cannot be taken by a system
  shortcut, because `docs/features.md` promises copy and paste across
  applications at v0.01 and a system shortcut is a key taken away from every
  application at once. **A key is the one printed on the person's own keyboard**,
  not a position on an American one — which leaves the compositor one job written
  down in `key.rs`: a layout with no Latin letters at all needs the shortcut
  matched against the person's Latin layout.

- [x] **8. Appearance** — implements `docs/features.md`'s "Making it yours",
  minus the accent, which is item 8a below. `crates/alo-appearance`, a **new
  crate** that depends on nothing in this workspace: `colour.rs` (one colour and
  how it is written), `token.rs` (the six in `docs/design/figma-brief.md`),
  `picture.rs` (one picture and how it meets the edges), `rotating.rs` (a folder
  of them, one at a time), `background.rs` (the three kinds), `display.rs`
  (which screen), `time.rs` (a time of day), `scheme.rs` (light, dark and the
  schedule), `text.rs` (how big the text is), `lock.rs` (what is on the screen
  when nobody is signed in), `shipped.rs` (what a machine looks like before
  anybody changes anything), `changes.rs` (what a person changed), and
  `appearance.rs` (the two resolved). 58 tests and a doctest, clippy clean.

  **The item said "background per display" and that turned out to be the wrong
  shape.** A background *per display and nothing else* reads the same until a
  projector is plugged in, at which point a machine whose owner chose a
  photograph shows a room full of strangers the wallpaper we chose, because the
  projector is a display nobody has set anything on. So there is one background
  — the person's — and a display they singled out is an exception they made on
  purpose. A display renamed by a driver update loses its exception and falls
  back to their choice, which is the right way round to fail.

  **The second decision was not in the item at all.** The desktop is seen by
  whoever is signed in; the lock screen is seen by whoever walks past. A person
  who pointed their background at a folder of their own photographs picked the
  *folder* — they did not pick, one by one, the pictures a machine left alone in
  a room shows to a corridor. So a lock screen that *follows* the desktop does
  not follow a rotating one: it shows the shipped wallpaper while the folder
  rotates, and says so (`lock_is_holding_back`) rather than leaving somebody to
  notice. Nothing is taken away — a person who wants their photographs on the
  lock screen sets them there and gets them, because saying so is a decision and
  following is not.

  Three decisions the next items inherit. **Only the difference is stored**, as
  in item 7: an untouched machine writes `{}`, and there is a test that says so,
  which is what lets a release ship a better wallpaper to every machine that
  never touched it. **Nothing reads the clock or the disk** — a schedule is
  answered at a time of day that is passed in, and a rotating folder is asked
  *how many pictures it holds* and *how long it has been running* rather than
  going to look — so item 1's rule now covers appearance too. **A promise in a
  standard is a test**: EN 301 549 requires text to reach 200%, so `text.rs`
  asserts the ceiling is at or above it rather than commenting that it should be.

  What this owes elsewhere: **the image must ship a wallpaper named `alo`**
  (`shipped::THE_WALLPAPER`), or a fresh machine has nothing behind its windows.
  That is written into the image item under *blocked — linux*.

- [x] **8a. The accent set** — implements **ADR 0010**. Two new files in
  `crates/alo-appearance`: `accent.rs` (the five, both values each, and the
  three refusals) and `contrast.rs` (how far apart two colours are to look at,
  to the standard). `Changes`, `Shipped` and `Appearance` gain the setting, and
  `Setting::Accent` puts it back. 17 new tests, 536 across the workspace,
  clippy clean.

  **Terracotta is unreachable rather than refused-at-the-door**, which is the
  shape the item asked for made stronger: the accent is a closed set of five, so
  a settings file naming terracotta fails to read at all, and the refusal in
  words is for the one road a colour can still arrive by —
  `Accent::of_colour`, which is where a panel that let somebody type a hex, and
  at v1 an agent asked to *make the accent this colour*, both land. Three
  refusals rather than one, because asking for the agent's colour, asking for a
  ground, and asking for a colour nobody designed are three different mistakes
  and send a person to three different places.

  Three decisions the next items inherit. **A colour set is a claim about
  legibility, so it is measured** — `contrast.rs` is EN 301 549 by way of WCAG
  2.1, and every accent is held to the text threshold against the grounds the
  design brief names, which is ADR 0010's "wants contrast verified" turned into
  a test. **The accent is stored by name and resolved against the scheme at the
  moment of asking**, so one choice covers both grounds and a release can
  correct a value for everybody who chose that colour. **The measurement found
  something the decision did not claim**: terracotta on cream is 2.87:1, under
  the threshold for a word *and* for a shape, so ADR 0010's mark and word are a
  measured requirement rather than a courtesy — noted in the ADR, and the
  shell's to honour.

- [x] **9. Strings** — implements `CLAUDE.md`'s *user-facing strings are
  externalized from day one* and `docs/features.md`'s "Language and access"
  (ADR-free). `crates/alo-strings`, a **new crate** that depends on nothing in
  this workspace: `key.rs` (what names one string), `template.rs` (a sentence
  with named gaps), `filling.rs` (what goes into them), `phrase.rs` (one string
  the code can say, and its note to a translator), `vocabulary.rs` (everything
  the code can say, and where a translation is checked against it),
  `translation.rs` (one language's strings as they arrive, and what can be wrong
  with them), `speaking.rs` (a checked translation — the only thing the lookup
  accepts), `language.rs` (which language, and which way it is read), `union.rs`
  (the 24, each named in itself), `said.rs` (one answer and where it came from),
  `strings.rs` (the lookup and the chain). 75 tests, 5 integration tests over
  the strings this repository already has, 3 doctests and a `compile_fail`,
  clippy clean.

  **The item said "so a missing translation is visible in development", and the
  design decision was refusing to make that a development-only fact.** A build
  flag that marked untranslated strings would answer with a `String` the rest of
  the time, and *shown English because nobody translated it* would be invisible
  in exactly the build a person in Latvia is running. So the answer is a `Said`
  that always carries where it came from — a translation, the source, or a key
  nothing declares — and marking in development is one of three ways of noticing
  rather than the only one; `Strings::unanswered` is the second, and
  `Said::is_translated` on every answer is the third.

  **The second decision was not in the item.** A translation is written by
  somebody whose language nobody here reads, so a sentence that dropped `{bytes}`
  would reach a person as *your file is too big* with no size in it, in their own
  language, with nothing anywhere saying so. `Vocabulary::check` is therefore the
  only door to a showable translation: it matches every gap against the source's,
  refuses a dropped or invented one in words addressed to a translator, and
  returns **everything** wrong at once, because being told about the next mistake
  each time you try again is how a translator gives up. A *partial* translation
  is deliberately not an error.

  Three decisions the next items inherit. **English is a source, not a default**
  — the sentence lives beside the key in the code, which is item 7's *only the
  difference is stored* reaching a fifth crate, and is what lets a release
  improve an English sentence for every machine that has no translation of it.
  **A person names their own second language**: the chain is what they said plus
  the broader form of each (`pt-BR` brings `pt`), and nothing infers a second
  language from a first. **A language is named in its own language** —
  `union.rs` holds the 24 endonyms, and a picker that said *Greek* would be one
  the people it exists for cannot read.

  What it could not close is items 9a–9e below; 9a is now built.

- [x] **9a. Plural forms** — cut from item 9, and built once the rules could be
  read rather than recalled: the loop fetched CLDR's
  `common/supplemental/plurals.xml` from `unicode-org/cldr` and worked from it.
  Four new files in `crates/alo-strings`: `form.rs` (the six shapes),
  `cldr.rs` (the table, each arm quoting the condition it came from, plus
  `Counting`), `plural.rs` (one countable string), and the plural half of
  `key.rs`, `vocabulary.rs`, `translation.rs` and `strings.rs`. 41 new tests and
  1 new doctest; 559 tests and 20 doctests across the workspace, clippy clean.

  **Whole numbers are the scope, and that is what made the table tractable.**
  alo OS counts things and a thing is a whole number, so `Counting` holds a
  `u64` and CLDR's `v`, `w`, `f`, `t` and `e` operands are all zero — which is
  why Czech's `many` and Lithuanian's `many` cannot happen here and French's
  `many` keeps only the half about whole millions. Each condition is quoted in
  full anyway, so what was dropped is visible. Counting something that is not
  whole is a decision to reopen, stated in `lib.rs`, not a form quietly picked.

  Three decisions the next items inherit. **A form's name says nothing about
  which numbers it covers** — Polish has no `other` for a whole number,
  Croatian's `one` covers 21, French's covers 0 — so three things are refusals:
  a form a language never uses, a number spelled out in a form that is not one
  number, and a countable string in a language whose rules are not in the table.
  `docs/quirks.md` records all three. **A countable string owns every form
  beneath its key**, in both directions, so `files.too-big` and
  `files.too-big.one` cannot both exist. **`unanswered` and `missing_from` now
  answer `Vec<Key>`** and expand a countable string into the forms *that*
  language needs — a Polish file with `one` and `other` is not two thirds done,
  and the old signature would have reported it complete.

- [x] **9b. `alo-files` onto `alo-strings`** — the largest list, and the one that
  includes words a person approves. Two new files in `crates/alo-files`:
  `words.rs` (every string this crate can say, the English beside each key, and
  the notes a translator needs) and `saying.rs` (what the six verbs are and what
  a person approves, in the language they read — **gone at 9g**, which put those
  three answers on the verb itself). `Failed`, `RealError`,
  `Touching`'s two refusals and `Did`'s all moved onto it; `verbs.rs` now
  declares the six **from** those constants. 41 phrases and one countable
  string, 14 new unit tests and 5 new integration tests; `Key::unchecked` is new
  in `alo-strings`. 578 tests and 20 doctests across the workspace, clippy
  clean.

  **The guarantee the item asked for is kept by there being one string, not two
  that agree.** `alo-capability` refuses a verb whose sentence does not name
  every argument, `alo-strings` refuses a translation that drops a gap the
  source has — and those are the same rule only while the string a translator is
  handed is the string the declaration was checked against. So the declaration
  reads the constant rather than repeating it, and a test walks all six.

  Three decisions the next items inherit. **`Failed` and `RealError` lost their
  `Display`**, which is the strong form of *hardcoded English is a bug*: a
  `Display` is one `to_string()` from a screen, in a shell whose author had no
  reason to think about it, so the only road to words is `said(&Strings)` and
  every answer says whether anybody translated it. What is given up is
  `std::error::Error` on two types that were never errors a programmer handles.
  **`Touching::of` and `Did::of` take the strings**, because the two refusals
  this crate words itself are carried into the record by
  `alo_capability::Refused` — so what a person was told is what is written down,
  one rendering rather than an English record beside a translated screen.
  **`Key::unchecked` takes a `&'static str`**, so a key can only come from a
  literal, and each crate that declares words puts every one of its own back
  through `Key::named` in a test — `alo-shortcuts`' shipped bindings and
  `alo-appearance`'s shipped wallpaper, one crate further on.

  What it does **not** do is put a translated sentence into a `Call` or an
  approval: `Call` renders and keeps its own, and moving that is 9e.

- [x] **9c. `alo-shortcuts` onto `alo-strings`** — the list a person reads every
  time they open the shortcuts panel. Two new files in `crates/alo-shortcuts`:
  `words.rs` (every string this crate can say — 38 phrases, the English beside
  each key, and the notes a translator cannot work without) and `refusing.rs`
  (why a combination cannot be a shortcut, and what it says). `Action`,
  `Modifier`, `Modifiers`, `Key`, `Chord`, `Taken` and `Clash` all lost their
  `Display` and gained `said` or `shown`. 63 unit tests (was 41), 6 new
  integration tests, 606 tests and 20 doctests across the workspace, clippy
  clean.

  **The item said "`Key::label`" and half of those labels turned out not to be
  strings at all.** Fifty-three of the sixty-nine keys print a mark that is the
  same on every keyboard in the union — `Q`, `7`, `,`, `F1` — and translating one
  would be naming a *position*, which is the model `key.rs` was built to reject:
  `Super+Q` is the key marked Q on the person's own keyboard. The other sixteen
  print a word, and it is a different word almost everywhere — *Entf*, *Pos1*,
  *Bild ↑* — so those are the strings, and they are the ones whose notes matter.
  Declaring all sixty-nine would have handed a translator forty-one rows reading
  `A`, `B`, `C` and made `unanswered` — *what a release note has to count* —
  report fifty-three strings nobody should ever translate. `docs/quirks.md`
  records it.

  Three decisions the next items inherit. **A sentence never joins a list**:
  `Taken` names the one action holding the chord in a gap, and `Clash` names the
  chord and hands `actions()` over to be drawn as rows, because the separator is
  not punctuation a program can pick — Greek writes `;` where English writes a
  question mark — and the conjunction would be a string placed by a machine that
  does not know the sentence. **A deserialiser has no `Strings` and never will**,
  so the one thing that still needs words where none can be asked for —
  `Chord`'s `serde(try_from)` — writes the *key* of the refusal rather than a
  sentence, and whoever shows it looks that key up. **`DefaultsError` keeps its
  English and its `Display`** and now names things by the stable names a
  settings file holds: it says a *release's* own defaults contradict themselves,
  which is read by whoever is fixing them, and `SnapLeft` is what they need
  rather than a row in whichever language was loaded. `Debug` on `Modifiers` and
  `Chord` is hand-written for the same reader.

  The stored format did not move: the settings file is written exactly as
  before, and the two serde tests say so.

- [x] **9d. `alo-appearance` onto `alo-strings`** — the eleven names a person
  picks a colour from, and every refusal about a value they chose. Three new
  files in `crates/alo-appearance`: `words.rs` (28 phrases, the English beside
  each key, and a note on every one of them), `unreadable.rs` (`NotRead` — what
  a settings file that did not read says, where nobody can be asked for words)
  and `testing.rs` (the fixture the other files' tests are written against).
  `Token`, `Accent` and all eight error types lost their `name` or their
  `Display` and gained `word` and `said`. `Word` moved into `alo-strings` as
  `word.rs` and all three declaring crates now use it. 98 unit tests (was 75), 8
  new integration tests; 635 tests and 20 doctests across the workspace, clippy
  clean.

  **A colour name is the one string that carries none of itself.** A sentence
  can be translated from its own words; *verdigris* cannot, and neither can
  *terracotta*, *charcoal* or *warm stone*. So every one of the 28 carries a
  note — the only one of the three lists where that is true — and the eleven
  colour ones describe the colour rather than assuming the word travels. The
  integration test is the argument made in German: *Grünspan* where English
  borrowed from French, *Anthrazit* where English named a grey after burnt wood,
  neither reachable from the other word by word.

  Three decisions the next items inherit. **`Word` lives in `alo-strings`
  now**, with `Word::phrase` replacing the loop each declaring crate had written
  out; `WordsError` in all three crates gained one `Word` variant in place of
  its `Sentence` and `Note` pair. **A deserialiser writes the key, and six of
  them share one type**: `serde(try_from)` requires a `Display` at the one point
  no `Strings` exists, so `NotRead` writes the key of the refusal and
  `docs/quirks.md` records it — `alo-shortcuts`' private `NotAChord` was the
  first of these and this is the sixth through tenth. **A number is not a
  string**: `TextScale` keeps `200%` and `TimeOfDay` keeps `18:00`, because how
  a number or a time is written belongs to the region rather than the language —
  but the percent sign is *in* the sentence, so a language that writes *200 %*
  can, and there is a test that says so.

  **`alo-strings`' integration test has gone**, which is what that file said
  would happen: it carried copies of four `alo-appearance` strings because it
  was built before any of its users existed, and all three users exist now. Its
  four tests live in `crates/alo-appearance/tests/what_this_crate_says.rs`,
  against the vocabulary the code actually uses rather than a copy of it.

- [x] **9e. `alo-capability` onto `alo-strings`** — every way the capability
  model says no, in the language the person reads. Three new files in
  `crates/alo-capability`: `words.rs` (33 phrases and one countable string, the
  English beside each key, and the notes a translator cannot work without),
  `refusing.rs` (`NotGranted` — why the grants refused, and what it says) and
  `testing.rs` (the fixture the other files' tests are written against).
  `GrantError`, `ArgError`, `CallError`, `ProposalError`, `AnswerError` and
  `NotAuthorised` lost their `Display`; `Reach::describe` and `Ask::describe`
  became `shown(&Strings)`. 107 unit tests (was 85), 6 new integration tests
  against the real vocabulary, 655 tests and 20 doctests across the workspace,
  clippy clean.

  **The decision this item turns on: a refusal is a value, and it is worded
  when somebody shows it.** Item 9b's rule — the words of a refusal are made
  where the refusal is made, so the record and the screen cannot disagree — is
  right and could not keep its shape here. Wording a refusal in this crate
  would mean handing `Grants` a `Strings`, and then *whether an agent may touch
  a folder* would depend on a vocabulary having been loaded. So
  `Grants::permitting` answers with a `NotGranted` carrying what was asked for
  and the grant that ran out, and `said(&Strings)` renders it. 9b's guarantee
  survives in the stronger form: the screen and the record render **the same
  value**, so neither can be a language the other is not.

  Three decisions the next items inherit. **`Refused` has two doors and they
  differ only in whose words they are**: `not_granted` for the refusal the
  grants made, `worded_elsewhere` for one only the crate holding a resolved
  path can say, which arrives already `Said`. **`Entry::refused` takes the
  strings rather than the words**, so a record cannot be handed a sentence
  about something else — `alo-record` depends on `alo-strings` for that and for
  nothing else. **Three errors keep their English**: `VerbError`, `VerbsError`
  and `SentenceError` refuse a *declaration* and are read by whoever is writing
  an adapter, which is `alo-shortcuts`' `DefaultsError` one crate on.

  What it could not close is 9f and 9g below.

- [x] **9f. `alo-models` onto `alo-strings`** — cut from 9e, which was two
  crates and one decision. Two new files in `crates/alo-models`: `words.rs` (29
  phrases, the English beside each key, and the notes a translator needs) and
  `refusing.rs` (`NotAllowed` — which rule refused a question and where it would
  have gone). `ProviderError`, `SecretError`, `RuntimeError` and `NotTried` lost
  their `Display`; `InferenceSource::describe` became `shown(&Strings)` and lost
  its `Display` too; `Tried::describe` became `said` and `caveats`. 20 new unit
  tests and 7 new integration tests against the real vocabulary; 682 tests and
  20 doctests across the workspace, clippy clean.

  **Two of these are not refusals, and that is what is different about this
  list.** *By someone, which has not said where it runs* is read **before** a
  person decides whether to paste a contract into a question (ADR 0008), so it
  is the one string here where a translation that softened it would take away
  the only thing on the screen saying the question is about to leave the
  building. Its note says so. The policy's three refusals put that clause inside
  themselves, which is why the source is a string rather than something a caller
  assembles: a refusal and the place named in it are one language.

  Three decisions the next items inherit. **A sentence that would have to count
  is a sentence with the number beside it** — item 10 settled that in this crate
  before `alo-strings` existed, and 9f kept it: `NotEnoughDisk` says *there is
  not enough room on this disk for that download* and carries the two numbers as
  fields, so nobody writes English's two plural shapes where a language has
  three. There is no `Plural` in this crate and a test says so. **A reason is a
  variant, never a `&'static str`**: `RuntimeError::Refused("…")` carried a
  sentence an adapter wrote in English, one `to_string()` from a screen, and is
  now `DownloadIncomplete` with a string of its own — an adapter that needs
  another reason adds one, the way a verb is added to a closed list. **A refusal
  the policy made is carried whole**: `NotTried::Forbidden` holds the
  `NotAllowed` rather than a rendering of it, so the words are the policy's in
  whichever language the person reads, and `Trying::under` still takes no
  `Strings` — what is permitted does not depend on a vocabulary having loaded.

  What it did not touch is `alo-egress`, which is 9h below and was not on any
  list before this iteration.

- [x] **9h. `alo-egress` onto `alo-strings`** — the last crate holding English,
  and the one with the indicator in it. Three new files in `crates/alo-egress`:
  `words.rs` (13 phrases, the English beside each key, and the notes a
  translator cannot work without), `refusing.rs` (`Refusal` — which rule refused
  — and `NotPermitted`, which moved here out of `policy.rs`) and `testing.rs`
  (the fixture the other files' tests are written against). `DestinationError`
  lost its `Display`, `Destination::describe` became `shown(&Strings)`,
  `Leaving::describe` became `said(&Strings)` and `Shown::describe` with it, and
  `EgressPolicy::refusal` answers with a value rather than a sentence. 45 unit
  tests (was 27), 7 new integration tests against the real vocabulary; 707 tests
  and 20 doctests across the workspace, clippy clean.

  **The indicator line is one sentence per reason, and that is the decision the
  item did not contain.** *`{agent} is asking a question of {destination}`* could
  have been a stem plus a place, which is three fewer strings and would have
  been wrong in English before it was wrong anywhere else: a question goes *of*
  somewhere, a fetch comes *from* it, and a language that inflects the place
  needs the whole sentence in front of it to choose. So there are three, each
  whole, and the preposition is inside the translated string where a translator
  can move it — `alo-shortcuts`' *a sentence never joins a list* met from the
  other side.

  Three decisions the next items inherit. **A destination that is data is not a
  string**: `Destination::word` answers `None` for a host a verb's argument
  named, because `alo.example` is somebody's address and a translation of it
  would be an invention — the rule a filename is held to in `alo-files`, now
  written into a type rather than a comment. **The rule and the refusal are two
  types**, because `alo-record` writes a held-back entry from a `NotPermitted`
  and from nothing else (item 5a): an enum carrying the egress in each variant
  would have made every variant a way to write down a refusal that never
  happened, so `NotPermitted` keeps private fields and a `pub(crate)`
  constructor the policy alone calls, and `Refusal` is the public value beside
  it. **`Entry::held_back` takes the strings**, as `Entry::refused` has since
  9e, so the record keeps the rendering the person read.

  What the twin guarantee turned out to be: `Destination::shown` and
  `InferenceSource::shown` **cannot** be one string, because one names where an
  answer came from (*by someone…*) and the other where a thing is going
  (*…of someone…*), which is a different grammatical position. What they must
  not do is differ about the provider, and that is a test walking every source
  that leaves: both name it, both name the region, and both say *has not said
  where it runs* when nobody has.

- [x] **9g. The sentence a person approves** — the question 9b left, 9c and 9d
  did not touch, and 9e answered without moving. A **public surface change**
  reaching `alo-capability`, `alo-files` and `alo-record` at once.
  `alo-capability`: `sentence.rs` rewritten around `alo_strings::Template`,
  `Arg` and `Verb` declared from `Word`s, `Call` carrying a `Key` and answering
  `sentence(&Strings)`, and `Proposal`, `Approved` and `Authorised` with it.
  `alo-files`: the six declared from the constants rather than out of them, and
  `saying.rs` gone. `alo-record`: `What::of` and four of `Entry`'s constructors
  take the strings. 108 unit tests in `alo-capability` (was 107), 7 integration
  tests (was 6); 707 tests and 20 doctests across the workspace, clippy clean.

  **The answer was 9e's, applied to the sentence, and the shape it took is a
  verb declared from the words rather than beside them.** `Verb::checked` and
  `Arg::taking` take an `alo_strings::Word` — a key, the English, the note —
  and there is no way to declare one from a bare string. That makes the
  guarantee structural rather than a convention `alo-files` happened to keep:
  the string a translator is handed is *necessarily* the string the declaration
  was checked against, so *a sentence names every argument* and *a translation
  may not drop a gap* are the same rule about the same string, in every crate
  that will ever declare a verb.

  **What the item did not say, and is the reason it was worth doing carefully:
  there were two parsers for one syntax.** `Sentence::parse` was this crate's
  own and `alo_strings::Template` is the vocabulary's, and they disagreed about
  `{{` — so a sentence with a literal brace in it was a declaration
  `alo-capability` refused and a phrase `alo-strings` accepted, which is the
  9-series' whole failure mode one level down. There is one parser now, and
  `sentence.rs` is what is left of the old one: the key, and the one rule that
  is about approval rather than about strings — a sentence made only of its
  arguments describes nothing.

  Three decisions the next items inherit. **`CallError::Unsayable` is gone**,
  because nothing renders a sentence at the moment a call is made any more and
  the variant could not happen — the unreachable branch deleted rather than
  translated, as in 9f and 9h. **`AnswerError::Lapsed` carries the call**, not a
  rendering of it, so the question quoted back to somebody is rendered with the
  vocabulary of the refusal around it; `docs/quirks.md`'s *two gaps in a
  translated sentence* is closed by that and by `CallError::Missing` carrying
  the purpose's key. **A word a verb is declared with can be one nobody
  declared**, which no check at declaration time can reach — it compiles and
  reaches a person as a key where the sentence they are approving belongs — so
  every crate that declares verbs owes the test `alo-files` now has
  (`everything_the_six_say_is_something_this_crate_declares`), and the contract
  says so.

- [x] **10. Test a provider before saving it** — implements `docs/features.md`'s
  v0.5 *test a provider before saving it* and closes the loose end in
  `provider.rs`. Three files in `crates/alo-models`: `secret.rs` (a key as it
  was just typed, for the length of one call), `tried.rs` (what a person is told
  — both shapes of one answer), `trying.rs` (the only file that knows what a
  provider's API looks like). One test-support file came with them,
  `testing.rs`, which `ollama.rs` now shares rather than keeping its own copy of
  the stub server. 26 new unit tests and 2 new doctests, one of them
  `compile_fail`; no new dependency.

  **The item is one sentence — a mistyped key is found now — and the design is
  the three things that sentence does not say.** *The policy is asked first, and
  there is no way to skip it*: `Trying::under` takes a `SourcePolicy`, so a
  machine set to keep questions in the building does not reach a provider
  outside it to see whether the key works, and a refused test opens no
  connection at all. *A redirect is refused rather than followed*, because the
  address the policy answered about is the address that gets reached and a key
  does not travel to a host nobody decided about. *Nothing of the person's
  leaves* — the test is a `GET` with no body, and the test that proves it reads
  what actually went out on the socket.

  Three decisions the next items inherit. **A key goes into this crate and does
  not come out**: `Secret` has no accessor, no `Display`, no `Serialize`, a
  hand-written `Debug`, and a `pub(crate)` reader asserted by a `compile_fail`
  doctest checked to fail on the privacy (E0624) — and it says outright what it
  does *not* claim, which is that the bytes are scrubbed on drop. **Names a
  provider wrote are held to `alo-files`' rule**: one that cannot be shown is
  counted and left out, a list longer than anybody reads is cut, and both are
  said in the answer. **A sentence here counts nothing out loud** — the plural
  rules are item 9a and are not written from memory — so `Tried::describe`
  carries no number and the numbers are accessors.

  Built and unit tested against a stub on a real socket. **Not run against a
  provider anybody pays for**, which is owed with the rest of the hardware
  verification; `docs/quirks.md` records the one convention this depends on and
  says the same thing about it.

---

## Blocked — linux

Not this loop's, on this machine. Listed so the queue is a true picture of v0.01
rather than only of what is convenient.

- **Compositor** — Wayland via Smithay, one display, keyboard and pointer.
- **Sign-in and the local account**, the agent overlay, the launcher and window
  management, copy and paste, window switching — all draw on the compositor.
- **Application verbs, the acting half** — AT-SPI, D-Bus, the portal backend
  (ADR 0005). The *file* half of this was listed here and was wrong: opening a
  folder needs no portal and no accessibility tree, so it was item 6a above and
  is built.
- **6b. Opening from a handle, and renaming without replacing.**
  `docs/quirks.md` records the two gaps the portable acting half cannot close: a
  path checked and then opened *by name* can have a link swapped in between the
  two, and `fs::rename` has no portable no-clobber form, so a destination is
  checked for and then renamed onto. Both have Linux answers with no portable
  spelling — `openat` with `O_NOFOLLOW` from a directory handle, and `renameat2`
  with `RENAME_NOREPLACE` — and both need a Linux host to compile as well as to
  test. Not a rewrite: the decisions, the refusals and the tests are settled,
  and this replaces the syscalls underneath them. The workspace forbids
  `unsafe`, so it needs either a pinned dependency wrapping the calls or an ADR,
  and choosing between those is the first thing the item does.
- **4b. Where the record file lives, and when it is shortened.** What item 4a
  could not close, and the whole of what is left of it: a path under `/var/lib`
  that the package decides, the setting the retention rule is read from and
  written to, and something with a timer that calls `Writing::prune`. All three
  are `alo-agentd`'s. The crate takes a path and a moment and holds no opinion
  about either, so this is wiring rather than design — but it is the daemon's
  wiring, and there is no daemon. It also owes one thing to the person: a
  managed machine's rule is the organisation's (ADR 0004), so where the setting
  is read from is a question about enrollment and not only about a file.

- **Egress enforcement** — item 5's policy, made true at the network boundary.
- **The image** — OCI-built, bootable, atomic. It owes item 8 one thing: a
  wallpaper named `alo` (`alo-appearance`'s `shipped::THE_WALLPAPER`), which is
  what a fresh machine shows behind its windows. An image without it boots to
  nothing behind the windows rather than to a colour nobody chose.
- **The workspace client running as an application on the shell.**

## Blocked — hardware

- **The model stack against a real Ollama.** Ticked in `ROADMAP.md` as built and
  tested, with this verification owed. A CPU-only run would close most of it and
  needs no GPU — but it needs Ollama installed, which is a person's decision to
  make rather than a loop's.
- **"The GPU works on first boot"**, which needs a machine that has one.
- **The v0.01 exit gate** — one person, one machine, one cold boot.

## Not ours

- **Agents point at the local model by default** — `alo-workplace`, and
  configuration rather than code (`AiConfig` has spoken to an OpenAI-compatible
  endpoint since 2025).
