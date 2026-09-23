@echo off
rem Make the variant of the base whose Fast Startup is ON.
rem
rem The base every walk is an overlay of has Fast Startup off, because a
rem shutdown with it on hibernates the Windows volume and a reader on the host
rem must not trust what it then finds. So the question the installer asks about
rem Fast Startup (ADR 0064 term 9) cannot be reached from that base at all.
rem This makes a second base from it, with hibernation on and Fast Startup on,
rem for the one walk that answers the question. Nothing reads this machine's
rem disk from the host.
echo ALOWALK-VARIANT-BEGIN %TIME% > COM1
powercfg /h on > C:\alo\variant.txt 2>&1
reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power" /v HiberbootEnabled /t REG_DWORD /d 1 /f >> C:\alo\variant.txt 2>&1
reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power" /v HiberbootEnabled >> C:\alo\variant.txt 2>&1
type C:\alo\variant.txt > COM1
echo ALOWALK-VARIANT-DONE %TIME% > COM1
rem Restarted rather than shut down, and the host stops the machine once the
rem firmware starts again: a restart closes the Windows volume whatever Fast
rem Startup says, while a *shutdown* with Fast Startup on hibernates the
rem session — and a machine that resumes a session never signs in again, so the
rem walk's own task would never run on an overlay of it.
shutdown /r /t 5 /f
