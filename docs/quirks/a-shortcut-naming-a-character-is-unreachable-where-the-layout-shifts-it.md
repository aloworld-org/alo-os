# A shortcut naming a character is unreachable on a layout that only reaches that character with Shift

**Version:** `alo-shell`'s `settings_chord`/`settings_seat` at 2026-09-28, against
xkeyboard-config as shipped in the Lima VM (Ubuntu 24.04.4 aarch64),
`/usr/share/X11/xkb/symbols/`.

**Whose:** ours, in how a press is named — not the layout's, and not xkb's.

**Behaviour.** `alo_shortcuts::Key` means *the key that prints this character*,
and `docs/design/the-shortcuts-and-the-edges.md` says so in as many words:
*"Plus and Minus mean the keys that produce `+` and `−` on the person's own
keyboard."* But a chord is named from the key's **unshifted** symbol —
`settings_seat` hands `chord_of` the result of
`raw_latin_sym_or_raw_current_sym`, deliberately, so that Shift stays a modifier
a person can hold rather than something swallowed into a key's name. So a `Key`
is reachable only where the character it prints sits at **level 1** of some key.

Read out of the layout data itself rather than reasoned about:

```
$ grep -E "AE12" /usr/share/X11/xkb/symbols/us
    key <AE12>	{[   equal,	 plus		]};
$ grep -E "AD12" /usr/share/X11/xkb/symbols/de
    key <AD12>	{[       plus,   asterisk,       asciitilde,         macron ]};
```

`de` has `plus` at level 1; the default `us` has it at level 2 and nowhere at
level 1. Of every `xkb_symbols` block in the `us` file, exactly two put `plus` at
level 1 — `dvp` and `drix`, both variants nobody is given by default. So
`Super`+Plus arrives from the main block on a German keyboard and does not on an
American one, from the same binding, with nothing saying so.

**It arrives as a failing test rather than as a dead key, but only because a test
asks.** `a_press_is_named_the_way_alo_shortcuts_names_it` walks `Key::ALL` and
asserts each is reached by some symbol. Adding `Key::Plus` to `alo-shortcuts`
without a symbol naming it failed there with `Plus cannot be pressed` — which is
the whole value of that test: a chord that can never arrive is otherwise
indistinguishable from one nobody has pressed yet.

**Our response, and what it does not do.** `Keysym::plus | Keysym::KP_Add` now
name `Key::Plus`, and `Keysym::minus | Keysym::KP_Subtract` name `Key::Minus` —
the pairing `Return | KP_Enter` in the same table already had. The numpad is a
real second route, measured the same way:

```
$ grep -h -A1 "key <KPAD>" /usr/share/X11/xkb/symbols/keypad
    key <KPAD> { [      KP_Add, KP_Add, KP_Add, KP_Add, XF86_Next_VMode ] };
```

`KP_Add` is at level 1, so `Super`+numpad-plus works on every layout that has a
numpad. **A laptop without a numpad and a US layout still cannot press
`Super`+Plus**, and that gap is named rather than closed: closing it means asking
the keymap which keycode carries `plus` at *any* level, the keymap is held at the
seat rather than in `settings_chord`, and a `Key` reachable with Shift held
changes how **every** chord is named. That is its own change, and inventing it
inside the symbol table would be the wrong place.

**One measured caveat we have not handled.** Some keypad variants give level 1 as
`0x100002B` — `+` in the Unicode-keysym form — not `Keysym::plus` (`0x2b`). Those
are different raw values, and `key_of` matches raw values, so such a variant would
fall through to `None`. Whether xkbcommon normalises the two before we see them
was **not** measured; it is a possibility read out of the same file, not an
observed failure.

**Date:** 2026-09-28.
