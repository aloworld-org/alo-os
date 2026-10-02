# ADR 0083 — An input method is a grant, not a global

**Status:** **proposed, 2026-10-02.** Found while wiring the protocol by which
anything not typed in English reaches an application, which the plan calls one of
the five this shell does not speak.

## The protocol is a pair, and only one half is safe to advertise

An application binds `zwp_text_input_v3` to say *I will take composed text*. An
input method — ibus, fcitx, a handwriting panel, a phone's keyboard — binds
`zwp_input_method_v2` to **produce** it. Smithay sets text-input focus from the
keyboard focus automatically, so the application half needs no wiring beyond its
global. But:

```rust
// smithay/src/wayland/seat/keyboard.rs
text_input.set_focus(Some(self.clone()));
// Only notify on `enter` once we have an actual IME.
if input_method.has_instance() { text_input.enter(); }
```

**An application is never told it has the text input unless an input method
exists.** So the two halves are one feature: wiring only the first produces a
global that is advertised and can never fire, which is the fault this repository
has catalogued four times — a road built and not reachable.

## And the second half is a keylogger by construction

`zwp_input_method_v2` receives **every keystroke** destined for the focused
surface, before the application sees it, and can send any text it likes in return.
That is not a weakness of the protocol; it is what an input method *is*. A person
typing a password into a window has typed it to whatever holds that global.

`InputMethodManagerState::new` takes a filter — `Fn(&Client) -> bool` — precisely
because a compositor is expected to decide who may bind it. **This shell has no
answer to that question today.** There is no notion of a trusted client in
`alo-shell`: `ClientState` carries compositor state and nothing about who is on
the other end of the socket.

## The decision

**The application half ships. The input-method half does not ship until there is a
grant for it.**

```
zwp_text_input_v3       advertised now — an application may ask
zwp_input_method_v2     not advertised — no client may become the input method
```

A filter of `|_| true` would be the whole of the fault: any application that
connects could become the keyboard for every other one, silently, with no record
and nothing to revoke. `CLAUDE.md`'s standing rule is *no path, window, application
or device is reachable that a person has not granted*, and **an input method is a
device in every sense that matters** — it is the keyboard.

## What this costs, stated rather than hidden

**Today this means a person cannot type Chinese, Japanese, Korean, Vietnamese,
Thai, or any language needing composition, in any application.** That is a real
and serious gap in a product whose first target is twenty-four European languages
and which says *hardcoded English is a bug*. It is not fixed by this ADR and is
not pretended to be.

What the application half buys in the meantime is that applications may bind and
are correctly focused, so the day the grant exists nothing in the application path
has to change.

## What the grant would have to be

Not settled here — it is the owner's — but the shape is constrained by what
already exists:

- an input method is **named** and **chosen by the person**, the way a keyboard
  layout is, rather than claimed by whichever client asks first;
- it is **visible while it is active**, because something reading every keystroke
  is exactly what `docs/features.md`'s indicator rules exist for;
- it is **revocable**, and revoking takes effect immediately, which is the standing
  rule for every other grant;
- and it is **one**, because two clients holding it is two things reading a
  password.

The nearest existing model is `alo-access`'s grants rather than anything in the
shell, and the decision about where it lives is part of granting it.
