# The serial port tears a journal line where a kernel line lands inside it

- **Version:** an alo OS walk image built from `main` at `912a756e`, booted in
  qemu 8.2 under KVM on a GitHub-hosted `ubuntu-24.04` runner, serial port to a
  file (`-serial file:`).
- **Whose:** the platform's — two writers share one serial port: the kernel's
  console, and journald forwarding to it (`TTYPath=/dev/ttyS0`).
- **Behaviour:** a kernel audit record was written into the middle of a
  journal line, and the journal line's end came on the next line by itself:

  ```text
  [  127.851670] alo-shipping[2168]: 8 of this machine's applications arrived, 0 s[  128.234364] audit: type=1130 audit(1794666039.907:235): pid=1 uid=0 … res=success'
  till owed
  ```

  A walk check that the line says *0 still owed* failed, on a machine where all
  eight applications had arrived.
- **Our response:** `crates/alo-walking/src/the_log.rs` reads the log as
  records rather than lines — a timestamp in the middle of a line starts a new
  record, and the next line with no timestamp finishes the one it interrupted —
  and reads only whole lines. `a_line_torn_by_a_kernel_line_is_mended` holds
  it. **Anything else that greps this serial log for a whole sentence has the
  same exposure**, including the walk scripts by hand on the lane machine.
- **Date:** 2026-11-14, seen once, on the second walk run on GitHub. How often
  it happens is not measured.
