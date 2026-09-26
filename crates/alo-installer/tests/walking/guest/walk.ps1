# The walk, from inside the guest.
#
# One boot does one thing, and what that is arrives on the walk medium as
# `instruction.txt`. Everything it learns it prints; the host reads the print
# off the serial line, because a process that is still running is not evidence
# that anything happened.

param([string]$Medium = 'D:')

$ErrorActionPreference = 'Continue'
$ProgressPreference = 'SilentlyContinue'

# The serial line, opened once and written through as the walk goes. A boot
# whose print arrives only at the end is a boot nobody can tell from a hung
# one, and this walk has already been mistaken for a hung one once.
#
# Through `System.IO.Ports.SerialPort`, not a `StreamWriter` on `\\.\COM1`:
# .NET Framework's `FileStream` refuses every `\\.\` device path, so that
# writer never opened, the exception was swallowed, and the walk printed
# nothing to the serial line (measured 2026-09-21).
$script:Wire = $null
$script:WireError = ''
try {
  $script:Wire = New-Object System.IO.Ports.SerialPort 'COM1', 115200, 'None', 8, 'One'
  $script:Wire.NewLine = "`r`n"
  $script:Wire.WriteTimeout = 5000
  $script:Wire.Open()
} catch { $script:WireError = $_.Exception.Message; $script:Wire = $null }
$script:Began = Get-Date
$script:Log = 'C:\alo\walk-log.txt'
Set-Content -LiteralPath $script:Log -Value '' -Encoding ASCII

function Say([string]$line) {
  $at = '{0,7:N1}s ' -f ((Get-Date) - $script:Began).TotalSeconds
  # Never `Write-Output`: inside a function that becomes part of what the
  # function returns, and it turned the manifest's result into an array whose
  # file count was 5 (measured 2026-09-21).
  [Console]::Out.WriteLine("$at$line")
  # Written through to the disk at once, so a machine stopped hard still
  # leaves the last line said where the host can read it.
  [System.IO.File]::AppendAllText($script:Log, "$at$line`r`n")
  if ($null -ne $script:Wire) { try { $script:Wire.WriteLine("$at$line") } catch {} }
}
if ($null -eq $script:Wire) { Say "the serial line did not open: $($script:WireError)" }

