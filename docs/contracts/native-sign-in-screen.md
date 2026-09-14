# Native sign-in screen

Status: additive trusted Rust API in `alo-shell`, 2026-09-14. ADRs 0002 and
0024; v0.01 *firmware to sign-in*. No protocol, agent capability, D-Bus
interface or stored format changes. The decisions it draws are
`alo-greeting`'s; this document describes only the drawing and input surface.

## Values

`SignInScreen::of(greeting, strings)` takes the result of
`alo_greeting::Greeting::at` (or `Greeting::of`) and the person's `Strings`. A
store that would not be read becomes a screen standing at `alo-greeting`'s
unreadable sentence; a store with no accounts stands at its *make an account*
sentence. Neither takes keys, and no account is ever created from this screen.

`SignInScreen::pressed(self, key)` consumes the screen and returns
`Signing::Still(Box<SignInScreen>)` or `Signing::HandedOver(alo_accounts::Session)`.
`HandedOver` holds no screen: after it, nothing can be drawn or typed at.
Enter in the name field moves to the password field; Enter in the password
field calls `Greeting::signs_in` exactly once and forgets the name and password
before looking at the answer.

`SignInScreen::shows()` returns `SignInShows::Fields { name, password_typed,
field, said }` or `SignInShows::Sentence(said)`. It never exposes the password,
only whether one is typed. `said` is always a `Said` produced by `alo-greeting`
(`Greeted::said`, `Standing::said`, `NotReadable::said`).

`SignInScreen::for_the_maintainer()` returns English naming the opener's socket
or the refused accounts file, for the host's own log. It contains nothing typed.

`SignInKey::of(keysym, modifiers)` maps BackSpace to `Erase`, Tab and
ISO_Left_Tab to `OtherField`, Return and KP_Enter to `Enter`, a printable
character with no Control, Alt or logo modifier to `Letter`, and everything
else to `Nothing`. `NAME_BYTES` (256) and `PASSWORD_BYTES` (1024) bound what is
taken.

## Input

`Server::sign_in_key(code, state, time)` takes a Linux evdev code through the
seat's XKB state, clears keyboard focus, intercepts the key so no client
receives it, and returns `None` for releases, repeated presses and unmatched
releases. It refuses codes outside `1..=0x2ff` (`InputError::InvalidKey`) and a
display without a keyboard (`InputError::Unavailable`).
`Server::sign_in_keys_released()` releases every held key without forwarding.

## Nested backend

`Nested::pump_sign_in(server, screen)` routes the parent window's keyboard
events, in order, to the screen. Unfocused parents route nothing and release
held keys. A handover is returned even if the parent closed in the same pump.
Invalid key codes are ignored. `RenderError::Closed` and `RenderError::Input`
drop the screen, zeroing what was typed.

`Nested::submit_sign_in(screen, labels, look)` lays the screen out for the
parent's current size with the bundled font from `WindowControlLabels` and
`SignInLook { scheme, scale }`, and submits it as the whole output: no clients,
popups or controls, default cursor. Outputs narrower than the scaled 152 px or
too short for two fields, or wider or taller than 16,384 px, refuse with
`RenderError::SignInScene` before anything is drawn.

## Drawing

Ground, field and ink colours are `alo-appearance` tokens: Porcelain, Cream
and Navy in the light scheme; Charcoal and Cream in the dark. Terracotta is
never used. The waiting field has a three-pixel edge and a caret; the other has
a one-pixel edge. The password field draws one band of fixed width once
anything is typed, never a mark per character. The name is drawn as typed on
one line, scrolled so its end is visible.

## Not provided

Field labels (no vocabulary declares them), a direct-display (DRM) submission,
pointer input, accessibility protocol and account creation.
