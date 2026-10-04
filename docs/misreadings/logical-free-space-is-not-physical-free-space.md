# Logical free space is not physical free space

**Concluded:** *there is 826 GB free, so a nine-gigabyte image build is
comfortable.*

**True:** `df -h /` inside a WSL guest reports room inside a vhdx whose maximum
is 1007 GB. The vhdx is a **file on the host**, and the host had **21 GB**. The
build needed about 29, filled `C:` to 1.9 MB, and the guest would not restart:
`Wsl/Service/CreateInstance/E_FAIL`.

**The mechanism.** Both numbers are true about different things, and the
script that printed 826 printed 21 four lines later as the lesser of two facts.
The larger number was reported because it answered the question as asked.

**The same trap, once more, an hour later:** a vhdx's `Length` is its **logical**
size. `GetCompressedFileSizeW` gives what it occupies — 176.3 GB against 108.6
GB allocated.

**The cure.** When a guest reports space, ask what backs it and report the
smaller number. When a sparse file reports a size, ask for the allocated size.
**The governing number is the one that runs out first.**
