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
