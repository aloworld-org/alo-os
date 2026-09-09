# Reclaiming development storage

2026-09-09. Owner-approved cleanup and smaller development builds. No release
requirement, security decision or test gate changed.

## Completed cleanup

No Windows or Linux Cargo/rustc worker was running when cleanup began. The desktop
supervisor had already stopped on its disk reserve. Both exact Linux targets were
resolved and their Cargo cache tags checked before invoking `cargo clean` with
an explicit target directory:

- `/root/ficina-target`: 5,667 files, 2.3 GiB reported removed.
- `/root/alo-os-target`: 17,986 files, 31.1 GiB reported removed.

These are permanent removals of regenerable build artifacts, not source deletion.
Git status was identical immediately before and after cleanup. Claude's
`/root/target-claude`, local models, the native Windows build cache and all Windows
system files were untouched. The focused check below subsequently recreated a
small `/root/alo-os-target`.

Linux filesystem usage fell from about 81 GiB to 49 GiB. This did not return the
same amount to Windows: C: remained about 12.15 GiB free and the Ubuntu VHDX file
remained 88,886,738,944 bytes (about 82.78 GiB).

## Host-space recovery is pending

The account is not elevated. `Get-VHD` against the exact Ubuntu disk failed with
"You do not have the required permission to complete this task." No compaction,
WSL shutdown, Windows configuration change or alternative permission bypass was
attempted. The administrator must coordinate shutdown and safe compaction of:

`C:\Users\SBW\AppData\Local\wsl\{53a36f96-4547-4d31-a5fa-72cf9a934ecd}\ext4.vhdx`

Do not delete that file, unregister Ubuntu or reinstall the distribution. It
contains the retained Linux environment, models and the other worker's cache.
Microsoft explains that deleting files does not automatically shrink a dynamic
VHD: <https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/compact-vdisk>.

## Smaller subsequent builds

Root `Cargo.toml` now sets development and test `debug` to `line-tables-only`.
This keeps filename/line backtraces, but omits full type/variable debug data.
Assertions, overflow checks, optimization and panic defaults are not changed.
Release and the separately configured BPF workspace are unchanged. No assertion,
test or gate was removed. This is a size reduction, not a measured prediction of
the next full workspace build's footprint.

Cargo profile semantics: <https://doc.rust-lang.org/cargo/reference/profiles.html#debug>.

## Verification and handoff

After a fresh C: check above the 12 GiB floor, Linux ran:

```text
CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-bounding-map --locked --offline -vv
```

Result: 26 unit tests and three documentation tests passed. Verbose rustc commands
showed `-C debuginfo=line-tables-only` for both the library and its unit-test
binary. This focused host test is not a full workspace or loaded-BPF gate.
Workspace `cargo fmt --all -- --check` and `git diff --check` also passed.

The existing unpublished native window-control painter work is preserved. Full
combined-tree verification, commit and push remain pending. Keep the desktop loop
stopped until storage is reviewed, recheck host free space before each build/test
phase and keep only one build workstream active. Claude's checkout was not edited.
