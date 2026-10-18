# Shell integration marks

Status: new public surface, 2026-10-18.
[ADR 0123](../decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
terms 1 to 3, which the owner accepted that day. No agent verb, no D-Bus
interface and no stored format changes; this is a byte sequence a shell writes
to its own pseudoterminal and alo OS reads.

**alo reads these, since 2026-10-18.**
`crates/alo-terminal/src/what_the_shell_says_about_the_prompt.rs` is the reader:
a second `vte` parser over the same bytes, as ADR 0123 term 7 calls for, leaving
`alacritty_terminal` untouched. *This paragraph said **nothing in alo OS reads
these yet** on the morning this document was published, and the reader was built
that afternoon.*

## Why a shell has to say it

alo cannot know what a person typed by looking. Reading it off the screen decides
what the **person** typed from what a **program** printed, and keeping a copy of
the bytes alo forwards diverges silently the first time somebody presses `Up`,
because the shell owns the line editor. ADR 0123 has the measurements. The only
component that knows what is in the line editor is the one that owns it, so the
shell says.

## The cycle: OSC 133, which is not ours

alo reads the semantic prompt marks every other terminal already reads:

| sequence | means |
|---|---|
| `OSC 133 ; A ST` | a prompt is about to be drawn |
| `OSC 133 ; B ST` | the prompt has ended; what follows is what the person types |
| `OSC 133 ; C ST` | the command is running; what follows is its output |
| `OSC 133 ; D ; <exit code> ST` | the command has finished, with that status |

`ST` is the string terminator — `ESC \` or `BEL`; both are read.

**A person who already has shell integration for kitty, iTerm2, WezTerm or VS
Code needs to do nothing.** Those emit these marks already. That is the whole
reason this is not an alo-specific sequence: an adapter author learns one
format, and somebody who has configured a terminal before keeps their
configuration.

Unknown parameters after the letter are ignored, which is how other terminals
already extend these marks. A sequence alo does not recognise is dropped and
changes nothing.

## The typed line: `alo-typed`

The cycle cannot carry a command **before it runs**. `B` fires when the prompt is
drawn, before there is anything to report, and `C` fires after `Enter`, when an
explanation is a history note rather than a warning. **No shell hook runs while
somebody is typing**, so the line has to be sent when the person asks for it:

```text
OSC 133 ; alo-typed ; <base64 of the command> ; <length in bytes> ST
```

- **`alo-typed`** is a namespaced token where a conforming reader expects `A` to
  `D`, so no reader can mistake it for a cycle mark, and the `alo-` prefix
  cannot collide with a key another terminal defines later. This is additive to
  OSC 133, not a number taken from somebody else's range.
- **the second parameter** is the line as it stands in the editor, base64
  encoded. Encoded because a command may contain `;`, `ESC`, or bytes that are
  not text at all.
- **the third parameter** is the length **of the decoded command, in bytes**,
  written in decimal — not the length of the base64. **Bytes, not characters**:
  in `bash`, `${#line}` counts characters and the two differ for anything
  outside ASCII, so a snippet declaring a character count has every accented
  command refused — and works perfectly in every ASCII test anybody writes.
  alo's own snippet uses `wc -c`, and a test with an accent in it fails if that
  changes.

### Why the length is there

**A payload that does not decode to exactly the declared length is refused**,
and the person is told the command was too long to read. They are never shown an
explanation of its first half.

*A single number cannot be caught being wrong. Two that must agree can.*

> **This section said something false when it was published, and it is corrected
> rather than quietly rewritten.** On 2026-10-18 it read: *`vte` caps an OSC
> payload at 1024 bytes and — measured in its own suite — dispatches the
> truncated payload rather than dropping it.* **It does not cap it.** Building
> the reader the same day and testing that claim found a 2005-byte command
> inside a 2699-byte payload arriving **whole**, in one read and in 64-byte reads
> alike. `vte`'s `Parser` holds `osc_raw` as an `ArrayVec` capped at 1024 only
> under `#[cfg(not(feature = "std"))]`, and as a growing `Vec` under
> `#[cfg(feature = "std")]`. **The cap is a `no_std` cap.**
>
> **The format is unchanged and the length stays**, for a weaker reason that is
> true: the cap is one cargo feature away, a `no_std` build truncates in silence
> with no flag to say so, and the check also catches a shell that mis-encodes.
> **An adapter author who already emits this sequence needs to change nothing.**

## What alo will not do with a mark

- **It never touches the gutter.** [ADR 0113](../decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)
  stands: who is speaking is decided by where bytes came from, never by what
  they say. A mark says where a prompt and a command are. It does not say who
  spoke, and a forged one cannot relabel anything.
- **A mark grants nothing.** It cannot run anything, cannot change or hold a
  grant, and cannot approve a change. Any program in the session can print this
  sequence, and that is survivable precisely because the sequence buys no
  authority: a forged mark can make alo explain a command nobody typed, and it
  cannot make alo *do* anything.
- **A mark is a claim, not a fact alo established.** Anything built on it carries
  that standing to the person, in the same words the recognised test result
  already uses: *Not independently verified*.
- **alo reads; it does not watch.** There is no mark between a prompt and a
  person asking, so there is nothing for alo to read while they type. That is
  the charter's *context is offered, never watched* holding by construction
  rather than by policy.

## What a snippet must do, and what it must not

A snippet emits the cycle marks from the shell's own prompt hooks, and emits
`alo-typed` from a line-editor widget bound to a key — `zle` in zsh,
`bind -x` in bash.

**It is installed by the person, never by alo.** alo OS ships the snippet, the
sentence saying plainly what it reveals — every command you run, to alo, on this
machine — and a one-press way to add it. alo does not write to anybody's shell
startup unprompted. Without it, the terminal works exactly as it does today and
the features that need it say so rather than guessing.

**alo OS ships one for `bash`, since 2026-10-18**, at
`crates/alo-terminal/shell-integration/alo.bash`, reached in code through
`alo_terminal::the_snippet_a_person_installs::for_the_shell_called`. **There is
no `zsh` snippet yet and that function answers `None` for it** — handing a `zsh`
user the `bash` one would be worse than nothing, because `bind -x` is not a
`zsh` builtin and the error would arrive on every terminal they opened.

**The one-press way to add it is not built.** What surface offers it is a
decision nobody has made, and it is named here rather than left to be
discovered.

*The snippet is held to a real interactive `bash` on a real pseudoterminal
rather than to a reading of itself: three tests source it, press the key, and
assert what alo reads back.*

## What this document does not settle

**Which key the person presses** is not part of this format, and still is not:
the binding lives in the snippet and this document describes what goes on the
wire. Any key that reaches a line editor produces the same bytes.

*For the record rather than as a requirement: the owner chose the shell's
`Ctrl`+`X` prefix family on 2026-10-18, and alo's own `bash` snippet binds
`Ctrl`+`X` `Ctrl`+`A` — measured unbound that day, while `Ctrl`+`X` `Ctrl`+`E` is
`edit-and-execute-command`. **A snippet somebody else writes may bind anything**
and alo reads it the same.*

**What alo does with a command once it has read it** — which model answers,
where, and what the person is told about that — is ADR 0123's terms 8 and 9 and
`docs/features.md`'s promise, not this surface. The owner ruled on 2026-10-18
that all three places may answer: on this machine, on this network, or with a
provider.

## Changing this

A public surface, so it changes additively and a break needs versioning and
deprecation. A new parameter is a new `alo-` key that older readers ignore. The
cycle marks are not ours to change at all; they are read as everybody else emits
them.
