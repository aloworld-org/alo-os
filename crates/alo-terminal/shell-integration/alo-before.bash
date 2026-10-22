# alo OS: keep a place to return to, before each command you run.
#
# What this does, in plain words: before each command you type, alo takes a
# copy of your own files, so you can put them back afterwards. It costs a few
# milliseconds and you will not notice it.
#
# What it reveals: nothing. Your commands are not sent anywhere — alo is told
# only that one is about to run.
#
# If alo cannot keep a place, it says so once and your command runs anyway.
# alo never stops you working to protect a feature.
#
# You added this. alo never puts it in your shell startup on its own — either
# you did it by hand, or from Settings under Terminal, which asks you first.
# To stop, delete these lines.

# The guard is `alo.bash`'s, for `alo.bash`'s reason and one more. That file
# defines `try` only if `alo-try` is on the path, so an image without it has no
# `try` rather than a `try` that reports a command not found from inside itself.
# `PS0` is worse than that if it is wrong: it is expanded **before every command
# you ever type**, so an absent `alo-before` would put a command-not-found line
# in front of each one of them, for ever. Measured on 2026-10-22: no image built
# before that date carried `alo-before` at all.
if command -v alo-before >/dev/null 2>&1 && [[ "$PS0" != *alo-before* ]]; then
	PS0='$(alo-before)'"$PS0"
fi
