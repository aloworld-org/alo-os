# A contract sentence no test held

**What was concluded.** `docs/contracts/how-code-runs-file.md`, from the day it
was written (2026-10-22): *A key nobody declared is refused rather than read
around.* Every lane that read the contract, this one included at first, took
that as a fact about the file.

**What was true.** Nothing refused one. `alo_remembering::the_level`'s `Written`
struct had no `#[serde(deny_unknown_fields)]`, and no test fed it an unknown
key. `serde` reads around keys it was not told about, so `in-the-termnal =
"at-full-trust"` would have parsed as a file saying nothing about the terminal.
The person would have been told nothing, and alo would have gone on asking. It
was found on 2026-11-06 because a second key was being added. It was caught by
reading the struct, not by anything that ran.

**The mechanism.** A contract is written beside the code and in the same voice
as the code's own guarantees. So a sentence about behaviour reads as already
measured. The other rules in the same list did each have a test, which made the
untested one look the same as the rest. *Nothing failed*, because a rule that is
never exercised cannot fail.

**The cure.** When a contract states a refusal, find the test that feeds the
refused input before you rely on it. If there is no such test, the sentence is
a plan. Here the test is now
`the_level_tests::a_key_nobody_declared_is_refused`. It also checks that a later
format is still refused for being later, before any key is judged.
