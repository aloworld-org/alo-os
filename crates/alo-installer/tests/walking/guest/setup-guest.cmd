@echo off
rem Runs once, elevated, from the answer file's FirstLogonCommands.
rem It puts the walk's every-start script on C:, registers it as an elevated
rem logon task, and opens the serial line the walk reads its evidence from.

mkdir C:\alo 2>nul
copy /y "%~dp0at-every-start.cmd" C:\alo\at-every-start.cmd

mode COM1: BAUD=115200 PARITY=n DATA=8 STOP=1 xon=off odsr=off octs=off dtr=on rts=on idsr=off

rem An entry under Run would start unelevated; the installer asks for an
rem administrator's rights, so the walk needs the full token a task gives it.
schtasks /create /tn alo-walk /tr "cmd /c C:\alo\at-every-start.cmd" /sc onlogon /ru alo /rp AloLaneB!2026 /rl highest /f

rem There is one channel and deliberately no second one. An earlier version of
rem this file asked Windows Update for the OpenSSH server as a convenience;
rem measured on 2026-09-21 it sat at *Operation Running* for more than ten
rem minutes inside a synchronous first-sign-in command, and the walk needs
rem nothing it offers. The serial line is the channel.

echo INSTALL_DONE > C:\alo\installed.txt

rem The task above triggers at a sign-in, and the sign-in that runs this one has
rem already happened — so without this line the first boot says nothing on the
rem serial line and the walk waits for a Windows that is already sitting at its
rem desktop.
call C:\alo\at-every-start.cmd
