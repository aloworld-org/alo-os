# The hosted runner's Ubuntu mirror stopped resolving

- **Version:** GitHub-hosted `ubuntu-24.04`, image `20261101.360.1`, runner
  `2.338.2`. Workflow `.github/workflows/gate.yml`, step *the libraries this
  workspace links, and a compositor for the fixtures that start their own*.
- **Whose:** the platform's — the runner's network and its package mirror.
- **Behaviour:** from about 14:07 UTC on 2026-11-11 every gate failed in that
  step. Job 130579513235 (run 43037327228, pull request #1179) is the one read:
  `apt-get update` took 3 min 20 s and ended with
  `W: Failed to fetch mirror+file:/etc/apt/apt-mirrors.txt/dists/noble/InRelease
  Temporary failure resolving 'azure.archive.ubuntu.com'` — a warning, exit 0,
  so the image's own stale lists were used. The download then printed
  `Ign:N http://azure.archive.ubuntu.com/ubuntu noble… <package>` once every
  20 s until `has timed out after 8 minutes`. Runs 43035720540 (#1174) and the
  queue candidate for #1176 failed the same way. `packages.microsoft.com`
  answered in the same second, so the network was up and one host was not.
- **What the mirror list did not do:** the sources say
  `mirror+file:/etc/apt/apt-mirrors.txt`, and that file lists
  `archive.ubuntu.com` and `security.ubuntu.com` after Azure's mirror. apt did
  print lines for both — and every one of their errors still read *resolving
  'azure.archive.ubuntu.com'*. Why is not known; what was measured is that the
  list's later entries did not rescue the run.
- **It was name lookup, not the mirror.** A first fix pointed apt at
  `archive.ubuntu.com` and failed the same way (job 130588464166, 14:37 UTC:
  `Ign:5 http://archive.ubuntu.com/ubuntu noble InRelease`, every 20 s). At
  14:40 UTC from a lane's Windows machine, whose resolver is FortiGuard's
  (96.45.45.45) and not Azure's: `archive.ubuntu.com`, `azure.archive.ubuntu.com`,
  `security.ubuntu.com` and `canonical.com` all gave *DNS server failure*;
  1.1.1.1, 8.8.8.8 and 9.9.9.9 answered (archive.ubuntu.com → 91.189.91.83
  and eleven more); and `curl --resolve archive.ubuntu.com:80:91.189.91.83`
  fetched `dists/noble/InRelease` with HTTP 200 in 0.53 s. Why some resolvers
  fail for Canonical's names and others do not is not known.
- **Our response:** the step fetches first, on its own clock. On failure it
  rewrites the sources to `http://archive.ubuntu.com/ubuntu/`, pins any of
  `archive.ubuntu.com` / `security.ubuntu.com` the runner cannot resolve to an
  address from Cloudflare's or Google's resolver (asked over HTTPS by IP), and
  fetches again. apt's signature check is unchanged, so the address is trusted
  no further than any mirror. A warning on the run and `apt path:` in the log
  say which happened. Same packages either way. The workflow comment above the
  step says why this is not a cache.
- **Date:** 2026-11-11.
