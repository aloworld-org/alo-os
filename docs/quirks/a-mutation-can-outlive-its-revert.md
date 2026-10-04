# A mutation can outlive its revert, and the test binary is what answers

**What happened.** Two mutations were applied to `crates/alo-shell/src/panel_raster.rs` to
check that new assertions bite, each reverted afterwards from a byte-identical copy. Both
failed as intended. Then the suite ran against the reverted file and one test failed — with
the mutant's exact signature:

```
assertion `left == right` failed
  left: (1360, 104)     the expansion control's own position
 right: (1360, 160)     where the first window belongs
```

A window slot at the control's position is precisely what deleting the `which + 1` offset
produces. But the file on disk was correct:

```
diff -q panel_raster.rs panel_raster.rs.orig   ->  identical
grep -n "saturating_add(which"                 ->  which.saturating_add(1).saturating_mul(…)
```

**The code was right and the test binary was the mutant.** Rebuilding made it pass.

**Why.** This repository is on `/mnt/c`, a 9p mount. Cargo decides what to rebuild from
source mtimes, and a file restored within a short window does not reliably read as newer
than the artefact built from the version before it. The write is not missed; the new mtime
is not *distinguishable* from the old one.

## Why it is worth a quirk: it lies in the dangerous direction

- **Harmless:** a revert that still runs the mutant reports a failure in correct code. It
  wastes time and ends with the file being read and found right.
- **Dangerous:** the same staleness after *applying* a mutation reports the mutation as
  **not caught**, because the suite is still running the pre-mutation binary. The conclusion
  is *this test does not bite* — which invites somebody to strengthen a test that was already
  sound, or to accept a change no test was holding.

**The second produces a green suite and says nothing.** It is this directory's recurring
shape from a new direction: not a check whose inputs were less specific than its question,
but a check whose **subject was a previous version of the thing**. The test, the assertion
and the file were all correct; the only wrong thing was which build answered.

## What works, measured

```text
cargo clean -p <crate>      Removed 804 files, 7.3GiB total
then cargo test             Compiling alo-shell v0.1.0 … test … ok
```

**`cargo clean -p <crate>` before each run of a mutation experiment**, and again before the
run that checks the revert. It is slow — a full recompile of that crate — and it is the only
thing here that was demonstrated to work.

## What does not work, also measured

**`touch` is not reliable on this mount.** Neither a plain `touch` nor
`touch -d "now + 1 minute"` produced a recompile in a state where the content had not
changed, so it cannot be trusted to force one in a state where it had:

```text
clean state, no change              0 crates compiled   (correct)
after touch                         0 crates compiled   (no rebuild forced)
after touch -d "now + 1 minute"     0 crates compiled   (no rebuild forced)
```

A `touch` did appear to fix the original failure, which is why it is tempting. **One
success is not a mechanism** — it worked there because the content genuinely differed from
the artefact, and it is the *mtime comparison* that is unreliable, not the write.

## And do not measure any of this by counting

Three of the counts taken while investigating this were wrong, all from the same habit:

```
wsl -d Ubuntu -- bash -lc '… -> $(cargo test … | grep -c Compiling) Compiling'
```

A count collected through a command substitution inside a `wsl -- bash -lc` string reported
`1` where the run had not compiled and `0` where it had. The first experiment also ran from a
**dirty** starting state, so both of its arms genuinely recompiled and the discriminator
looked broken when the experiment was.

**Read the output.** `Compiling <crate>` appears on its own line when a crate is rebuilt and
is absent when it is not, and that line is legible in two seconds. The count of it, taken
across that boundary, is not evidence of anything — see
`a-backgrounded-process-does-not-survive-the-wsl-call-that-started-it.md` for the rest of
what that boundary does to a string.
