# Cargo's default job count does not parallelise on a memory-tight guest, it swaps — and the guest then stops answering

**Version:** the development PC's WSL2 guest, 12 CPUs and 7,940 MB, `CARGO_BUILD_JOBS`
unset; and the Mac lane's Lima VM, 6 CPUs and 4 GiB, 2026-09-27.

**Whose:** ours, in how we invoke the build.

**Behaviour:** cargo defaults to one job per CPU. A linker costs **450–730 MB**,
measured on the Mac lane, so twelve of them do not fit in 7.9 GB. On the Mac's
4 GiB VM a full rebuild at the default reached **7,016 MB of 8,191 MB swap with 61%
of CPU in I/O wait**, and one linker ran 39 minutes while its log had not moved in
38 — it was not compiling, it was swapping.

**The failure is total rather than gradual.** The machine does not get slower; it
stops answering. On this guest `wsl.exe` then refuses new connections with
`Wsl/Service/0x8007274c` while the VM itself is working perfectly — three cores of
CPU, memory fine, the build progressing. Every probe times out and the timeout
means nothing.

**Measured here, and it does not support a general ratio.** Two runs on the same
trigger, `touch crates/alo-appearance/src/lib.rs` then `cargo test --workspace
--no-run`, sampling from outside the guest because a sampler inside one that stops
answering records nothing:

| jobs | seconds | peak memory used | peak swap |
|---|---|---|---|
| 4 | 165 | 2,209 MB | 183 MB (baseline, untouched) |
| 8 | 109 | 3,126 MB | 184 MB (baseline, untouched) |

About 230 MB per job over a 670 MB baseline — well under the 450–730 MB linker
band, because link steps rarely coincide on a *partial* rebuild. **The full-rebuild
case, which is where the Mac's failure happened, is untested here.**

**Our response:** `CARGO_BUILD_JOBS=4` in this lane's gate, and 2 on the Mac's.
Neither is an established ceiling — the Mac's is what it dropped to after six
failed, and mine is measured safe rather than measured maximal. Do not inherit
either as a ratio; measure your own, and watch `free -m` while it links rather than
trusting a number from another machine.

**Date:** 2026-09-27.
