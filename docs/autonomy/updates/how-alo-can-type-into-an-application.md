# How alo can type into an application

**What this is:** the measurement `docs/autonomy/demo-two-alo-inside-applications-plan.md`
stage 6 waits on — ADR 0143 Q2, *how does alo type*, which the owner left to
engineering, to be decided on evidence. **Measured once, by hand, on
2026-11-13. No test or gate runs any of it again. Not on an alo machine.**
What would run again, and where, is under *What runs again*.

## What it means for a person

- **alo cannot put text into VSCodium the way a screen reader's dictation
  does.** VSCodium — and so VS Code, and anything built on Chromium — does not
  offer the accessibility interface for changing text at all: not on its
  editor, not on its terminal. The road ADR 0143 hoped was simplest does not
  reach the demo's editor.
- **It can type there the way a keyboard does**: the desktop hands the window
  key presses, which the window cannot tell from a person's. Measured here,
  that typed every character asked for — accents, Japanese, a dash, a tick,
  a tab and new lines — and pressing VSCodium's own *File → Save* through the
  accessibility tree saved it to disk.
- **But an editor rewrites what is typed into it**, as it would for a person:
  it indents after a `{`, closes brackets on its own, and turns a tab into
  spaces. With VSCodium's own settings the file on disk was **not** the text
  that was typed. With its automatic editing turned off, it was the same to
  the byte, twice.
- **GNOME Text Editor takes text the screen-reader way**, exactly, inserted or
  replaced whole. **Firefox 160 says it does, and nothing changes.**

So the demo's editor can be typed into only by the desktop delivering keys to
one window, and the person gets exactly the text they approved only when
VSCodium's automatic editing is off in the copy alo types into — which is a
choice about the person's application, or needs a copy of alo's own (stage 9,
not built). **Nothing is built by this change.** The options are at the end.

## The setup

WSL Ubuntu 24.04 (at-spi2-core 2.52) on the measuring machine; a real
accessibility bus started by at-spi2's own launcher inside a private session
bus; everything **run as root**, so VSCodium got `--no-sandbox`.

| Application | Where it came from | Started with |
|---|---|---|
| VSCodium 1.143.17232 | Flathub, `com.vscodium.codium` | `ACCESSIBILITY_ENABLED=1`, `--force-renderer-accessibility=complete` (and `=basic` once), `--ozone-platform=wayland`, a user-data directory of the script's own |
| GNOME Text Editor 46.3, GTK 4.14.5 | Ubuntu's package, **not** Flathub | `--standalone`, on Wayland |
| Firefox 160.0 | Mozilla's own build, **not** Flathub | `GNOME_ACCESSIBILITY=1`, Wayland, a fresh profile with the first-run pages off |

Two compositors: **Weston** (headless) for the screen-reader road, and **sway**
(headless) for the keyboard road, because Weston takes no key presses from a
program. `wtype` hands sway a keyboard layout holding the characters it needs,
then presses them — what a desktop delivering keys to one window has to do.
**alo's own desktop was not measured**: it has no code that delivers keys
(below).

Each check read **what changed**, never a call's answer: the text read back
through the tree, the window title gaining and losing VSCodium's unsaved `●`,
and the file read from disk with `cat -A`.

## The screen-reader road: AT-SPI's EditableText

| Application, control | Offers EditableText? | Insert at the start | Replace the whole text | Runs |
|---|---|---|---|---|
| VSCodium editor, `editor.editContext` on (the default): an `entry` named after the file, class `native-edit-context` | **no** | — | — | 3 (`=complete` twice, `=basic` once) |
| VSCodium editor, `editor.editContext` off: the older hidden `textarea`, class `inputarea` | **no** | — | — | 2 |
| VSCodium's built-in terminal: an `entry` named *Terminal 1, sh …*, class `xterm-helper-textarea` | **no** | — | — | 4 |
| Text Editor's document, a `text` node | yes | **yes**: answered true, and the document read back with the text in front (22→40 characters) | **yes** (40→24) | 2 |
| Firefox page, `<input>` | yes | **no**: answered true; text and page title unchanged | no, the same | 2, one after focusing it |
| Firefox page, `<textarea>` | yes | no, the same | no, the same | 2 |
| Firefox page, `contenteditable` | yes | no, the same | — | 2 |

**Confirmed:** Chromium offered EditableText on no node in any run, so nothing
alo sends through that interface reaches VSCodium. **Only fits:** that
Chromium's Linux accessibility does not implement the interface — its source
was not read. **For Firefox,** why a call that answers *true* changes nothing
is **not known**; the page's own `input` handler, which sets the title, never
ran either, which rules out a stale read on our side.

## The keyboard road: keys the compositor delivers to one window

The text typed, in every run:

```
function f(a) {
    return "é 日本 — ✓";
}
<div>
	end
```

