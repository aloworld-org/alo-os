# A test clock on whole seconds hid a file that rounds

**What was concluded.** `alo_arranging::keeping::keep` was tested: twenty-odd
tests, a read-back proof that refuses any file not reading as the layout meant,
and a test that removing the proof's body fails. It was taken as working.

**What was true.** On a booted machine on 2026-11-09 every keep after the first
rearrangement was refused, five times in an hour:
*the canvas layout was not kept: ReadBackAsSomethingElse*. A stop on a Place's
ribbon was held in memory at the moment the shell's clock gave
(`SystemTime::now()`, nanoseconds) and written to the file in whole seconds. The
proof read the file back, found a different moment, and refused — correctly.
Nothing after the first move reached the disk.

**The mechanism.** Every test built its moments as `UNIX_EPOCH +
Duration::from_secs(n)`, for a good reason (a series ordered by the test, not by
a clock), and that reason also made every moment already whole. The one value
in the format that loses precision was never given precision to lose. The
fault *passed*; nothing caught it until a machine ran with a real clock.

**The cure.** Where a format is narrower than the type it stores, a test feeds
the type's full width — a moment part-way through a second, a float with a
fraction, a string with a character the format escapes — and asserts the round
trip. Fixed test clocks are fine; make one of them not round.
`a_rearrangement_is_kept_at_a_moment_part_way_through_a_second` is that test
here.
