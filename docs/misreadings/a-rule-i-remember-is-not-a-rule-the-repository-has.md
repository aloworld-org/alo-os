# A rule I remember is not a rule the repository has

**Concluded, and enforced for most of a day:** *a `**bold**` span must open and
close on one line.* Assertions were written to check it, prose was reshaped to
satisfy it, and several writes failed on it.

**True:** measured across `docs/decisions`, `docs/autonomy` and
`docs/contracts` — **144 of 164 files wrap a bold span across a newline, 4823
spans in all**, and `STATE.md` alone has 925. The task being edited wrapped one
two lines above the insertion point.

**The mechanism.** The rule began as a real fault — an **unclosed** `**`, which
does break rendering — and was generalised from *every marker must close* to
*every marker must close on its own line*. The stricter form is checkable, which
made it feel more rigorous rather than less correct.

**The cure.** Before enforcing a style rule from memory, grep the repository for
whether it holds. Assert that `**` markers are **balanced across a block**,
after stripping inline code and fenced blocks — never per line.