$instruction = @{}
Get-Content "$Medium\alo-walk\instruction.txt" | ForEach-Object {
  if ($_ -match '^\s*([a-z-]+)\s*=\s*(.*?)\s*$') { $instruction[$Matches[1]] = $Matches[2] }
}
Say ("instruction: " + (($instruction.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ' '))

$mode = $instruction['mode']
$step = 0
if ($instruction.ContainsKey('step')) { $step = [int]$instruction['step'] }

# ---------------------------------------------------------------------------
# The manifest: the Windows partition's files, by name, size and digest
# ---------------------------------------------------------------------------
#
# Not every file on C:, because a running Windows writes its own logs and its
# own registry while anything at all is happening. This is the set a partition
# change would damage and which Windows itself leaves alone between two
# readings a minute apart: everything the computer starts from, the programs
# and libraries of the system directory, and every file on the partition the
# firmware starts Windows from.

function Manifest([string]$into) {
  $sha = [System.Security.Cryptography.SHA256]::Create()
  $lines = New-Object System.Collections.Generic.List[string]
  # Only what starts the computer, read inside the guest around the kill: the
  # whole partition is hashed from the host with the machine off, at the
  # host's speed. Measured on 2026-09-21, hashing the system directory's 623
  # programs in here took 232 s, and its libraries would have taken over
  # twenty minutes a reading.
  $roots = @(
    @{ Path = 'C:\Windows\Boot'; Kind = 'recurse' },
    @{ Path = 'S:\'; Kind = 'recurse' }
  )
  foreach ($root in $roots) {
    if (-not (Test-Path $root.Path)) { Say "manifest: no $($root.Path)"; continue }
    Say "manifest: reading $($root.Path) $($root.Kind) $($root.Filter)"
    $files = switch ($root.Kind) {
      'filter'  { Get-ChildItem -LiteralPath $root.Path -File -Filter $root.Filter -Force -ErrorAction SilentlyContinue }
      'recurse' { Get-ChildItem -LiteralPath $root.Path -File -Recurse -Force -ErrorAction SilentlyContinue }
      default   { Get-ChildItem -LiteralPath $root.Path -File -Force -ErrorAction SilentlyContinue }
    }
    foreach ($file in $files) {
      try {
        $stream = [System.IO.File]::Open($file.FullName, 'Open', 'Read', 'ReadWrite')
        $digest = [System.BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-', '').ToLower()
        $stream.Close()
        $lines.Add("$digest $($file.Length) $($file.FullName.ToLower())")
      } catch {
        # A file a running Windows holds open exclusively is named as held
        # rather than left out: a file that vanished from the list would be a
        # change the list has to show.
        $lines.Add("held-open $($file.Length) $($file.FullName.ToLower())")
      }
    }
    Say "manifest: $($lines.Count) files so far"
  }
  $sorted = @($lines | Sort-Object)
  Set-Content -LiteralPath $into -Value $sorted -Encoding ASCII
  $whole = [System.BitConverter]::ToString(
    $sha.ComputeHash([System.IO.File]::ReadAllBytes($into))).Replace('-', '').ToLower()
  return @{ Digest = $whole; Count = $sorted.Count }
}

function Mountvol([string[]]$words) {
  $ran = Start-Process -FilePath "$env:SystemRoot\System32\mountvol.exe" -ArgumentList $words `
    -PassThru -NoNewWindow
  if (-not $ran.WaitForExit(30000)) {
    Stop-Process -Id $ran.Id -Force -ErrorAction SilentlyContinue
    Say "mountvol $($words -join ' ') did not finish in 30 s"
    return
  }
  Say "mountvol $($words -join ' '): exit $($ran.ExitCode)"
}
function MountTheStartPartition() { Mountvol @('S:', '/S') }
function UnmountTheStartPartition() { Mountvol @('S:', '/D') }

# ---------------------------------------------------------------------------
# The firmware's own start-up entries, read from its variables
# ---------------------------------------------------------------------------
#
# `bcdedit` reports Windows' copy of the list; the firmware starts what its own
# `Boot####` variables hold. So every state this walk prints includes those,
# each with its device path and the length of its optional data — which is
# what a copy of Windows' boot manager entry got wrong (docs/quirks.md).
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class AloWalkFirmware {
  [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
  public static extern uint GetFirmwareEnvironmentVariableEx(string name, string guid, byte[] buffer, uint size, ref uint attributes);
  [StructLayout(LayoutKind.Sequential)] struct Luid { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] struct Privilege { public uint Count; public Luid Id; public uint Attributes; }
  [DllImport("advapi32.dll", SetLastError = true)] static extern bool OpenProcessToken(IntPtr process, uint access, out IntPtr token);
  [DllImport("advapi32.dll", SetLastError = true, CharSet = CharSet.Unicode)] static extern bool LookupPrivilegeValue(string system, string name, out Luid id);
  [DllImport("advapi32.dll", SetLastError = true)] static extern bool AdjustTokenPrivileges(IntPtr token, bool disableAll, ref Privilege state, uint length, IntPtr previous, IntPtr returned);
  [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
  // Reading a firmware variable needs the privilege switched on too: without
  // it every read returns nothing (measured 2026-09-22).
  public static int SwitchOn() {
    IntPtr token; Luid id;
    if (!OpenProcessToken(GetCurrentProcess(), 0x28, out token)) return Marshal.GetLastWin32Error();
    if (!LookupPrivilegeValue(null, "SeSystemEnvironmentPrivilege", out id)) return Marshal.GetLastWin32Error();
    Privilege state = new Privilege { Count = 1, Id = id, Attributes = 2 };
    AdjustTokenPrivileges(token, false, ref state, 0, IntPtr.Zero, IntPtr.Zero);
    return Marshal.GetLastWin32Error();
  }
}
'@
Say "firmware privilege: $([AloWalkFirmware]::SwitchOn())"
function BootOptions() {
  $found = @()
  for ($n = 0; $n -lt 0x100; $n++) {
    $name = 'Boot{0:X4}' -f $n
    $buffer = New-Object byte[] 4096
    $attributes = [uint32]0
    $size = [AloWalkFirmware]::GetFirmwareEnvironmentVariableEx($name, '{8BE4DF61-93CA-11D2-AA0D-00E098032B8C}', $buffer, 4096, [ref]$attributes)
    if ($size -eq 0) { continue }
    $length = [BitConverter]::ToUInt16($buffer, 4)
    $end = 6; while (-not ($buffer[$end] -eq 0 -and $buffer[$end + 1] -eq 0)) { $end += 2 }
    $description = [Text.Encoding]::Unicode.GetString($buffer, 6, $end - 6)
    $at = $end + 2
    $slot = $null; $start = $null; $file = ''
    if ($buffer[$at] -eq 4 -and $buffer[$at + 1] -eq 1) {
      $slot = [BitConverter]::ToUInt32($buffer, $at + 4)
      $start = [BitConverter]::ToUInt64($buffer, $at + 8)
      $next = $at + [BitConverter]::ToUInt16($buffer, $at + 2)
      if ($buffer[$next] -eq 4 -and $buffer[$next + 1] -eq 4) {
        $file = [Text.Encoding]::Unicode.GetString($buffer, $next + 4, [BitConverter]::ToUInt16($buffer, $next + 2) - 4).TrimEnd([char]0)
      }
    }
    $optional = [Math]::Max(0, [int]$size - ($at + $length))
    $found += [pscustomobject]@{ Name = $name; Description = $description; Slot = $slot; Start = $start; File = $file; Optional = $optional }
  }
  return $found
}
function TheAloOption() {
  $mine = @(BootOptions | Where-Object { $_.Description -eq 'alo OS' })
  if ($mine.Count -eq 1) { return $mine[0] } else { return $null }
}

function TheState([string]$why) {
  Say "--- state ($why) ---"
  Get-Disk | Sort-Object Number | ForEach-Object {
    Say ("disk {0}: name=[{1}] serial=[{2}] bus=[{3}] unique=[{4}] size={5} style={6}" -f `
      $_.Number, $_.FriendlyName, $_.SerialNumber, $_.BusType, $_.UniqueId, $_.Size, $_.PartitionStyle)
  }
  Get-Partition | Sort-Object DiskNumber, PartitionNumber | ForEach-Object {
    $label = ''; $fs = ''
    try { $volume = $_ | Get-Volume -ErrorAction Stop; $label = $volume.FileSystemLabel; $fs = $volume.FileSystem } catch {}
    Say ("partition {0}/{1}: offset={2} size={3} letter=[{4}] type={5} fs=[{6}] label=[{7}]" -f `
      $_.DiskNumber, $_.PartitionNumber, $_.Offset, $_.Size, $_.DriveLetter, $_.GptType, $fs, $label)
  }
  $hiberboot = (Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Power' -Name HiberbootEnabled -ErrorAction SilentlyContinue).HiberbootEnabled
  $hibernate = (Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power' -Name HibernateEnabled -ErrorAction SilentlyContinue).HibernateEnabled
  Say ("fast-startup: HiberbootEnabled=[{0}] HibernateEnabled=[{1}]" -f $hiberboot, $hibernate)
  Say "--- bcdedit /enum firmware ---"
  & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware 2>&1 | ForEach-Object { Say $_ }
  Say "--- the firmware's own entries ---"
  BootOptions | ForEach-Object { Say ("bootoption {0}: [{1}] slot={2} start={3} file=[{4}] optional-data={5} bytes" -f $_.Name, $_.Description, $_.Slot, $_.Start, $_.File, $_.Optional) }
  Say "--- end state ---"
}

# ---------------------------------------------------------------------------
# The effect each of staging.rs's seven steps leaves behind
# ---------------------------------------------------------------------------
#
# The walk kills the installer on the *effect*, not on a sentence it printed.
# What this task exists to show is that the storage cmdlets and `bcdedit` do
# what `program.rs` asks of them, so the change having really happened is the
# only honest signal that the step is done.

$script:BaseWindowsSize = 0
$script:BasePartitions = 0
$script:BaseOffsets = @()

function Baseline() {
  $windows = Get-Partition -DriveLetter C
  $script:BaseWindowsSize = $windows.Size
  $all = @(Get-Partition -DiskNumber $windows.DiskNumber)
  $script:BasePartitions = $all.Count
  $script:BaseOffsets = @($all | ForEach-Object { [uint64]$_.Offset })
  Say "baseline: windows disk=$($windows.DiskNumber) partition=$($windows.PartitionNumber) offset=$($windows.Offset) size=$($script:BaseWindowsSize) partitions=$($script:BasePartitions)"
}

# The installer's area is the partition that was not there before it ran — not
# the first one after C:. Windows 11 puts its recovery partition after C:
# (measured on this guest, 2026-09-21: 735 MiB at the end of the disk), so the
# first partition after C: is Windows' own.
function TheArea() {
  $windows = Get-Partition -DriveLetter C
  $new = @(Get-Partition -DiskNumber $windows.DiskNumber |
    Where-Object { $script:BaseOffsets -notcontains [uint64]$_.Offset })
  if ($new.Count -eq 0) { return $null }
  return $new[0]
}

# ---------------------------------------------------------------------------
# The run
# ---------------------------------------------------------------------------
$payload = 'C:\alo\payload'
if (Test-Path $payload) { Remove-Item -Recurse -Force $payload -ErrorAction SilentlyContinue }
New-Item -ItemType Directory -Path $payload -Force | Out-Null
Copy-Item -Recurse -Force "$Medium\alo-walk\payload\*" $payload
Get-ChildItem -Recurse -File $payload | ForEach-Object {
  Say ("payload-file {0,10} {1}" -f $_.Length, $_.FullName.Substring($payload.Length))
}

MountTheStartPartition

if ($mode -eq 'manifest-only') {
  $one = Manifest 'C:\alo\manifest.txt'
  Say "manifest: digest=$($one.Digest) files=$($one.Count)"
  TheState 'a Windows nothing has been done to'
  UnmountTheStartPartition
  Say 'ALOWALK-DONE manifest-only'
  return
}

# Fast Startup on, in this boot, when the instruction asks for it.
#
# The second base has hibernation on, and **this guest does not keep it across
# a restart**: `powercfg /h on` succeeds and `powercfg /a` then lists Hibernate
# and Fast Startup as available, and after the next start Windows has put
# `HibernateEnabled` back to 0 (measured 2026-09-23, `docs/quirks.md`). So the
# one walk that answers the question turns it on in the same boot it asks in,
# and says so. Everything after this line is the installer's own doing.
if ($instruction['fast-startup'] -eq 'on') {
  Say 'turning Fast Startup on for this boot, because this guest does not keep it across a restart'
  $turned = C:\Windows\System32\powercfg.exe /h on 2>&1
  if ($turned) { $turned | ForEach-Object { Say "powercfg: $_" } }
  New-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Power' -Name HibernateEnabled -Value 1 -PropertyType DWord -Force | Out-Null
  New-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Power' -Name HiberbootEnabled -Value 1 -PropertyType DWord -Force | Out-Null
}

$before = Manifest 'C:\alo\manifest-before.txt'
Say "manifest-before: digest=$($before.Digest) files=$($before.Count)"
TheState 'before the installer runs'
Baseline

$exe = Join-Path $payload 'alo-installer.exe'
if (-not (Test-Path $exe)) { Say "FAIL: no installer at $exe"; return }

$start = New-Object System.Diagnostics.ProcessStartInfo
$start.FileName = $exe
$start.WorkingDirectory = $payload
$start.UseShellExecute = $false
$start.RedirectStandardInput = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$process = [System.Diagnostics.Process]::Start($start)
Say "installer pid: $($process.Id)"
$began = Get-Date

# Read what it says until it asks, and then type what it showed — the way a
# person does, with the name the installer itself put on the screen.
$said = New-Object System.Collections.Generic.List[string]
$offered = $null
$asked = $false
while (-not $process.HasExited -or -not $process.StandardOutput.EndOfStream) {
  $line = $process.StandardOutput.ReadLine()
  if ($null -eq $line) { break }
  $said.Add($line)
  Say "installer: $line"
  if ($line -match 'The disk (.+?) \(\d+ GB\) is empty, and alo OS can be installed onto it') {
    $offered = $Matches[1]
    Say "offered: [$offered]"
  }
  if ($line -match 'type the name of the disk alo OS replaces') {
    $asked = $true
    break
  }
}

if (-not $asked) {
  Say 'the installer never asked; it said the above and stopped'
  $rest = $process.StandardOutput.ReadToEnd()
  if ($rest) { $rest -split "`r?`n" | ForEach-Object { Say "installer: $_" } }
  $err = $process.StandardError.ReadToEnd()
  if ($err) { $err -split "`r?`n" | ForEach-Object { Say "installer-stderr: $_" } }
  TheState 'after the installer stopped'
  UnmountTheStartPartition
  Say 'ALOWALK-DONE not-offered'
  return
}

if ($mode -eq 'refuse') {
  # The second control: the same installer, run the same way, making every
  # read and every side effect running it has on Windows, and refusing at the
  # consent because nothing is typed. What a kill leaves beyond this is what
  # staging changed.
  Say 'typing: [] (the refusal control types nothing)'
  $process.StandardInput.WriteLine('')
  $process.StandardInput.WriteLine('')
  $process.StandardInput.Close()
  $rest = $process.StandardOutput.ReadToEnd()
  if ($rest) { $rest -split "`r?`n" | ForEach-Object { if ($_) { Say "installer: $_" } } }
  Say "installer exited: $($process.WaitForExit(120000)); code=$($process.ExitCode)"
  TheState 'after the refusal'
  $after = Manifest 'C:\alo\manifest-after.txt'
  Say "manifest-after: digest=$($after.Digest) files=$($after.Count)"
  if ($before.Digest -eq $after.Digest) { Say 'THE WINDOWS FILES ARE UNCHANGED' } else { Say 'THE WINDOWS FILES CHANGED' }
  UnmountTheStartPartition
  Say 'ALOWALK-DONE refuse'
  return
}

if ($mode -eq 'kill-at-consent') {
  # The third control: the same installer, run the same way, left at the
  # consent for as long as a kill run spends after it, and then killed —
  # everything a kill run does to Windows except staging. What a kill leaves
  # beyond this is staging's.
  Say 'typing: [] (the kill control is killed at the consent)'
  Start-Sleep -Seconds 60
  $process.Kill()
  Say 'killed at the consent'
  Start-Sleep -Seconds 2
  TheState 'after the kill at the consent'
  $after = Manifest 'C:\alo\manifest-after.txt'
  Say "manifest-after: digest=$($after.Digest) files=$($after.Count)"
  if ($before.Digest -eq $after.Digest) { Say 'THE WINDOWS FILES ARE UNCHANGED' } else { Say 'THE WINDOWS FILES CHANGED' }
  UnmountTheStartPartition
  Say 'ALOWALK-DONE kill-at-consent'
  return
}

if ($null -eq $offered) { Say 'FAIL: it asked, and never named a disk to type' }
Say "typing: [$offered]"
$process.StandardInput.WriteLine($offered)
$process.StandardInput.Flush()

# The one question after the consent: Fast Startup, asked only on a computer
# whose Fast Startup is on (ADR 0064 term 9). The answer arrives in the
# instruction, and lines are read until the question is asked or until staging
# has plainly begun, so a boot with nothing to answer waits for nothing.
#
# **Every walk answers it**, because this Windows' own value reads as on even
# with hibernation off (`docs/quirks.md`): the settled base keeps
# `HiberbootEnabled` at 1 after `powercfg /h off`. A walk that did not answer
# would leave the installer waiting at the question for ever. The answer is
# *leave on* unless the instruction names another, so nothing about Windows is
# changed by walks that are not about this question.
$answer = 'leave on'
if ($instruction.ContainsKey('answer') -and $instruction['answer'] -ne '') {
  $answer = $instruction['answer']
}
$answered = $false
# At most three lines: when the question is asked it is the first thing said
# after the consent, so a computer that is not asked is not waited on.
for ($read = 0; $read -lt 3 -and -not $answered -and -not $process.HasExited; $read++) {
  $line = $process.StandardOutput.ReadLine()
  if ($null -eq $line) { break }
  $said.Add($line)
  Say "installer: $line"
  if ($line -match 'and press Enter') {
    Say "typing: [$answer]"
    $process.StandardInput.WriteLine($answer)
    $process.StandardInput.Flush()
    $answered = $true
  }
}
if (-not $answered) { Say 'the installer did not ask about Fast Startup' }

if ($mode -eq 'whole-road') {
  $rest = $process.StandardOutput.ReadToEnd()
  if ($rest) { $rest -split "`r?`n" | ForEach-Object { Say "installer: $_" } }
  Say "installer exited: $($process.WaitForExit(600000)); code=$($process.ExitCode)"
  TheState 'after the whole road'
  $after = Manifest 'C:\alo\manifest-after.txt'
  Say "manifest-after: digest=$($after.Digest) files=$($after.Count)"
  UnmountTheStartPartition
  Say 'ALOWALK-DONE whole-road'
  return
}

# ---------------------------------------------------------------------------
# The kill, landed on the boundary and then checked
# ---------------------------------------------------------------------------
#
# Where each boundary is caught:
#   1, 2, 3  the step's own PowerShell child (the 1st, 2nd and 3rd after the
#            consent): the installer is frozen (NtSuspendProcess) while it waits
#            on that child, the child finishes the step, and it is killed.
#   4 to 7   **the first program of the next step is held before it runs.**
#            Windows' Image File Execution Options start a stand-in in that
#            program's place, so the installer is caught at the very moment it
#            asks Windows to start it, and killed there:
#              4  powershell.exe, once the 3rd child is seen: the 4th writes the entry
#              5  powershell.exe, once the 4th child is seen: the 5th takes the letter
#              6  bcdedit.exe,    once the 5th child is seen: bcdedit sets the next start
#              7  shutdown.exe,   from the start: the restart
#            Measured on 2026-09-21 that polling cannot do this for step 4: the
#            installer gets from its last written file into the next program
#            faster than any poll here can see and act, and four tries landed
#            one or two steps late. Nothing in the installer knows about the
#            hold, and it is let go before anything is read.

Add-Type -Namespace AloWalk -Name Freeze -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("ntdll.dll")]
public static extern int NtSuspendProcess(System.IntPtr process);
'@

function Children() {
  @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$($process.Id)" -ErrorAction SilentlyContinue)
}

$ifeo = 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options'
$held = 'C:\alo\held.txt'
Remove-Item -LiteralPath $held -Force -ErrorAction SilentlyContinue
# The stand-in: writes down the command line it was started with, and never
# returns. Not `cmd.exe` running a script: measured on 2026-09-22, a held
# PowerShell's encoded command is longer than cmd's 8 191-character line, cmd
# failed at once, and the installer saw a failed step and began putting back
# before it could be killed.
$standIn = 'C:\alo\held.exe'
if (-not (Test-Path -LiteralPath $standIn)) {
  Add-Type -OutputType ConsoleApplication -OutputAssembly $standIn -TypeDefinition @'
public static class Held {
  public static void Main() {
    System.IO.File.WriteAllText(@"C:\alo\held.txt", System.Environment.CommandLine);
    System.Threading.Thread.Sleep(System.Threading.Timeout.Infinite);
  }
}
'@
}
$script:Holding = $null
$script:HoldingMadeTheKey = $false
function Hold([string]$image) {
  $key = Join-Path $ifeo $image
  $script:HoldingMadeTheKey = -not (Test-Path -LiteralPath $key)
  if ($script:HoldingMadeTheKey) { New-Item -Path $key -Force | Out-Null }
  New-ItemProperty -LiteralPath $key -Name Debugger -Value $standIn -PropertyType String -Force | Out-Null
  $script:Holding = $key
  Say "holding the next start of $image"
}
function LetGo() {
  if (-not $script:Holding) { return }
  if ($script:HoldingMadeTheKey) { Remove-Item -LiteralPath $script:Holding -Recurse -Force -ErrorAction SilentlyContinue }
  else { Remove-ItemProperty -LiteralPath $script:Holding -Name Debugger -ErrorAction SilentlyContinue }
  Say "let go of $(Split-Path -Leaf $script:Holding)"
  $script:Holding = $null
}
function EndTheStandIn() {
  Get-Process -Name 'held' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

$killed = $false
$deadline = (Get-Date).AddSeconds(420)
$seenShells = New-Object System.Collections.Generic.List[int]
if ($step -eq 7) { Hold 'shutdown.exe' }
try {
  while ((Get-Date) -lt $deadline -and -not $killed) {
    if ($script:Holding -and (Test-Path -LiteralPath $held)) {
      $process.Kill()
      $killed = $true
      Say "held before it ran: $((Get-Content -LiteralPath $held -Raw).Trim())"
      break
    }
    if ($process.HasExited) { Say "the installer exited before step $step was reached"; break }
    foreach ($child in Children) {
      if ($child.Name -ne 'powershell.exe' -or $seenShells.Contains([int]$child.ProcessId)) { continue }
      $seenShells.Add([int]$child.ProcessId)
      Say ("powershell child {0} started (pid {1})" -f $seenShells.Count, $child.ProcessId)
      if ($step -le 3 -and $seenShells.Count -eq $step) {
        $frozen = [AloWalk.Freeze]::NtSuspendProcess($process.Handle)
        Say ("installer frozen: status {0:X8}" -f $frozen)
        Wait-Process -Id $child.ProcessId -Timeout 120 -ErrorAction SilentlyContinue
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $killed = $true
        break
      }
      if ($step -eq 4 -and $seenShells.Count -eq 3) { Hold 'powershell.exe' }
      if ($step -eq 5 -and $seenShells.Count -eq 4) { Hold 'powershell.exe' }
      if ($step -eq 6 -and $seenShells.Count -eq 5) { Hold 'bcdedit.exe' }
    }
    Start-Sleep -Milliseconds 20
  }
} finally {
  LetGo
  EndTheStandIn
}
if ($killed) {
  Say ("killed at step {0}, {1:N2} s after it started" -f $step, ((Get-Date) - $began).TotalSeconds)
} else {
  Say "NOT KILLED: step $step was never reached inside the deadline"
}

Start-Sleep -Seconds 2
Get-Process -Name 'alo-installer' -ErrorAction SilentlyContinue | ForEach-Object {
  Say "an installer was still running after the kill: $($_.Id)"; Stop-Process -Id $_.Id -Force
}

try { $rest = $process.StandardOutput.ReadToEnd() } catch { $rest = '' }
if ($rest) { $rest -split "`r?`n" | ForEach-Object { if ($_) { Say "installer: $_" } } }
try { $err = $process.StandardError.ReadToEnd() } catch { $err = '' }
if ($err) { $err -split "`r?`n" | ForEach-Object { if ($_) { Say "installer-stderr: $_" } } }

TheState "after the kill at step $step"

# Where the kill landed, read from Windows' own tools and the firmware's own
# variables — each step's effect present, and the next step's absent.
$area = TheArea
$fw = & "$env:SystemRoot\System32\bcdedit.exe" /enum '{fwbootmgr}' 2>&1 | Out-String
$all = & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware 2>&1 | Out-String
$entry = $null
foreach ($block in ($all -split '(?:\r?\n){2,}')) {
  if ($block -match '(?m)^description\s+alo OS\s*$' -and $block -match '(?m)^identifier\s+(\{[0-9a-fA-F-]{36}\})') { $entry = $Matches[1] }
}
$option = TheAloOption
$volume = if ($null -ne $area) { Get-Volume -Partition $area -ErrorAction SilentlyContinue } else { $null }
$everythingSaid = ($said -join "`n") + "`n" + $rest
$facts = [ordered]@{
  # A kill that lands on a boundary finds the installer mid-road, never
  # already undoing: an installer that had begun putting back was reacting to
  # a step that failed, which is not the kill this walk means.
  'installer had begun putting back' = ($everythingSaid -match 'is being put back')
  'windows smaller'   = ((Get-Partition -DriveLetter C).Size -lt $script:BaseWindowsSize)
  'area made'         = ($null -ne $area)
  'area formatted'    = ($null -ne $volume -and $volume.FileSystem -like 'FAT*' -and $volume.FileSystemLabel -eq 'ALO-INSTALL')
  'area has a letter' = ($null -ne $area -and [bool]$area.DriveLetter)
  'choice written'    = ($null -ne $area -and [bool]$area.DriveLetter -and (Test-Path ("{0}:\EFI\BOOT\chosen.cfg" -f $area.DriveLetter)))
  'any alo OS entry'  = ($null -ne $entry -or $null -ne $option)
  'entry listed'      = ($null -ne $entry -and $fw -match [regex]::Escape($entry))
  'entry points at the area with no optional data' = ($null -ne $option -and $null -ne $area -and $option.Start -eq [uint64]($area.Offset / 512) -and $option.File -eq '\EFI\BOOT\BOOTX64.EFI' -and $option.Optional -eq 0)
  'next start set'    = ($fw -match 'bootsequence')
}
$facts.GetEnumerator() | ForEach-Object { Say ("fact: {0} = {1}" -f $_.Key, $_.Value) }
$f = $facts
$landed = switch ($step) {
  1 { $f['windows smaller'] -and -not $f['area made'] }
  2 { $f['area made'] -and -not $f['area formatted'] }
  3 { $f['area formatted'] -and $f['area has a letter'] -and -not $f['choice written'] }
  4 { $f['choice written'] -and -not $f['any alo OS entry'] }
  5 { $f['entry listed'] -and $f['entry points at the area with no optional data'] -and $f['area has a letter'] -and -not $f['next start set'] }
  6 { $f['entry listed'] -and $f['entry points at the area with no optional data'] -and -not $f['area has a letter'] -and -not $f['next start set'] }
  7 { $f['next start set'] -and $f['entry points at the area with no optional data'] }
}
if ($f['installer had begun putting back']) { $landed = $false }
if ($landed) { Say "LANDED EXACTLY after step $step" } else { Say "DID NOT LAND after step $step exactly: see the facts above" }

# What the installer wrote into the area, while the area still has a letter to
# read it through. This is the name the environment is told to look for under
# /dev/disk/by-id/ after the restart, and half of what `naming.rs` claims.
$area = TheArea
if ($null -ne $area -and $area.DriveLetter) {
  $root = "{0}:\" -f $area.DriveLetter
  Get-ChildItem -Recurse -File $root -ErrorAction SilentlyContinue | ForEach-Object {
    Say ("area-file {0,10} {1}" -f $_.Length, $_.FullName.Substring($root.Length))
  }
  $chosen = Join-Path $root 'EFI\BOOT\chosen.cfg'
  if (Test-Path $chosen) {
    Get-Content $chosen | ForEach-Object { Say "chosen.cfg: $_" }
    $line = (Get-Content $chosen | Select-Object -First 1)
    if ($line -match 'set alo_installing_to=(.+)$') { Say "the-name-written: $($Matches[1])" }
  }
}

$after = Manifest 'C:\alo\manifest-after.txt'
Say "manifest-after: digest=$($after.Digest) files=$($after.Count)"
if ($before.Digest -eq $after.Digest) {
  Say 'THE WINDOWS FILES ARE UNCHANGED'
} else {
  Say 'THE WINDOWS FILES CHANGED, and here is every line that differs:'
  Compare-Object (Get-Content 'C:\alo\manifest-before.txt') (Get-Content 'C:\alo\manifest-after.txt') |
    ForEach-Object { Say ("{0} {1}" -f $_.SideIndicator, $_.InputObject) }
}
UnmountTheStartPartition

# A probe, only when the instruction names one: exploration that is never part
# of the walk itself. The probe's file travels on the walk disc beside this one.
if ($instruction['probe']) {
  $probeFile = "$Medium\alo-walk\probe-$($instruction['probe']).ps1"
  if (Test-Path $probeFile) { . $probeFile } else { Say "probe: there is no $probeFile" }
}
# Removing alo OS again, started as a person starts it: the same copy, with the
# removal's own word, and the disk's name typed back from the sentence the
# removal itself printed — so the name comes from the program under test rather
# than from anything written here. The state is read before and after, and the
# removal does not restart the computer, so the lines after it are this boot's.
if ($mode -eq 'remove') {
  $left = Join-Path $instruction['left-at'] $instruction['left-as']
  Say "the road back: $left"
  TheState('before the removal')
  if (-not (Test-Path -LiteralPath $left)) {
    Say 'FAIL: the installer left nothing to remove alo OS with'
  } else {
    Say ("the shortcut: {0} exists={1}" -f $instruction['shortcut'],
      (Test-Path -LiteralPath $instruction['shortcut']))
    $removal = New-Object System.Diagnostics.ProcessStartInfo
    $removal.FileName = $left
    $removal.Arguments = $instruction['argument']
    $removal.UseShellExecute = $false
    $removal.RedirectStandardInput = $true
    $removal.RedirectStandardOutput = $true
    $removal.RedirectStandardError = $true
    $running = [System.Diagnostics.Process]::Start($removal)
    Say "the removal's pid: $($running.Id)"
    $named = $null
    $typed = $false
    while (-not $running.HasExited -or -not $running.StandardOutput.EndOfStream) {
      $line = $running.StandardOutput.ReadLine()
      if ($null -eq $line) { break }
      Say "removal: $line"
      if ($line -match 'alo OS is on the disk (.+?)\. Removing it erases') {
        $named = $Matches[1]
        Say "the disk it named: [$named]"
      }
      if (-not $typed -and $line -match 'and press Enter') {
        if ($null -eq $named) {
          Say 'FAIL: it asked for a disk before it named one'
          $running.StandardInput.WriteLine('')
        } else {
          Say "typing: [$named]"
          $running.StandardInput.WriteLine($named)
        }
        $running.StandardInput.Flush()
        $typed = $true
      } elseif ($line -match 'Press Enter to close') {
        $running.StandardInput.WriteLine('')
        $running.StandardInput.Flush()
      }
    }
    Say "the removal exited: $($running.WaitForExit(600000)); code=$($running.ExitCode)"
  }
  TheState('after the removal')
  Say 'ALOWALK-DONE remove'
  return
}

# The way back in, started as a person starts it: the copy the installer left
# in Windows' own place for programs, with the switch's word as its argument,
# and the word typed at its question. It sets the firmware's next start and
# restarts this computer, so nothing after this line runs.
if ($mode -eq 'switch') {
  $left = Join-Path $instruction['left-at'] $instruction['left-as']
  Say "the way back: $left"
  if (-not (Test-Path -LiteralPath $left)) {
    Say 'FAIL: the installer left no way back'
  } else {
    $shortcut = $instruction['shortcut']
    Say ("the shortcut: {0} exists={1}" -f $shortcut, (Test-Path -LiteralPath $shortcut))
    $switch = New-Object System.Diagnostics.ProcessStartInfo
    $switch.FileName = $left
    $switch.Arguments = $instruction['argument']
    $switch.UseShellExecute = $false
    $switch.RedirectStandardInput = $true
    $switch.RedirectStandardOutput = $true
    $switch.RedirectStandardError = $true
    $running = [System.Diagnostics.Process]::Start($switch)
    Say "the way back's pid: $($running.Id)"
    $typed = $false
    while (-not $running.HasExited -or -not $running.StandardOutput.EndOfStream) {
      $line = $running.StandardOutput.ReadLine()
      if ($null -eq $line) { break }
      Say "switch: $line"
      if (-not $typed -and $line -match 'and press Enter') {
        Say "typing: [$($instruction['agree'])]"
        $running.StandardInput.WriteLine($instruction['agree'])
        $running.StandardInput.Flush()
        $typed = $true
      }
    }
    Say "the way back exited: $($running.WaitForExit(120000)); code=$($running.ExitCode)"
  }
  Say 'ALOWALK-DONE switch'
  return
}

# What this boot turned on, it turns off again: the base has hibernation off,
# and a machine left with it on is not the machine the next boot expects.
if ($instruction['fast-startup'] -eq 'on') {
  $back = C:\Windows\System32\powercfg.exe /h off 2>&1
  if ($back) { $back | ForEach-Object { Say "powercfg: $_" } }
  Say 'hibernation is off again, as the base has it'
}

Say "ALOWALK-DONE kill-at-step $step"
