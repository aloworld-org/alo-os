# A walk tested without the frames between its steps

**Found 2026-11-01, while building redo on `⊞`+Shift+Z. Nothing caught it: the
tests passed for four weeks, and no boot walk had pressed the chord twice.**

## What was concluded

`crates/alo-shell/tests/a_place_remembers_time/` held, in
`the_canvas_a_person_leaves_is_itself_held`, that *pressing the chord twice
walks backwards through two held states instead of losing the middle one*, and
`canvas_a_place_remembers_time.rs` said the cursor made the chord *a walk rather
than a toggle*.

## What was true

On a running machine it was a toggle. Each step back moves windows, so on the
next frame `direct_desktop` sees the layout change, keeps it and calls
`Server::these_places_remember` — and that call ended the walk by clearing the
cursor. The second press then found the canvas it had just left as the newest
thing held and went forward to it.
`going_back_twice_walks_back_with_the_desktop_keeping_each_step` fails without
the fix and passes with it.

## The mechanism

**The test pressed the chord twice in a row, and the machine never does.**
Between two key presses a desktop frame always runs, and that frame does the one
thing that broke the walk. The test's own helper for keeping a layout was
correct, but it was called between arrangements and never between presses, so
the suite tested a sequence of events a person cannot produce.

A second, quieter difference sat under it: the helper carried the layout
forward from the seat's own copy, which a step back has already edited, and the
real desktop carries it forward from its file, which a step back never touched.

## The cure

When a test drives a sequence of a person's actions, **let a frame pass between
each pair**, with the frame doing what production's does. If production writes
through a second owner, here the desktop's file, model that owner in the test
rather than reusing the first owner's state. `TheDesktop` in
`a_place_remembers_time/mod.rs` is the shape.
