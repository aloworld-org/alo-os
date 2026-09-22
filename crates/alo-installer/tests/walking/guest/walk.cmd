@echo off
rem Handed the medium's drive letter by C:\alo\at-every-start.cmd, whose own
rem output already goes to C:\alo\walk.txt. Nothing here may write to that file:
rem measured on 2026-09-21, a second redirection to it is refused with *being
rem used by another process* and the script behind it never runs.
rem
rem The walk script holds the serial line itself, so nothing here redirects to
rem COM1 around it: a serial port is opened by one holder at a time.
echo ALOWALK-CMD-START %1 %TIME% > COM1
copy /y "%1\alo-walk\walk.ps1" C:\alo\walk.ps1 > nul
powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "C:\alo\walk.ps1" -Medium "%1" > C:\alo\walk-ps.txt 2>&1
echo ALOWALK-CMD-END %ERRORLEVEL% %TIME% > COM1
type C:\alo\walk-ps.txt > COM1
rem Windows shuts itself down when the boot has done what it was told, the way
rem a person's restart does. Asking from outside with the power button was
rem ignored for three minutes after a kill on 2026-09-21, and the machine had
rem to be cut off, which left NTFS as no reader should trust.
echo ALOWALK-SHUTTING-DOWN %TIME% > COM1
shutdown /s /t 5 /f
