# ADR 0153 — alo types into an application by keys in its own copy, and reads the text back before it saves

**Status:** accepted, 2026-11-13, by the owner. The decision reached this lane
**relayed by the coordinator**, who put the measurement in front of the owner and
asked; the words below are the owner's as relayed. It answers [ADR
0143](0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
Q2, which the owner had left to engineering on a measurement.

## What it changes for the person

alo can write into an application's text — a file in the code editor the
machine ships, a document in Text Editor — the way a person types, and they
watch it happen. Before anything is typed they approve a sentence that names
the application and the place, with **the whole text** beneath it. After it is
typed alo **reads the text back**, and if what is there is not what they
approved it says so and **does not save**. Their own copy of VSCodium is never
changed: alo types in a copy of its own, and only that copy's settings are
alo's.

## The measurement it rests on

`docs/autonomy/updates/how-alo-can-type-into-an-application.md`, measured once
by hand on 2026-11-13, not on an alo machine:

- VSCodium, and anything built on Chromium, offers **no** accessibility
  interface for changing text (five runs). The screen-reader road cannot reach
  the demo's editor.
- Keys a compositor delivers to the window type every character, and the
  editor saves them; but **VSCodium rewrites what is typed** — indents, closes
  brackets, turns tabs to spaces — unless its automatic editing is off. Off, the
  file matched the text byte for byte, twice.
- GNOME Text Editor takes text through the accessibility interface exactly.
  Firefox 160 answers that it did and changes nothing.

## What the owner decided

*"Keys, in alo's own copy."* Recorded in full, as relayed:

1. **alo types by key presses into one named window**, only while the
   accessibility tree says the control that has the keyboard in it is the
   editable one the person approved.
2. **It reads the text back and never saves if it differs.**
3. **VSCodium's automatic editing is switched off only in alo's own separate
   copy** — the copy the desktop already starts with a profile of its own
   (`crates/alo-launching/src/alos_copy.rs`, #1163). **The person's own VSCodium
   settings are never changed.**
4. **Applications such as GNOME Text Editor take the direct road**: the text is
   put in through the accessibility interface, behind the same verb.

## How it works

**One verb, two roads.** `accessible.type_text` names an application, the
editable control by its name, and the text. Which road carries it is decided at
the moment of acting, from what the control offers:

- **The direct road.** The control offers AT-SPI's `EditableText`. The text is
  inserted where the control's caret is, through that interface.
- **The keyboard road.** It does not. The compositor delivers the text as key
  presses **to the client whose window holds the keyboard**, and only after it
  has checked that this window is the named application's — alo's own copy of it
  at the default, the person's own window only where they chose that (ADR 0143
  Q6). The keys go **to that one client's keyboard and no other**: the
  compositor sends that client a keyboard layout of its own that holds every
  character of the text, the presses and releases, and then the person's own
  layout again, **in one uninterrupted run on the compositor's own thread**, so
  no key the person presses can arrive between them and no other window ever
  sees the layout or a key. It is not a virtual keyboard any program could use,
  and nothing on the session or the accessibility bus can ask for it — only
  `alo-agentd`, through the application door, for a turn the person approved.

**The first character.** The measurement lost the first character when keys
followed a new layout at once (it fits a race inside `wtype` and sway, not
confirmed). Wayland has no acknowledgement of a layout, and no signal to wait
for. What the protocol does guarantee is **order on one connection**: a layout
sent on the same connection before the first key is read by the client before
that key. So the layout and the keys go on the target's own connection, in that
order, and are never split between two connections as `wtype`'s are. **Whether
that is enough is measured on a booted machine**, and the read-back below is
what catches it if it is not.

**Checked before, checked after.**

- *At the moment of typing* the tree is read again: the named control must
  exist, be **the one control in that window that has the keyboard**, be
  editable, and be **neither a password field nor a terminal**. A terminal is a
  terminal application, or a node the tree describes as one — VSCodium's names
  itself *Terminal …* and is an `xterm-helper-textarea`. Typing into a terminal
  is ADR 0111's choice and the person's alone, so it is refused unless they made
  it.
- *After typing* the control's text is read back. **It must contain the
  approved text exactly**; if it does not, alo says what it found, and does not
  save. A read-back that cannot be made — a control whose text the tree does not
  show — is said too, and nothing is saved.
- *Saving* is `Ctrl+S` delivered the same way, once the read-back matched.

**One approval types once.** ADR 0143 Q1's default — one file, shown in full,
saved in the same approval — is what is built: one approved call types one text
into one control and saves it. *Each block on its own* and *everything for one
request* are the person's setting by ADR 0143 and **are not built by this
decision**; nothing a person has been offered is taken away, because neither has
been offered on a screen yet.

## The settings alo writes into its own copy of VSCodium

Written into `<alo's copy's profile>/data/User/settings.json` only, by the
desktop, when it starts alo's copy — never into the person's profile at
`~/.var/app/com.vscodium.codium/config`. Each is VSCodium's own setting, with
what it would otherwise do to typed text:

| Setting | Set to | Otherwise, a typed key… |
|---|---|---|
| `editor.autoIndent` | `none` | after a new line, indents the next line to match |
| `editor.autoClosingBrackets` | `never` | after `(`, `[` or `{`, adds the closing one |
| `editor.autoClosingQuotes` | `never` | after a quote, adds the closing quote |
| `editor.autoClosingOvertype` | `never` | types over a closing character it added instead of adding one |
| `editor.autoClosingDelete` | `never` | deletes a pair together |
| `editor.autoSurround` | `never` | wraps a selection in brackets or quotes |
| `editor.quickSuggestions` | off everywhere | opens suggestions while typing |
| `editor.suggestOnTriggerCharacters` | `false` | opens suggestions after `.` and similar |
| `editor.acceptSuggestionOnEnter` | `off` | **Enter accepts a suggestion** instead of making a new line |
| `editor.acceptSuggestionOnCommitCharacter` | `false` | a character such as `(` accepts a suggestion |
| `editor.wordBasedSuggestions` | `off` | suggests words from the file |
| `editor.tabCompletion` | `off` | Tab completes a word |
| `editor.snippetSuggestions` | `none` | offers snippets as suggestions |
| `editor.detectIndentation` | `false` | changes indentation to match the file |
| `editor.insertSpaces` | `false` | **Tab becomes spaces** |
| `editor.formatOnType` | `false` | reformats a line as it is typed |
| `editor.linkedEditing` | `false` | edits a matching tag as well |
| `html.autoClosingTags` | `false` | after `<div>`, adds `</div>` |
| `files.autoSave` | `off` | saves before alo has read the text back |

Which of these were needed was not separated in the measurement.
`autoClosingDelete`, `acceptSuggestionOnCommitCharacter` and
`snippetSuggestions` were **not** in the measured set (`files.autoSave: off` was
in every run's) and are added from
VSCodium's documentation, read, not measured. The read-back is what catches one
that is missing.

**A settings file alo cannot read is left as it is.** Settings in alo's copy can
be changed by the person in alo's copy, so the desktop changes only these keys
and keeps everything else; a file it cannot read as JSON is not overwritten, and
the desktop says so in its log. The read-back still holds.

## What this does not do

- It never writes a file behind an application's back (ADR 0143 decision 3).
- It never changes the person's own VSCodium, or any application's settings
  outside alo's own copy.
- It adds no input method, no clipboard use and no global synthetic keyboard;
  pasting and an input method were not measured and are not this decision.
- An adapter still cannot declare the `synthetic` mechanism
  (`docs/contracts/app-adapters.md`): the keyboard road is this machine's own,
  carried out by the compositor for the fallback's one verb.
