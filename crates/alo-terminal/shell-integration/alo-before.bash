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

if [[ "$PS0" != *alo-before* ]]; then
	PS0='$(alo-before)'"$PS0"
fi
