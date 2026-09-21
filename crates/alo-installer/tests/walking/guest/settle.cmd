@echo off
rem Settle the installed Windows once, before it becomes the base every walk
rem is an overlay of.
rem
rem Fast startup off: with it on, a shutdown hibernates the kernel session and
rem leaves NTFS as a Linux reader should not trust, and every reading of the
rem disk is taken from the host with the machine off.
echo ALOWALK-SETTLE-BEGIN %TIME% > COM1
powercfg /h off > C:\alo\settle.txt 2>&1
reg query "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Power" /v HiberbootEnabled >> C:\alo\settle.txt 2>&1
type C:\alo\settle.txt > COM1
rem Whatever Windows does in the minutes after its first start (compiling .NET
rem assemblies among it: the first PowerShell here took four minutes to start)
rem is given ten minutes to do, now, once, rather than in every walk.
powershell.exe -NoProfile -NonInteractive -Command "Write-Output ('ALOWALK-SETTLE-PS ' + $PSVersionTable.PSVersion)" > C:\alo\settle-ps.txt 2>&1
type C:\alo\settle-ps.txt > COM1
echo ALOWALK-SETTLE-WAITING %TIME% > COM1
timeout /t 600 /nobreak > nul
echo ALOWALK-SETTLE-SHUTDOWN %TIME% > COM1
shutdown /s /t 5 /f
