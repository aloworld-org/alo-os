# A mention is not a call

**Concluded:** *`dispatch_window_shortcut` has four production callers.*

**True:** it has **one**, at `crates/alo-shell/src/window_command.rs:66`. The
other three hits were the function's own definition at
`shortcut_dispatch.rs:43` and two doc comments naming it in prose.

**The mechanism.** `grep -rn 'name' --include=*.rs` answers *where does this
string appear*, and the question asked was *what calls this*. A definition and a
doc comment both contain the name, so the count is always at least one too high
and usually more. The number looked plausible because four callers is an
ordinary number for a dispatcher.

**The cure.** Count call sites by their shape — `name(` with an argument list —
and read the lines rather than the total. Then answer *is this production* with
the **module gate**, never the directory: this workspace keeps tests in
`src/*_tests.rs`, in `#[cfg(test)] mod` blocks, and in plain `src/*.rs` files
gated at their `mod` line. `window_command.rs`'s own `cfg(test)` begins at line
112, which is what made its line 66 a real caller.
