# A scripted replace that matched nothing said nothing

## What was concluded

Commit ba08725d, which landed in #1278, says: *"Now every sentence that
mentions ~ must say it is refused."* The commit and its pull request both
said the test had been strengthened that way.

## What was true

The test on `main` still checked only the exact words `"~ means"`, which is
the weaker check the commit said it replaced. The second half of the same
commit, about the example on the right side of "never", did land.

The edit was made by a script calling JavaScript's `String.replace`. The text
it searched for was never found, because the search string's escaping differed
from the file's. `replace` then returns its input unchanged, and nothing failed.
The tests passed, because the old check passed too. Found on 2026-11-15 while
reading the landed file for another reason.

## The mechanism

A replace that matches nothing is not an error in any tool used here:
`String.replace`, `sed s///` and `awk sub()` all succeed without changing
anything. A strengthened test that did not apply leaves behind the weaker test,
which still passes. So the edit's failure and its success look the same: green
tests, a clean diff of the other half, and a commit message describing
something that never happened.

A second fault from the same scripting turned up the same day. In a JavaScript
template literal, a Rust string's line continuation (`\` followed by a newline)
is eaten, and a word came out with six spaces in the middle of it.

## The cure

- Make every scripted replace prove it matched. Throw when the search text is
  absent, as `if (!s.includes(old)) throw` does. Or use the Edit tool, which
  refuses a search text that is not found.
- Read the changed lines back before writing the commit message that describes
  them.
- Never write a Rust string with a line continuation through a JavaScript
  template literal.
