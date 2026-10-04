# A fixture its writer would produce cannot test preservation

**Concluded:** *`a_format_1_file_is_still_format_1_afterwards` proves the bytes
pass through unparsed.*

**True:** replacing the move with a parse-and-reserialise — the implementation
the design argues against — left **all eight tests passing**, including one
asserting byte equality. A fixture written by `kept()` round-trips to identical
bytes, so nothing could tell the two implementations apart.

**The mechanism.** The test compared the result against a file its own writer
had produced. Any implementation that goes through that writer agrees with it,
so the assertion could not fail for the reason its name claimed.

**The cure.** Test preservation with **a file the writer would never produce** —
here, one carrying a comment and a blank line, which `written.rs` does not emit.
Re-breaking the move then failed **that test alone while the other seven
passed**, which is the proof the first version never had.

**Generally:** break the thing and watch the **specific** new assertion fail. A
test that passes on a broken implementation is a test about something else.
