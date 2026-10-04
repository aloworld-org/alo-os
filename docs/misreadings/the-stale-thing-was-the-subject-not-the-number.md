# The stale thing was the subject, not a number beside it

**Concluded**, from two screenshots a week apart, and reported to the owner:
*alo OS boots to a Fedora text login, so a person still cannot see an alo
screen.*

**True.** The machine in those screenshots has no screen in it. Read off the
installed disk, read-only:

```
usr/bin on the deployment      alo-agentd          — the entire list
usr/lib/systemd/system         alo-agentd, alo-boundaryd, alo-convertd,
                               alo-modeld, alo-sessiond
alo-compositor, any file       not present
```

and the dates say why:

```
the compositor entered the image   2026-09-28   #213
the kept base was deployed         2026-09-26   19:45
```

**The base is two days older than the screen.** A Fedora text login is the
correct behaviour for that build. The picture was real, dated and about a
different artefact than the one the question was about.

## Why it survived so long

`Machine::is_kept` decides whether to reuse the cached machine:

```rust
[windows_of(yard, base), second_of(yard, base), variables_of(yard, base)]
    .iter()
    .all(|file| std::fs::metadata(file).is_ok_and(|about| about.len() > 0))
```

**Three files exist and are non-empty.** Nothing asks what image they came from,
or when. So every walk since 28 September tested a pre-compositor system and
passed — correctly, because the thing it asserts is that the kept computer is the
same computer twice, and it was.

## The mechanism, which is not the usual one

This directory's other entries are about instruments: a check that cannot fail, a
fixture its writer would produce, a comparison against a mirror. **Here the
instrument was sound.** The walk is careful, its assertions are honest, its own
doc comment states precisely what it cannot hold and why. The method was right
and the subject was wrong.

That is the failure mode where **rigour makes the result more convincing and no
less wrong**. A sloppy walk would have been doubted. This one passed in 1,354
seconds, photographed its screen twice, and agreed with a week-old run — and
every one of those facts is about a machine nobody meant to be testing.

## The cure

**A cached artefact needs a freshness check, and absence of one is not
neutral** — it silently redirects every test built on it. The cheapest form is
the one the repository already uses elsewhere: record what a thing was made from
beside the thing, and compare. A kept machine could carry the image digest it was
installed from, and `is_kept` could refuse a base built from an image the tree no
longer describes.

**And the question to ask of a green result is one the usual one does not
reach.** *What would this do if the thing were absent entirely* interrogates the
check. This needs the other half: **what exactly was under the instrument, and
how do I know it was the thing I meant?** Here the answer was a file path that
had been correct a week earlier.

**Recorded because it was reported before it was checked.** The owner had the
wrong conclusion from this machine for about an hour, in the one area where
nothing else in the repository could have contradicted it — because every other
measurement of a running system comes through the same walk, from the same base.
