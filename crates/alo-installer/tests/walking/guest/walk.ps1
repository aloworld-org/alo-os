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
  Say "--- bcdedit /enum firmware ---"
  & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware 2>&1 | ForEach-Object { Say $_ }
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

function StepIsDone([int]$which) {
  try {
    switch ($which) {
      1 { return (Get-Partition -DriveLetter C).Size -lt $script:BaseWindowsSize }
      2 {
        $windows = Get-Partition -DriveLetter C
        return @(Get-Partition -DiskNumber $windows.DiskNumber).Count -gt $script:BasePartitions
      }
      3 {
        $area = TheArea
        if ($null -eq $area -or -not $area.DriveLetter) { return $false }
        $volume = Get-Volume -Partition $area -ErrorAction SilentlyContinue
        return ($null -ne $volume -and $volume.FileSystem -like 'FAT*')
      }
      4 {
        $area = TheArea
        if ($null -eq $area -or -not $area.DriveLetter) { return $false }
        # chosen.cfg is the last file the copy writes and reads back.
        return (Test-Path ("{0}:\EFI\BOOT\chosen.cfg" -f $area.DriveLetter))
      }
      5 {
        $enum = & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware 2>&1 | Out-String
        if ($enum -notmatch 'alo OS') { return $false }
        return ($enum -match '(?s)alo OS.*?\\EFI\\BOOT\\BOOTX64\.EFI') -or
               ($enum -match '(?s)\\EFI\\BOOT\\BOOTX64\.EFI.*?alo OS')
      }
      6 {
        $area = TheArea
        if ($null -eq $area) { return $false }
        return -not $area.DriveLetter
      }
      7 {
        $enum = & "$env:SystemRoot\System32\bcdedit.exe" /enum '{fwbootmgr}' 2>&1 | Out-String
        return ($enum -match 'bootsequence')
      }
    }
  } catch { return $false }
  return $false
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

if ($null -eq $offered) { Say 'FAIL: it asked, and never named a disk to type' }
Say "typing: [$offered]"
$process.StandardInput.WriteLine($offered)
$process.StandardInput.Flush()

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
# Measured on 2026-09-21: polling a step's effect with Get-Partition takes about
# 300 ms a look, and the installer goes from step 4 to step 6 faster than that —
# four bcdedit calls and one short PowerShell — so a kill *aimed* at step 4
# landed after step 6. So the installer is **frozen** (NtSuspendProcess) the
# moment the boundary is reached, its children are killed, and then it is; and
# afterwards the state is read and the landing is said exactly, never assumed.
#
# Where each boundary is caught:
#   1, 2, 3, 6  the step's own PowerShell child (the 1st, 2nd, 3rd and 4th after
#               the consent): the installer is frozen while it waits on that
#               child, the child finishes, and nothing after it can start.
#   4           chosen.cfg — the last file the copy writes — through the letter
#               step 3 gave the area, looked at every few milliseconds.
#   5           the 4th PowerShell child appearing: all four bcdedit calls of
#               step 5 are behind it, and it is killed before it can start.
#   7           the next start set, read from bcdedit: the installer pauses for
#               the person before it restarts, so there is time.

Add-Type -Namespace AloWalk -Name Freeze -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("ntdll.dll")]
public static extern int NtSuspendProcess(System.IntPtr process);
'@

