# Recording a question nobody was asked — a design

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-record`, `alo-turn`, `alo-agentd`)
- Contributor: Claude Code
- Status: **complete.** The daemon writes one, and **response/record agreement
  is now claimed** — a daemon test drives a real policy refusal through the
  production path, reads the record back **off the disk**, and compares its
  wording with the response the agent was given.

## What is built, and what is not

| | |
|---|---|
| `Happened::NeverPutAnywhere`, `Entry::never_put_anywhere` | **built** |
| Compatibility behaviour under an unknown tag | **built, four tests** |
| Stored bytes carry no question, credential or endpoint | **built** |
| `Turning::a_question_that_went_nowhere` | **built** |
| Write-failure closes the turn, refusal not reported | **built, mutation-checked** |
| Daemon emission | **built** |
| Persisted response/record agreement | **built, mutation-checked** |
| Write failure: not-recorded answer, closed turn, no connection, no fallback | **built, mutation-checked** |

The rest of this note is the design the remaining work follows.

## The gap

A question refused by an organisation's rule is answered to the agent and
**written down nowhere**. `alo_record::Entry`'s public constructors are all
points in a *verb's* journey, and `Turning::writing_down` is private —
deliberately, as "the only road to the record in this crate". A refusal decided
in `crate::doing`, before any turn method is entered, reaches none of it.

ADR 0001 §7 wants every execution and every refusal in the record. This one is a
refusal and it is not there.

## What must not happen

**A question refusal must not masquerade as a file-verb call.** `Happened::Ran`,
`Stopped` and `TurnedAway` all carry a `What` or a verb name, and a reader —
a person — sorts entries by what they appear to be. A refused *question* recorded
in any of those shapes would put something in the record that never happened: a
call nobody made.

The precedent for the right shape is already there. `Happened::AnsweredHere`
records a question answered locally and holds **who asked and nothing else** —
no `What`, no verb, and explicitly no question text. A refused question is that
same kind of event with a reason attached.

## The variant

Additive, and one:

```rust
/// A question was refused before it was put anywhere.
NeverPutAnywhere {
    /// Which agent asked.
    agent: Line,
    /// Why it was not put, in the words the person was shown.
    why: Line,
},
```

No `What`. No verb. No approval, no grant — none were involved. It sits beside
`AnsweredHere` because it is the same event's other ending.

`Happened` is an externally tagged serde enum with `rename_all = "kebab-case"`,
written one JSON object per line to `record.jsonl`, so this adds the tag
`never-put-anywhere` and changes no existing tag or field.

## Record compatibility, stated exactly

- **Entries already written are unaffected.** No existing variant gains, loses or
  renames a field, so every line in an existing `record.jsonl` parses exactly as
  it did. A test reads a fixture of previously-written lines back and asserts
  that.
- **Readers at this version read both.** The reader ships in the same binary as
  the writer.
- **A downgraded binary cannot read the new tag.** An externally tagged enum has
  no catch-all — `#[serde(other)]` does not apply — so an older `alo-record`
  meeting `never-put-anywhere` fails that line. This is the honest cost and it is
  named rather than designed around.

  Under `CLAUDE.md`'s expand → migrate → contract, **this change is the expand
  step and nothing more**: the variant is added and written, and nothing is built
  that *requires* it to be present. Anything that later reads records for
  policy-refusal statistics must tolerate its absence, because records written
  before this exist and always will.

## One refusal, one rendering, carried to both

The rule is already decided once, by `alo_models::NotAllowed`, and worded once,
by `crate::doing::a_rule_refused` — which both doors already go through. The
recording path takes **that same `Said`**:

```
NotAllowed  ──►  a_rule_refused(..) ──►  Said ──┬──►  ToAnAgent::refused(&said)
   (decides)          (words, once)             │
                                                └──►  Entry::never_put_anywhere(agent, &said, now)
```

Nothing re-renders, nothing re-decides, and no second sentence is composed. The
record stores `Line::of(said.text())` — the same characters the person read, in
the language they read them in.

**A narrow additive door on `Turning`**, because the record's one road stays one
road:

```rust
/// A question that was refused before it was put anywhere, written down.
pub fn a_question_that_went_nowhere(
    &mut self,
    why: &Said,
    now: SystemTime,
) -> Result<(), NotDone>
```

It builds the entry and calls the existing private `writing_down`. The agent
comes from `self.turn.grantee()` — the turn already knows who is asking — so a
caller cannot name somebody else.

## What is stored, and what is refused storage

| Kept | Why |
|---|---|
| the agent | ADR 0001 §7: under whose authority |
| the rendered refusal | what the person was told, so the record and the screen agree |
| the moment | as every entry |

**Not kept, and there is nowhere for it to go:**

- **the question**, for `AnsweredHere`'s stated reason — a record that kept
  questions would be a transcript of everything a person ever said to their
  machine;
- **any credential**, which never reaches this path at all now that the rule is
  asked before the keyring;
- **the endpoint** — the refusal names the *source* the person configured, which
  is already in the sentence they read, and nothing adds an address beside it.

The refusal sentence is our own vocabulary and never client text; that property
is already tested where the sentence is made.

## When recording fails

`writing_down` maps a failure to `NotDone::NotRecorded` and closes the turn,
which is ADR 0001 §7's *a machine that cannot write the record stops*. The
question path must behave the same way and **must not report the policy refusal
as though nothing else had gone wrong**:

- the response becomes the **not-recorded** refusal, in `alo-turn`'s own words,
  not the policy sentence;
- the turn closes, as it does for every other unrecorded event;
- **no success and no quiet substitution.** A person told only *your
  organisation's rule refused this* would believe the machine had behaved
  correctly, when in fact it also failed to write down that it had.

Tested with a record that refuses writes — `alo-keeping`'s failing fixture
already exists for this — asserting the response is the not-recorded sentence and
**not** the policy sentence.

## What would make response/record agreement true

Not asserted until a daemon test does this:

1. a managed bound refuses a provider question through the production path;
2. the response is captured;
3. the **persisted** record is read back;
4. the entry is `never-put-anywhere`, and its `why` equals the response's text
   exactly.

Comparing two values a test rendered itself would prove only that one function is
deterministic. Reading it back off the record is what makes it an agreement.

## Order of work

1. The variant and its constructor, with the old-records-still-parse test.
2. The `Turning` door, with the write-failure test.
3. `doing.rs` calling it, with the read-back agreement test.

Each is a publishable increment and step 3 is where any agreement claim belongs.

## What this does not touch

Production policy loading. Nothing here supplies an organisation's bound, invents
a configuration key, or makes managed-policy support any more complete than it is
— which is not at all. Both dependencies stay as they were recorded.
