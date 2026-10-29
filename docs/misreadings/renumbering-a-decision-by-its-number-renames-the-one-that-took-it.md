# Renumbering a decision by its number renames the one that took it

**What was concluded.** On 2026-10-29 a branch's ADR had to move from 0138 to
0139, because another lane's ADR had landed first as 0138. The renumbering was
done as *every file that says `ADR 0138` now says `ADR 0139`*, across the tree.

**What was true.** After the rebase onto `main`, `ADR 0138` was **two**
decisions' number in the tree: the branch's own, and the one that had just
landed (`0138-a-follow-up-is-sent-with-the-conversation-so-far.md`). The
replace rewrote 36 files that cite the landed decision, in eleven crates. It was
caught at once, because the list of files it touched was printed and named crates
the branch had never opened. It was undone by restoring every file except the
branch's own.

**The mechanism.** A number stops identifying a decision exactly when you need
to change it. Renumbering is only ever needed **because the number is now taken**,
so by the time a renumber runs, the old number belongs to two documents, and
any rule that matches on the number alone matches both. The slug
(`0138-the-first-start-sets-…`) still identified one of them; `ADR 0138` did
not.

**The cure.** Renumber **by the files the branch itself added or changed**,
not by the number:

```sh
git diff --name-only <the branch's base> HEAD | xargs grep -ln 'ADR 0138\|0138-<own slug>'
```

Rewrite only those, and then grep the whole tree for the old slug, not the old
number, to confirm nothing still points at a file that has moved. The number
alone is never a safe search term once two decisions share it, and they always
do when a renumber is needed.
