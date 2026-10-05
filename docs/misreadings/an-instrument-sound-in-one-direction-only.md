# An instrument sound in one direction only

**Concluded:** *nothing in any `src/` returns these types — no `-> Type`, no
`-> Option<Type>`, no `-> Result<Type, _>` — so the list is corroborated from a
direction it cannot be wrong about in the same way.* Sent to another lane on
2026-10-05 as independent support for their *built but never constructed*
list, while two of this lane's own measurements that hour were wrong.

**What was true.** The check is sound when it answers **zero** and unsound when
it answers anything else, and the sentence above does not say so.

A function returning a type is not a function building one. The other lane ran
it against their four and found it immediately:

```
alo-displays/src/changes.rs:83
    pub const fn night_light(&self) -> Option<NightLight> { self.night_light }
```

That hands back a field. The type is `Copy`, so an owned return costs nothing
and **constructs nothing**. The check counts it as a source.

So: *nothing returns it* really does imply *nothing builds it* — a type no
function yields cannot reach a caller. But *something returns it* implies
nothing at all, and that is the half a reader will use, because a non-zero
answer is the one that looks like a finding.

**For the twenty-three it was run on, the corroboration happens to hold** —
every one answered zero, so no getter was miscounted. It was right about the
data and wrong as an instrument, which is the least useful way to be right.

## The mechanism

**A one-directional instrument presented without its direction.** Both halves
matter and the second is the one that went unsaid:

- The check has a **sound direction** and an **unsound** one.
- The sentence offering it described neither, so a reader inherits a tool whose
  failure mode is invisible at the call site.

It was offered as *corroboration from a direction it cannot be wrong about in
the same way* — carefully hedged against the *other* instrument's failure mode
and silent about its own. Hedging one instrument is what made the sentence
sound measured.

**And the shape fooled both instruments at once.** The other lane's matcher read
`-> &Screenshot {` as a struct literal, because of the brace. This one reads
`-> Option<NightLight>` as a construction, because of the arrow. **Two
instruments built from opposite directions, defeated by the same getter, in one
evening.** A getter looks like a source to anything that pattern-matches a
signature, which is a fact about getters rather than about either tool.

## The wrong answer that announces itself, and the one that does not

Two bad measurements, same hour, same author, and only one was caught by its
own output.

**The first announced itself.** A per-type count whose second alternation was
unanchored matched every `derive(Default)` in the repository and reported
`Default=243` for all twenty-three types:

```
ApprovalFrame   Default=243
ApprovalLook    Default=243
…               …            (twenty-three times)
```

**Twenty-three identical non-round numbers is not a measurement**, and that is
the only reason it was caught. A less uniform wrong answer — three types at 2
and the rest at 0 — would have shipped and been quoted.

**The second did not announce itself.** The returns check produced a clean,
plausible, uniform zero, and was passed on as evidence. It took another lane
running it on different data to find the getter.

**So the dangerous instrument is the one that agrees with you.** The first was
loud because it was absurd; the second was quiet because it was correct about
the sample it was run on. Nothing in its own output could have exposed it,
because the input had no getters in it.

## The cure

**Say which direction an instrument is sound in, in the sentence that offers
it.** Not in the method, not on request: in the claim. *Nothing returns it, so
nothing builds it — but something returning it proves nothing, because a getter
returns a field* is the same measurement with its shape attached, and it is one
clause longer.

**And prefer the direction that can only clear, never convict.** A check whose
zero is trustworthy and whose non-zero is not should be used to *shorten* a
suspect list and never to lengthen one. That is a rule about how to hold it,
and it survives the next reader in a way *be careful* does not.

**Found by the laptop lane**, running this lane's check against their own four —
which is the thing that has caught every instrument fault on this fleet today:
not the author testing it, but somebody else running it on data the author did
not choose.
