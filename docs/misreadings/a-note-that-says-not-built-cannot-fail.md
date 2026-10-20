# A note that says *not built* cannot fail, and it outlives the building

**Concluded**, from reading
`crates/alo-shell/src/a_press_on_the_dock.rs` while tracing why a
chord reached nothing: *nothing on this machine can start an
application.* Written up that way to the owner's lane and used as the
reason a launcher would have nothing to open.

The enum said it in as many words:

```rust
/// Several windows: the person chooses. **Not built** — task 3.
OpenThePicker,
/// Nothing open: start it. **Not built** — nothing here can start an
/// application.
Launch,
```

**True.** Both are carried out, twenty lines below, in the same file:

```rust
WhatAnIconDoes::Launch => {
    self.asked_to_start.push(app.clone());
```

and the road from there is `crate::an_application_is_started`, which
sends each ask through a sender the shell is given —
`alo_portals::activated`, D-Bus activation, supplied in production at
`crates/alo-desktop/src/main.rs:1113`. The picker is the same: the ask
goes onto `Server::the_dock_was_asked_for` and its rows are built at
every draw by `Server::what_the_dock_has_open`, whose own table is
`crate::what_the_dock_has_open_tests`.

## The mechanism

**A note about what does not exist is the one claim in a repository that
nothing can falsify.** Every other kind of staleness has something that
notices: a renamed function breaks a call, a changed signature breaks a
caller, a number that moved breaks a test that asserts it. *Not built*
asserts an absence, and **the act of building it does not touch the
sentence that denied it** — the compiler has no opinion, no test fails,
and the note reads exactly as true on the day the road lands as it did
the day before.

It is worse than a wrong number, for the reason
`a-guard-that-cannot-fire-is-a-comment.md` names about guards: a reader
checking the claim finds the variant, finds the note, and stops. The
citation is true — the variant is there, the line is there — and it is
about a state of the world that has ended. **Nothing in the file points
down twenty lines**, and the reader has no reason to look, because the
note already answered the question they came with.

The second-order cost is the expensive one. A lane that believes an
absence **builds the thing**, and a second road to start an application
is not a tidy duplicate: `crate::an_application_is_started`'s own
header refuses reading an `Exec` line under the second law, and a lane
that did not know activation was already wired is a lane reaching for
exactly that.

## The cure

**Write the limit, not the absence.** A note that says *this is not
built* goes stale silently; a note that says *this starts an
application with a bus name and returns a refusal for one without*
breaks when either half stops being true, because both halves name
something a test can press.

And when a note does have to record an absence, **give it a date and
name what would end it** — *not built as of 2026-10-19; it needs a
sender, see `AsksAnApplicationToStart`* — so a reader meeting it a
month later knows it is a claim with an age rather than a property of
the code. The one in this repository that works like this is
`Action::Launcher`'s: it says what is undecided, names both candidate
surfaces, and a test asserts the dead-action list is **exactly**
`[Launcher]`, so building the surface fails the test and forces the
note out. **That is an absence somebody can fail.**

The cheap habit, when you find one at all: before quoting a *not built*
note as evidence, read to the end of the file it is in. Both of these
were twenty lines from their own dispatcher.

## And the absence with a clock on it, 2026-10-20

The two notes above were wrong about the present. This one was **right when it
was written and stopped being right**, which is the same mechanism with a date
inside it and is harder to see.

`crates/alo-shell/src/dock_raster.rs` passed the bar no focus rings at all,
under this:

```rust
// **No focus ring on the bar itself, and that is a gap and not a
// decision.** The keyboard reaches the two surfaces the bar opens; it
// does not yet reach the bar's own slots, so there is nothing here for
// a ring to mark. When it does, the rectangle is the slot's and the
// rule is `a_focus_ring`'s — the measurement is already in
// `alo_dock::measures`.
rings: &[],
```

Every sentence of that was true the day it was written. On **2026-10-18** the
bar gained a keyboard performer — `Server::the_keyboard_takes_the_slot_it_is_on`
— and the second sentence became false. Nothing in that change had any reason
to visit this file, and **no test could fail**: the clause the note hangs on is
a claim about what *another* module does not do yet.

What it cost is the part worth recording. F6 moved the keyboard onto the Dock
and the screen did not change by one byte, so four presses on a booted machine
were reported — by this lane, to another lane and to the owner — as **F6 does
not reach the Dock**. The chord was right, the handler was right, the
end-to-end trace another lane ran was right, and the screen was the liar.

**Two things make this one worse than a plainly stale note.** The note was
*accurate about intent* — *when it does, the rule is `a_focus_ring`'s* — and
the code could not honour it, because `a_focus_ring` was a function nested
inside another function that the bar could not reach. And the diagnosis it
produced pointed at the input path, which is the half that was working.

**The cure, beyond the one above.** A note whose truth depends on another
module's state names that module and what would end the wait, so the search
that finds the change also finds the note: *nothing here for a ring to mark
until `which_slot_on_the_bar` has a writer* would have been found by a grep for
`which_slot_on_the_bar` on the day it got one. And when a note says *the rule
is X's*, check that X is callable from where the note is — if it is not, the
note is describing a change nobody can make.
