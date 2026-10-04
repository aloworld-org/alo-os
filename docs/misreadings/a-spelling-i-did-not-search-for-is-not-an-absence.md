# A spelling I did not search for is not an absence

**Concluded:** *`enqueue.py` has zero `sys.exit` calls, so it cannot report a
refusal* — and it was then "fixed".

**True:** it uses `raise SystemExit`, and line 42 of the original already exited
1 on a refusal. The file was never broken. The fix added two unreachable
duplicate lines, and the proof offered for it — *exit 1 on a merged pull
request* — **would have passed before the change too**.

**The mechanism.** One spelling was searched and its absence read as the
absence of the behaviour. Python has two spellings for this and the two files
audited used one each: `enqueue.py` 0 `sys.exit` and 4 `raise SystemExit`,
`dequeue.py` 2 and 0.

**The cure.** Search for the behaviour, not one of its spellings, and when a
grep returns zero ask what else the thing could be called. Then: **a test that
would have passed before the change tests nothing** — run it against the
unchanged file first.
