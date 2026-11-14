# A membership in a group only `/usr/lib/group` holds is written nowhere

- **Version:** the pinned base,
  `ghcr.io/aloworld-org/fedora-bootc@sha256:5a3472162ba7781a01d249b9a56bc5a9b9fa01c08edae1f73c77dfa1e504e868`,
  its own `systemd-sysusers`, in podman under WSL Ubuntu on the Windows lane
  machine, and on a GitHub `ubuntu-24.04` runner.
- **Whose:** the platform's — systemd-sysusers with Fedora's nss-altfiles.
- **Date:** 2026-11-14.

## Behaviour

The base keeps most system groups in `/usr/lib/group`, which NSS reads through
`altfiles` (`group: files [SUCCESS=merge] altfiles [SUCCESS=merge] systemd`).
`wheel` is in both files; `audio` is in `/usr/lib/group` only:

```
/etc/group:      wheel:x:10:
/usr/lib/group:  wheel:x:10:   audio:x:63:
```

A `sysusers.d` file with `m tst audio` and `m tst wheel`, then
`systemd-sysusers`, exit code 0, no warning:

```
/etc/group after:  wheel:x:10:tst      (no audio line at all)
id -nG tst:        tst wheel           (no audio)
```

sysusers finds `audio` through NSS, so it does not create it, and then has no
`/etc/group` line to add the member to. It writes the membership nowhere and
says nothing. With `grep '^audio:' /usr/lib/group >> /etc/group` first, the
same run gives `audio:x:63:tst` and `id -nG tst` lists `audio`.

This is what stopped the image building from `main` at `fafcc6d9`, at the
assertion `id -nG alo-greeter | ... grep -qx audio` added for the screen reader
before sign-in (#1245). The Walk workflow's first run on `main` (2026-11-14,
run 43626601619) failed there; the walk never started.

## Our response

`image/Containerfile` copies the `audio` line into `/etc/group` in its own step
just before `systemd-sysusers`; if the base ever has no `audio` at all, that
step fails the build. `the_screens_before_sign_in_speak.rs` holds the recipe to
the step. **Any new `m <login> <group>` line** for a group the base keeps only
in `/usr/lib/group` needs the same copy, and the assertion beside it is what
catches a missing one, at build time.
