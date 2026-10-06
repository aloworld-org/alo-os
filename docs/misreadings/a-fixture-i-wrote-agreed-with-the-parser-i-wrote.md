# A fixture I wrote agreed with the parser I wrote

**Found 2026-10-06, Mac lane, when the kernel loop would not enqueue a pull
request whose required check had passed fifty minutes earlier.**

## What was concluded

That CI had not reported yet. The loop said so every thirty seconds, in those
words — *`alo/gates-on-a-runner` has not reported on this commit yet* — and it
was on course to say, at the sixty-minute mark, that *the required checks had
not finished*. A sentence about CI being slow, which would have sent the next
reader to the runner.

## What was true

The check had passed at 16:52. `gh pr checks` said `pass`. The exact REST
endpoint the loop polls, queried with the loop's own credential, answered
`"state": "success"` for the exact context the loop required.

The loop could not see it, because it searched for this:

```
"context":"alo/gates-on-a-runner"
```

and GitHub sends this:

```
"context": "alo/gates-on-a-runner"
```

**One space.** The literal matched nothing on any real reply.

## The mechanism

**The parser and its tests were written by the same hand in the same hour, so
the tests encoded the author's guess about the wire format rather than the
wire format.** Every fixture in that module is hand-typed compact JSON. Every
reply GitHub sends is pretty-printed. Four tests passed, forever, about a
shape the server never produces.

This is not *the tests were too few*. There were four, they were thoughtful,
and one of them is specifically about telling *has not reported* from
*passed* — the right distinction, tested entirely inside the wrong dialect. **A
fixture is only evidence about the real world if it came from the real
world.**

Two things made it invisible rather than merely wrong:

- **The failure direction was "wait".** An unreadable reply and a pending
  check were the same answer, so the bug expressed itself as patience. Nothing
  went red; the loop looked like it was working.
- **The one real landing path was rarely exercised.** Earlier tasks were
  enqueued by hand, so the loop had been publishing without reaching this
  line.

## The cure

**Take fixtures from the wire.** The test added with this entry carries a
response captured from `GET /repos/.../commits/<sha>/status` on the day,
spacing and `creator` sub-object included, and says in its own doc comment
where it came from. The hand-written compact fixtures stay, deliberately —
both shapes are valid JSON and a parser owes both an answer. What it may not
do is work on only the one its author happened to type.

**Find the key, then read its value; never search for a `"key":"value"` pair
you built by hand.** The sibling helper in the same file already did this and
was correct. The broken function spelled the pair itself and so re-implemented
a parser it already had.

**And never let *I could not read this* share an answer with *not yet*.** They
need opposite responses — one is worth waiting for and the other never is —
so they are now different variants, and an unparseable reply ends the wait
instead of extending it. Bad credentials, a renamed repository and an unknown
commit all reply with a `message` and no `statuses`, and all three used to
read as patience.

## Related

- [`a-cancelled-check-is-not-a-failing-one.md`](a-cancelled-check-is-not-a-failing-one.md)
  — the same seam, the same confusion of two CI states that need opposite
  responses.
- [`i-watched-the-client-and-called-it-the-job.md`](i-watched-the-client-and-called-it-the-job.md)
  — same day: a check that could not observe the thing its sentence named.
