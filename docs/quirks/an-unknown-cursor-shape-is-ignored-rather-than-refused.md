# An unknown cursor shape is ignored rather than refused

- **Version:** Smithay 0.7.0, `src/wayland/cursor_shape.rs`; `cursor-shape-v1`
  from `wayland-protocols` 0.32.
- **Whose:** Smithay's. alo OS uses its cursor-shape delegate unchanged and
  patches no engine.
- **Date:** 2026-11-06.

## What the specification says

`wp_cursor_shape_device_v1.set_shape` takes a `shape` from the protocol's
enumeration, and the interface defines an `invalid_shape` error: *the specified
shape value is invalid*. A client that sends a number outside the enumeration
should be disconnected with that error.

## What happens

Smithay's dispatcher matches `SetShape { shape: WEnum::Value(shape), .. }` and
answers every other `SetShape` with a comment, *ignore unknown shapes*. No error
is posted. The pointer keeps whatever it showed, and the client carries on as
if its request had been honoured.

## What it means for a person

Nothing they can see: an application that asks for a shape nobody has named
gets the cursor it already had, which is what a person would want. What is lost
is the signal to the application's authors that it sent rubbish.

## Where it is held

`crates/alo-shell/tests/cursor_shapes/mod.rs`,
`an_unknown_shape_changes_nothing`, sends shape 9999 by hand on every build and
asserts that the pointer is unchanged **and** that no protocol error arrived.
The day Smithay starts raising `invalid_shape`, the second half fails and says
so, and the test should then assert the error instead.