function Children() {
  @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$($process.Id)" -ErrorAction SilentlyContinue)
}
function FreezeAndKill([bool]$killChildrenToo) {
  # Terminated first and at once, and its children after: measured on
  # 2026-09-21, freezing it and then listing its children took long enough for
  # all four bcdedit calls of step 5 to finish, so the freeze had not held it.
  $process.Kill()
  if ($killChildrenToo) { Children | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
}

$killed = $false
$deadline = (Get-Date).AddSeconds(420)
$seenShells = New-Object System.Collections.Generic.List[int]
$areaLetter = $null
while ((Get-Date) -lt $deadline -and -not $killed) {
  if ($process.HasExited) { Say "the installer exited before step $step was reached"; break }
  if ($step -eq 4 -and $null -ne $areaLetter) {
    # Nothing but the file, as fast as it can be looked at: measured on
    # 2026-09-21, a 200 ms process listing in this loop was long enough for
    # the installer to get from chosen.cfg into `bcdedit /copy`.
    $chosen = "{0}:\EFI\BOOT\chosen.cfg" -f $areaLetter
    while (-not [System.IO.File]::Exists($chosen) -and -not $process.HasExited -and (Get-Date) -lt $deadline) { }
    if ([System.IO.File]::Exists($chosen)) { FreezeAndKill $true; $killed = $true }
    break
  }
  foreach ($child in Children) {
    if ($child.Name -eq 'powershell.exe' -and -not $seenShells.Contains([int]$child.ProcessId)) {
      $seenShells.Add([int]$child.ProcessId)
      Say ("powershell child {0} started (pid {1})" -f $seenShells.Count, $child.ProcessId)
      $want = @{ 1 = 1; 2 = 2; 3 = 3; 6 = 4 }
      if ($want.ContainsKey($step) -and $seenShells.Count -eq $want[$step]) {
        # Frozen while it waits on its own child; the child finishes the step.
        $frozen = [AloWalk.Freeze]::NtSuspendProcess($process.Handle)
        Say ("installer frozen: status {0:X8}" -f $frozen)
        Wait-Process -Id $child.ProcessId -Timeout 120 -ErrorAction SilentlyContinue
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $killed = $true
      } elseif ($step -eq 5 -and $seenShells.Count -eq 4) {
        FreezeAndKill $true
        $killed = $true
      }
      if ($killed) { break }
    }
  }
  if ($killed) { break }
  if ($step -eq 4) {
    if ($null -eq $areaLetter) {
      $area = TheArea
      if ($null -ne $area -and $area.DriveLetter) { $areaLetter = [string]$area.DriveLetter; Say "the area's letter: $areaLetter" }
    } else {
      $chosen = "{0}:\EFI\BOOT\chosen.cfg" -f $areaLetter
      $until = (Get-Date).AddSeconds(2)
      while ((Get-Date) -lt $until) {
        if ([System.IO.File]::Exists($chosen)) { FreezeAndKill $true; $killed = $true; break }
        Start-Sleep -Milliseconds 2
      }
      if ($killed) { break }
      continue
    }
  }
  if ($step -eq 7 -and (StepIsDone 7)) { FreezeAndKill $true; $killed = $true; break }
  Start-Sleep -Milliseconds 50
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

# Where the kill landed, read from Windows' own tools — each step's effect
# present, and the next step's absent.
$area = TheArea
$fw = & "$env:SystemRoot\System32\bcdedit.exe" /enum '{fwbootmgr}' 2>&1 | Out-String
$all = & "$env:SystemRoot\System32\bcdedit.exe" /enum firmware 2>&1 | Out-String
$entry = $null
if ($all -match '(?s)identifier\s+(\{[0-9a-f-]+\})\s+device[^\r\n]*\r?\n\s*path\s+\\EFI\\BOOT\\BOOTX64\.EFI\s+description\s+alo OS') { $entry = $Matches[1] }
$hasAnyAloEntry = $all -match 'description\s+alo OS'
$volume = if ($null -ne $area) { Get-Volume -Partition $area -ErrorAction SilentlyContinue } else { $null }
$facts = [ordered]@{
  'windows smaller'   = ((Get-Partition -DriveLetter C).Size -lt $script:BaseWindowsSize)
  'area made'         = ($null -ne $area)
  'area formatted'    = ($null -ne $volume -and $volume.FileSystem -like 'FAT*' -and $volume.FileSystemLabel -eq 'ALO-INSTALL')
  'area has a letter' = ($null -ne $area -and [bool]$area.DriveLetter)
  'choice written'    = ($null -ne $area -and [bool]$area.DriveLetter -and (Test-Path ("{0}:\EFI\BOOT\chosen.cfg" -f $area.DriveLetter)))
  'any alo OS entry'  = [bool]$hasAnyAloEntry
  'entry complete and listed' = ($null -ne $entry -and $fw -match [regex]::Escape($entry))
  'next start set'    = ($fw -match 'bootsequence')
}
$facts.GetEnumerator() | ForEach-Object { Say ("fact: {0} = {1}" -f $_.Key, $_.Value) }
$f = $facts
$landed = switch ($step) {
  1 { $f['windows smaller'] -and -not $f['area made'] }
  2 { $f['area made'] -and -not $f['area formatted'] }
  3 { $f['area formatted'] -and $f['area has a letter'] -and -not $f['choice written'] }
  4 { $f['choice written'] -and -not $f['any alo OS entry'] }
  5 { $f['entry complete and listed'] -and $f['area has a letter'] -and -not $f['next start set'] }
  6 { $f['entry complete and listed'] -and -not $f['area has a letter'] -and -not $f['next start set'] }
  7 { $f['next start set'] }
}
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

# ---------------------------------------------------------------------------
# The probe: which way of making an entry does the firmware store rightly?
# ---------------------------------------------------------------------------
#
# Measured on 2026-09-21: the entry program.rs makes — `bcdedit /copy
# {bootmgr}`, then `device partition=<area>`, then `path` — reaches the
# firmware's Boot#### variable with the path and **without the device**: the
# firmware starts \EFI\BOOT\BOOTX64.EFI from Windows' own EFI system partition.
# Each variant below makes one entry; the host then reads the firmware's
# variable store and says what each one holds.
if ($instruction['probe'] -eq 'entries') {
  $area = TheArea
  $letter = [string]$area.DriveLetter
  Say "probe: the area is partition $($area.PartitionNumber) offset=$($area.Offset) size=$($area.Size) guid=$($area.Guid) letter=[$letter]"
  $bcd = "$env:SystemRoot\System32\bcdedit.exe"
  function NewEntry([string]$called) {
    $made = & $bcd /copy '{bootmgr}' /d $called 2>&1 | Out-String
    if ($made -match '(\{[0-9a-f-]+\})') { return $Matches[1] } else { Say "probe ${called}: /copy said $made"; return $null }
  }
  function Run([string[]]$words) { $out = & $bcd @words 2>&1 | Out-String; Say ("probe: bcdedit {0} -> {1}" -f ($words -join ' '), $out.Trim()) }
  $a = NewEntry 'probe-A device then path'
  if ($a) { Run @('/set', $a, 'device', "partition=$($letter):"); Run @('/set', $a, 'path', '\EFI\BOOT\BOOTX64.EFI') }
  $b = NewEntry 'probe-B path then device'
  if ($b) { Run @('/set', $b, 'path', '\EFI\BOOT\BOOTX64.EFI'); Run @('/set', $b, 'device', "partition=$($letter):") }
  $c = NewEntry 'probe-C device path device'
  if ($c) { Run @('/set', $c, 'device', "partition=$($letter):"); Run @('/set', $c, 'path', '\EFI\BOOT\BOOTX64.EFI'); Run @('/set', $c, 'device', "partition=$($letter):") }
  $volume = (Get-Volume -Partition $area).Path
  $d = NewEntry 'probe-D device by volume'
  if ($d) { Run @('/set', $d, 'device', "partition=$($volume.TrimEnd('\'))"); Run @('/set', $d, 'path', '\EFI\BOOT\BOOTX64.EFI') }
  foreach ($e in @($a, $b, $c, $d)) { if ($e) { Run @('/set', '{fwbootmgr}', 'displayorder', $e, '/addlast') } }

  # E: the load option written directly, the way the UEFI specification lays
  # it out, through SetFirmwareEnvironmentVariableEx.
  try {
    Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class AloFirmware {
  [StructLayout(LayoutKind.Sequential)] struct Luid { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] struct Privilege { public uint Count; public Luid Id; public uint Attributes; }
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool OpenProcessToken(IntPtr p, uint access, out IntPtr token);
  [DllImport("advapi32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool LookupPrivilegeValue(string s, string name, out Luid id);
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool AdjustTokenPrivileges(IntPtr t, bool none, ref Privilege p, uint len, IntPtr prev, IntPtr ret);
  [DllImport("kernel32.dll")] static extern IntPtr GetCurrentProcess();
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] public static extern bool SetFirmwareEnvironmentVariableEx(string name, string guid, byte[] value, uint size, uint attributes);
  public static string Enable() {
    IntPtr token; if (!OpenProcessToken(GetCurrentProcess(), 0x28, out token)) return "OpenProcessToken " + Marshal.GetLastWin32Error();
    Luid id; if (!LookupPrivilegeValue(null, "SeSystemEnvironmentPrivilege", out id)) return "Lookup " + Marshal.GetLastWin32Error();
    Privilege p = new Privilege { Count = 1, Id = id, Attributes = 2 };
    if (!AdjustTokenPrivileges(token, false, ref p, 0, IntPtr.Zero, IntPtr.Zero)) return "Adjust " + Marshal.GetLastWin32Error();
    return "enabled " + Marshal.GetLastWin32Error();
  }
}
'@
    Say "probe E: privilege $([AloFirmware]::Enable())"
    $sig = ([Guid]$area.Guid).ToByteArray()
    $hd = New-Object byte[] 42
    $hd[0] = 4; $hd[1] = 1; $hd[2] = 42; $hd[3] = 0
    [BitConverter]::GetBytes([uint32]$area.PartitionNumber).CopyTo($hd, 4)
    [BitConverter]::GetBytes([uint64]($area.Offset / 512)).CopyTo($hd, 8)
    [BitConverter]::GetBytes([uint64]($area.Size / 512)).CopyTo($hd, 16)
    $sig.CopyTo($hd, 24)
    $hd[40] = 2; $hd[41] = 2
    $file = [Text.Encoding]::Unicode.GetBytes("\EFI\BOOT\BOOTX64.EFI`0")
    $fp = New-Object byte[] (4 + $file.Length)
    $fp[0] = 4; $fp[1] = 4
    [BitConverter]::GetBytes([uint16]$fp.Length).CopyTo($fp, 2)
    $file.CopyTo($fp, 4)
    $end = [byte[]](0x7f, 0xff, 4, 0)
    [byte[]]$list = $hd + $fp + $end
    $desc = [Text.Encoding]::Unicode.GetBytes("probe-E written directly`0")
    [byte[]]$option = [BitConverter]::GetBytes([uint32]1) + [BitConverter]::GetBytes([uint16]$list.Length) + $desc + $list
    Say ("probe E: load option {0} bytes: {1}" -f $option.Length, [BitConverter]::ToString($option))
    $ok = [AloFirmware]::SetFirmwareEnvironmentVariableEx('Boot00A0', '{8BE4DF61-93CA-11D2-AA0D-00E098032B8C}', $option, [uint32]$option.Length, 7)
    Say "probe E: Boot00A0 written: $ok error=$([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
  } catch { Say "probe E failed: $($_.Exception.Message)" }

  & $bcd /enum firmware 2>&1 | ForEach-Object { Say "probe-enum: $_" }
  function DumpBootVars([string]$when) {
    for ($n = 0; $n -lt 0xB0; $n++) {
      $name = 'Boot{0:X4}' -f $n
      $buffer = New-Object byte[] 4096
      $size = [AloFirmwareRead]::GetFirmwareEnvironmentVariableEx($name, '{8BE4DF61-93CA-11D2-AA0D-00E098032B8C}', $buffer, 4096, [ref]0)
      if ($size -eq 0) { continue }
      $length = [BitConverter]::ToUInt16($buffer, 4)
      $end = 6; while (-not ($buffer[$end] -eq 0 -and $buffer[$end + 1] -eq 0)) { $end += 2 }
      $description = [Text.Encoding]::Unicode.GetString($buffer, 6, $end - 6)
      $pathAt = $end + 2
      $node = "type=$($buffer[$pathAt]) sub=$($buffer[$pathAt + 1])"
      if ($buffer[$pathAt] -eq 4 -and $buffer[$pathAt + 1] -eq 1) { $node += " partition=$([BitConverter]::ToUInt32($buffer, $pathAt + 4)) start=$([BitConverter]::ToUInt64($buffer, $pathAt + 8))" }
      $optionalAt = $pathAt + $length
      $optional = if ($size -gt $optionalAt) { [Text.Encoding]::ASCII.GetString($buffer, $optionalAt, [Math]::Min(16, $size - $optionalAt)) -replace '[^ -~]', '.' } else { '' }
      Say ("bootvar ($when) {0}: [{1}] {2} optional-data={3} bytes [{4}]" -f $name, $description, $node, ($size - $optionalAt), $optional)
    }
  }
  try {
    Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class AloFirmwareRead {
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)]
  public static extern uint GetFirmwareEnvironmentVariableEx(string name, string guid, byte[] buffer, uint size, ref uint attributes);
}
'@
    DumpBootVars 'right after they were made'
    Start-Sleep -Seconds 60
    DumpBootVars 'a minute later'
  } catch { Say "probe: reading the firmware's variables failed: $($_.Exception.Message)" }
  # The letter goes back the way the installer takes it, so the disk is as a
  # kill after step 6 would leave it apart from the probes.
  Remove-PartitionAccessPath -DiskNumber $area.DiskNumber -PartitionNumber $area.PartitionNumber -AccessPath "$($letter):\" -ErrorAction SilentlyContinue
  Say 'probe: done'
}
Say "ALOWALK-DONE kill-at-step $step"
