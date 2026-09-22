@echo off
rem Runs elevated at every logon. Says on the serial line that this Windows
rem reached an interactive desktop session, with the evidence rather than the
rem claim, and then runs whatever the walk put on the walk medium.

mode COM1: BAUD=115200 PARITY=n DATA=8 STOP=1 xon=off odsr=off octs=off dtr=on rts=on idsr=off >nul 2>&1

echo ALOWALK-BEGIN > COM1
echo boot-time: %DATE% %TIME% > COM1
echo user: %USERNAME% on %COMPUTERNAME% > COM1

rem Evidence of a desktop session: the session table, and the shell's own
rem process. A logon script running is not by itself a desktop.
qwinsta > C:\alo\session.txt 2>&1
type C:\alo\session.txt > COM1
tasklist /fi "IMAGENAME eq explorer.exe" > C:\alo\shell.txt 2>&1
type C:\alo\shell.txt > COM1
whoami /groups /fo csv /nh > C:\alo\groups.txt 2>&1
findstr /c:"S-1-5-32-544" C:\alo\groups.txt > COM1

set WALK=
for %%d in (D E F G H I J K L M N O P Q R S T U V W X Y Z) do if exist %%d:\alo-walk\walk.cmd set WALK=%%d:
if "%WALK%"=="" (
  echo ALOWALK-NO-INSTRUCTION > COM1
  echo ALOWALK-END > COM1
  goto :eof
)

echo ALOWALK-MEDIUM %WALK% > COM1
call %WALK%\alo-walk\walk.cmd %WALK% > C:\alo\walk.txt 2>&1
type C:\alo\walk.txt > COM1
echo ALOWALK-END > COM1