| Application, settings | What the editor held afterwards | On disk after saving | Runs with a result |
|---|---|---|---|
| VSCodium, its own defaults | `unction f(a) {` / 8 spaces + `return …` / `}` / `<div>` / 4 spaces + `end` / **an extra `}`** | the same, saved by `Ctrl+S` | 1 (two more lost to the instrument) |
| VSCodium, its automatic editing off (list below), saved by pressing *File → Save* through the tree | exactly the text | **byte for byte the text**; the title lost its `●` | 2 |
| Text Editor, its defaults | `unction f(a) {`, then every later line indented by 4 | the same | 1 |

**The first character went missing** in both runs where the keys were pressed
the moment the layout was handed over, and in none of the four where `wtype`
waited 0.4 s first. That fits a race inside `wtype` and sway; **it was not
confirmed**, and alo's own delivery must be tested for it rather than assumed
past.

**The final new line was missing from every file**, and that one is the
instrument's: the script passed the text through `$(…)`, which drops it, and
`cmp` stopped at exactly that byte.

**The settings that made VSCodium type exactly:** `editor.autoIndent: none`;
`autoClosingBrackets`, `autoClosingQuotes`, `autoClosingOvertype` and
`autoSurround: never`; quick suggestions and suggestions on trigger characters
off; `acceptSuggestionOnEnter: off`; `wordBasedSuggestions: off`;
`tabCompletion: off`; `detectIndentation: false`; `insertSpaces: false`;
`formatOnType: false`; `html.autoClosingTags: false`; `linkedEditing: false`.
Which of them were needed was not separated. **Each is a setting of the
person's application.**

## What the tree tells alo about where it is typing

- VSCodium's editor is an `entry` whose **name is the file's name**, and
  `MULTI_LINE`; in complete mode its whole text reads back. So alo can say
  *which file* it is typing into, and can read the result back and compare it
  with what was approved before it presses *Save*.
- VSCodium's terminal is an `entry` named *Terminal …*, `SINGLE_LINE`, class
  `xterm-helper-textarea`, and when the panel opened **it took the focus from
  the editor**. A key press is not aimed at a control; it goes wherever the
  focus is. Whether that is the terminal can be read at the moment of typing,
  but the delivery itself cannot refuse it.

## The options

1. **Keys delivered to one window, with the tree as the check.** Recommended,
   because it is the only one measured to reach VSCodium. The desktop delivers
   the approved text as key presses to the named window and no other, only
   while the tree says the focused control in it is the named editable one —
   never a password field, never a terminal unless ADR 0111's choice allows
   it. Before *Save*, alo reads the text back and, if it is not what was
   approved, says so and does not save. It costs: key delivery in `alo-shell`
   and a request on the application door, neither of which exists; the adapter
   contract's refusal of `synthetic` amended for typing into one window; and
   VSCodium's automatic editing off where alo types — **either** in alo's own
   copy of VSCodium (stage 9's *a window of alo's own*, not built) **or** in the
   person's own VSCodium by their choice, offered with what it changes.
2. **The screen-reader road where it works, keys only where it does not.** GTK
   applications such as Text Editor take EditableText exactly and need nothing
   new in the desktop. It cannot reach VSCodium, so it does not serve the demo
   alone; it could be built first behind the same verb, with option 1 added for
   Chromium applications.
3. **Paste, or an input method committing the text in one piece.** By their
   documentation editors do not auto-indent what is pasted — **unmeasured
   here**. Pasting borrows the person's clipboard; an input method sees every
   key the person types, which ADR 0083 makes a grant nobody can make yet.
   Either needs measuring before it is chosen.

**Not an option:** writing the file behind the editor's back, which the owner
ruled out (ADR 0143, decision 3).

## What runs again

Nothing yet. When the verb is built, its refusals and *one approval types once*
are unit tests against a stand-in tree, and CI's real-window test
(`crates/alo-adapters/tests/the_accessibility_fallback_on_a_real_application.rs`)
can grow a GTK text field for the screen-reader road. **The keyboard road into
VSCodium can be checked only on a booted alo machine**, by the Demo 2 walk
(stage 13): a file typed and saved in the shipped editor, and read from disk.

## Not shown here

alo's own desktop delivering anything; a person who is not root, with the
sandbox on; Text Editor and Firefox as Flatpaks behind Flatpak's accessibility
proxy (VSCodium was a Flatpak, run without its sandbox); LibreOffice; a
password field in any of them; pasting; an input method.

The scripts (`typing-probe.py`, `menu-press.py`, `typ-vsc.sh`,
`typ-vsc-all.sh`, `typ-ff.sh`, `typ-other.sh`, `typ-keys.sh`, `typ-keys2.sh`,
`typ-keys-plain.sh`) are in `/root/typ/bin/` in WSL Ubuntu on the measuring
machine, and their outputs under `/root/typ/out/`. They are not in the
repository.
