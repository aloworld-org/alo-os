# The fix that made the number grow

**Concluded:** *216 feature surfaces are built, tested, and never brought into
being by anything a machine runs.* Sent to two other machines as the list of
code worth writing before an evening of testing, with their slices broken out by
crate.

**What was true.** The number was 123, and the detector behind it was wrong four
times in under an hour. Three of the four were found by other lanes. Only the
second one is unusual, and it is the reason this entry exists.

## The four

**One — `Type::` does not catch a struct literal.** `Popup` was reported as
never built; `crates/alo-shell/src/popups.rs` builds a `Popup { … }` with no
`::` anywhere. *Found by the Mac, in their own crate, before the list was acted
on.* Their framing is the one to keep: **a hit is a question and a miss is not
an answer.**

**Two — the fix made the count go up, and that was the signal.** Adding
`Type\s*\{` to the match took the list from **216 to 243**. A correction that
moves a count the wrong way says the mechanism is not the one you fixed. The
real fault was underneath: the detector excluded each type's own file, so
`popups.rs:126` — *the exact line the Mac had pointed at* — was being discarded
before any pattern ran. A type built by a production function in its own module
is built; that function needing a caller of its own is a different question, and
conflating the two reported the evidence against the finding as proof of it.

With both fixed: **123**.

**Three — `-> &Type {` is a function signature.** The struct-literal form then
matched `pub const fn taking(&self) -> &Screenshot {`, where the brace opens a
body. `Screenshot` and `Recording` flipped from test-only to built on the
strength of a getter returning a reference to one. **A true statement was one
step from being retracted**: *nothing on a machine takes a screenshot* had been
reported to the owner, the corrected detector appeared to refute it, and the
retraction was avoided only by printing the matching line instead of believing
the new answer.

**Four — a guard is built by a test on purpose.** `alo-image`'s `TheFilesystem`
exists to be constructed by a test; its own doc says *so a second one is a
failing test rather than a machine that cannot undo*. For a guard, *nothing in
production builds it* is the design and not the defect. A detector hunting
unwired types flags every guard in the repository and is wrong about all of
them. *Found by the third PC, in their own crate, after being asked to check.*

## The mechanism

**A moving number reads as progress.** 216 → 243 was the detector reporting that
the fix had missed, and the ordinary reading of a count that moved after a
change is that the change did something. It did: it added a second wrong answer
beside the first. Had the number gone *down* to 190, the same unfixed mechanism
would have been invisible and the list would have shipped.

The third PC said it better than this entry can, and it is theirs:

> A correction that moves a count the wrong way says the mechanism is not the
> one you fixed, and most people read a moving number as progress.

## The cure

**When a fix moves a count in the direction the fix cannot explain, the fix did
not reach the fault.** Not *recheck the number* — the number is fine. Find the
mechanism that could produce movement in that direction and ask whether it is
the one that was changed.

And when a corrected instrument contradicts something already reported, **print
the line it matched before retracting**. That step is what saved *nothing on a
machine takes a screenshot*: the new answer was `pub const fn taking(&self) ->
&Screenshot {`, and reading it took a minute where believing it would have
unsaid a true thing to the person relying on it.

And beside it, the one all four share, which the third PC reached from a
different instrument the same night after their unit scan missed
`alo-convertd.socket` by reading `multi-user.target.wants` for `.service` names:

> **A check that only knows one activation mechanism will report every other
> kind as absent.**

Name-matching knows the forms it was taught. `Type::` and `Type {` do not see a
factory, a `Default`, a trait object or a socket. That limit does not go away
with a better pattern; it is a fact about asking a text question of a program.

## What survived

The finding did. **`SettingsWindow`, `Screenshot`, `Recording` and `NightLight`
are built only by tests**, each confirmed by hand rather than by the detector,
and `ROADMAP.md` ticks *Capture*. A person cannot take a screenshot on an alo OS
machine. The check that holds those four is
`crates/alo-citing/tests/every_surface_a_person_uses_is_built_somewhere.rs`, and
it carries all four corrections in its own words.

**What did not survive is the 216**, and the distance between it and 123 is the
distance between a list and a list somebody can act on. Three lanes found three
faults in one detector in under an hour. One lane would have sent 216.
