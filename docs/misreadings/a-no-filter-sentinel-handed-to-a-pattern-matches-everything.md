# A "no filter" value passed to a pattern matcher matches everything

**What was concluded.** On 2026-11-12 a measuring script for the window-buttons
lane cleaned up after each case with:

```sh
pkill -f "$filter"
```

`$filter` named the application under test (`irefox`, `ext`). Cases that had
nothing to look for passed `-`, meaning *no filter*. The script did check for
that, but only on the line that walks the accessibility tree:
`[ "$filter" != - ] && walk …`. The cleanup had no such check. The script read
as if it killed only the application it had started.

**What was true.** Every case marked `-` ran `pkill -f -`. With `-f`, pkill
matches the pattern against the **whole command line** of every process, and
one hyphen appears in almost every command line: a flag, a path, a
`systemd-journald`. At about 08:45 it killed nearly every root process in WSL
Ubuntu. That took out other lanes' builds and gates under the cargo lock, a
Flathub install, and systemd's own services. The distro dropped to emergency
mode, and WSL's DNS proxy stopped answering until somebody else replaced
`resolv.conf` by hand. Nothing failed that pointed at the script: its own cases
reported only empty output.

**The mechanism.** **A sentinel means something only to the code that agreed
on it.** `-` meant *nothing* to the `[ … != - ]` test that knew about it. To a
regular expression it is one ordinary character, and an ordinary character
matches everywhere. The guard and the matcher were three lines apart and read
as one rule. They were two, and only one of them knew the sentinel. **Pattern
matchers fail open**: an empty or too-short pattern selects more, never less.
So the mistake shows up as damage somewhere else, not as an error in the script
that made it.

**The cure.** **Never select processes to kill by a pattern.** Kill what you
started, by what you got when you started it:

```sh
setsid "$@" > log 2>&1 &   # its own process group
p=$!
…
kill -- -"$p"              # that group, and nothing else
```

Where a pattern is truly needed (`grep`, `find`, a test filter), give a
missing filter its own branch. Never pass the sentinel into the matcher. Ask
of every pattern: *what does this select when the value is empty, or a single
character?* For anything that kills or deletes, the answer must be *nothing*.
