# A GitHub release file is served from another host

- **Version:** github.com, 2026-11-17, asked with `curl -sI` from the market
  lane's Windows machine.
- **Whose:** the platform's (GitHub).
- **Behaviour:** a release address answers with a redirect, not the file:

  ```
  $ curl -sI https://github.com/storytold/photocraft/releases/download/v0.23.1/photocraft-0.23.1-linux-x86_64.flatpak.asc
  HTTP/1.1 302 Found
  Location: https://release-assets.githubusercontent.com/github-production-release-asset/139…
  ```

  The bytes come from `release-assets.githubusercontent.com`, a host the
  person's machine was never told it would reach.
- **Our response:** `alo_software::fetching::TheNetwork` follows **one**
  redirect, and only to `https://release-assets.githubusercontent.com/`
  (`WHERE_GITHUB_KEEPS_RELEASE_FILES`); any other `Location` is refused. The
  egress indicator shows the installation to `github.com`, where the machine
  was asked to go. The file host is GitHub's own and is named here; whether the
  indicator should also name it is not decided, and nothing checks it on a
  booted machine yet. What makes the file trustworthy is not the host: it is
  the pinned SHA-256 and the pinned key's signature (ADR 0158).
- **Date:** 2026-11-17. Measured once, by hand; nothing re-runs it.
