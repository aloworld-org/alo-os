# A library list right on the machine the tests ran on

**Concluded:** *a boxed command needs the library folder, and the library
folder is `/usr/lib`.* `alo_running::where_the_libraries_are` tried
`["/usr/lib", "/lib"]` and took the first that was a directory. The kernel test
that proves a box works had its own copy of the same list, and it passed.

**What was true.** Fedora, which alo OS ships on, keeps its loader and its C
library in `/usr/lib64`. Measured inside the image on 2026-10-31:

```text
/usr/bin/id: interpreter /lib64/ld-linux-x86-64.so.2
ld-linux resolves to /usr/lib64/ld-linux-x86-64.so.2
```

So on a booted machine no dynamically linked command could start in a `try`
box: the kernel refused it its own loader. Nobody noticed, because the same box
also refused the person's terminal, so the failure printed nothing — the
helper's `eprintln!` panicked on the refused write and ended with 101.

**The mechanism.** The tests ran on Ubuntu, which keeps both in `/usr/lib`.
The list was correct on every machine it was ever checked on, and **the check
was a copy of the list**, so the two could only ever agree. *Wrong together* is
the shape: a test that restates the value it tests measures that the value was
typed twice.

**The cure.** Ask the machine for the property, not the path. The test now
resolves every library folder the machine has — `/usr/lib`, `/usr/lib64`,
`/lib`, `/lib64` — and asserts each is under what the box grants, so it fails on
any machine with a folder the grant misses. And where a test needs the value
the product uses, it calls the product rather than keeping its own.

Found by a person on a booted machine, not by a check: nothing caught it.
