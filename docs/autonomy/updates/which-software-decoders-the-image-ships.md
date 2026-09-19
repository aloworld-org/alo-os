# Which software decoders the image ships

**2026-09-19.** The question ADR 0051 left for counsel was answered, and the
answer is [ADR 0058](../../decisions/0058-which-software-decoders-the-image-ships.md).
`crates/alo-playing` carries it. 43 tests.

## What was open

ADR 0051, on 2026-09-17, settled the shape and left the list. **Encoding** was
finished — AV1 or VP9, Opus, Matroska or WebM, royalty-free. **Decoding** had an
order — the machine's own hardware decoder, then a redistributable licensed
decoder, then a plain refusal — and a hole where a fourth step would go: *which
software decoders may the image itself carry?*

That hole was held as `SoftwareDecoders::NotAnsweredByCounsel`, a type with one
value, with an acceptance test that read the decision and **failed the day
somebody answered**. It did. This report is what the failure turned into.

## The answer

**We ship a decoder when nobody can charge us for shipping it** — because it was
made free, because the patents expired, or because a company that paid the
royalty published a binary for us to pass on. Everything else is the silicon's
job, and where there is no silicon the machine says so.

| Format | In the image |
|---|---|
| AV1, VP9 | yes, free by design |
| Opus, Vorbis, FLAC, PCM | yes, free by design |
| MP3, AAC-LC | yes, **because their terms ran out** |
| H.264 | no — the silicon, or Cisco's `openh264` |
| HEVC | no — the silicon alone, or a refusal by name |

Two readings were considered and rejected, and both are written up in the ADR.
**Shipping a full media stack** on the argument that the EU does not enforce
patents on software is wrong in the way that matters: the EPO grants and
European courts enforce patents on technical processes implemented in software,
and a company selling a machine is exactly the party a pool pursues. **Shipping
nothing encumbered at all** was the safest reading and produces the failure this
product exists to argue against — an ordinary phone video that does not play.

## What changed in the code

- **`Right::ByExpiry` is a new answer**, sitting between *free* and *the
  silicon*. It is not a tidier `RoyaltyFree`: a format that was made free was
  free the day it was published, and a format whose term ran out was encumbered
  until a date somebody can argue about. Folding them together would have made
  the one arguable row invisible, which is the row a lawyer has to read.
  `Right::rests_on_a_patent_term` returns true for exactly this one.
- **`Audio::has_expired` and `Video::has_expired`** name MP3 and AAC-LC, and
  nothing in video. Baseline H.264 is the closest call and is still excluded:
  the family's terms differ by profile and by filing, and **a file does not
  announce its profile before it is decoded**, so the safe step is the one
  somebody else already paid for.
- **`Audio::is_royalty_free` lost MP3**, which it used to carry. It now means
  *free by design* and only that.
- **`SoftwareDecoders` carries the list**, with `THE_ONE_FOR_COUNSEL` naming
  AAC-LC — the single row that rests on a term, and the single sentence still
  with a lawyer.

## What the change cost, found by running it

Four existing tests failed, and each failure was the decision arriving rather
than a mistake:

- **The sound of a film can no longer fail on its own.** Every audio codec the
  decision names is now free or past term, so `the_right_to_sound` cannot return
  `Right::None` for anything in the closed list. Two tests — one in `deciding`,
  one in `refusing` — existed to prove that *a picture that decodes must not turn
  a refusal into silence*, and no file can arrange that any more. They now build
  the decision by hand and assert the rule, with a note saying why: **the rule
  has to outlive the list**, because the day an encumbered audio codec is added
  it must already be right.
- **The common refusal halved.** A phone's H.264 film on a bare machine used to
  fail on two tracks; it now fails on one. The count is kept in the test, because
  a film refused for one reason and a film refused for two are different problems
  to fix.
- **A new test says the same case plays** once the machine has the silicon —
  picture licensed by the chip, sound licensed by nobody. That is the whole
  return on deciding AAC-LC, stated as something a person would notice.

## What is still open, and it is smaller

**One sentence, for a lawyer:** *may a machine sold in the EU include a software
decoder for AAC-LC and for MP3, on the ground that their patents have expired?*

It replaces ADR 0051's four-format question. The deadline did not move and is
still a shipment rather than a version: **before the certified laptop goes to
anybody outside this team.** If the answer is no, one row of the table changes
and `alo-playing` changes with it; nothing else depends on it.

The ADR says plainly that it is not legal advice. It is a position on the facts
we have, written so that a lawyer can correct one row instead of re-deriving the
whole thing.

## What this unblocked

`docs/autonomy/v0-5-devices-and-media-plan.md` task 1 was blocked on two things.
One of them is gone. **It is now blocked on a machine to play a real sample file
on, and on nothing else** — which cannot be written, only run.
